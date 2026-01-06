use crate::component::{GlyphMesh, JustifyText, TextAnchor, TextMesh, TextMeshGlyphs};
use crate::FontMesh;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

/// Helper function to calculate the width of a line of text
#[inline]
fn calculate_line_width(line: &str, face: &fontmesh::Face) -> f32 {
    line.chars().map(|ch| get_glyph_advance(ch, face)).sum()
}

/// Helper function to get the advance width for a character
#[inline]
fn get_glyph_advance(ch: char, face: &fontmesh::Face) -> f32 {
    fontmesh::glyph_advance(face, ch).unwrap_or_else(|| {
        if ch.is_whitespace() {
            // Use font metrics for a proportional fallback space width
            // Typically ~25% of the font height is a good space width
            (fontmesh::ascender(face) - fontmesh::descender(face)) * 0.25
        } else {
            0.0
        }
    })
}

/// Helper function to calculate the X offset based on text justification
#[inline]
fn calculate_justification_offset(justify: JustifyText, line_width: f32) -> f32 {
    match justify {
        JustifyText::Left => 0.0,
        JustifyText::Center => -line_width * 0.5,
        JustifyText::Right => -line_width,
    }
}

/// Helper function to calculate anchor offset for text positioning
fn calculate_anchor_offset(anchor: TextAnchor, min_bound: Vec3, max_bound: Vec3) -> Vec3 {
    let size = max_bound - min_bound;
    let center = min_bound + size * 0.5;

    match anchor {
        TextAnchor::TopLeft => Vec3::new(-min_bound.x, -max_bound.y, 0.0),
        TextAnchor::TopCenter => Vec3::new(-center.x, -max_bound.y, 0.0),
        TextAnchor::TopRight => Vec3::new(-max_bound.x, -max_bound.y, 0.0),

        TextAnchor::CenterLeft => Vec3::new(-min_bound.x, -center.y, 0.0),
        TextAnchor::Center => Vec3::new(-center.x, -center.y, 0.0),
        TextAnchor::CenterRight => Vec3::new(-max_bound.x, -center.y, 0.0),

        TextAnchor::BottomLeft => Vec3::new(-min_bound.x, -min_bound.y, 0.0),
        TextAnchor::BottomCenter => Vec3::new(-center.x, -min_bound.y, 0.0),
        TextAnchor::BottomRight => Vec3::new(-max_bound.x, -min_bound.y, 0.0),

        TextAnchor::Custom(pivot) => {
            let pivot_pos = min_bound.truncate() + size.truncate() * pivot;
            Vec3::new(-pivot_pos.x, -pivot_pos.y, 0.0)
        }
    }
}

/// Helper function to create a Bevy mesh from vertex/normal/index data
fn create_mesh_from_data(
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Marker component indicating that a [`TextMesh`] has been processed.
#[derive(Component)]
pub struct TextMeshComputed;

/// Marker component indicating that a [`TextMeshGlyphs`] has been processed.
#[derive(Component)]
pub struct TextMeshGlyphsComputed;

type TextMeshQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static TextMesh, &'static mut Mesh3d),
    Or<(Changed<TextMesh>, Without<TextMeshComputed>)>,
>;

pub fn update_text_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    font_assets: Res<Assets<FontMesh>>,
    mut query: TextMeshQuery,
) {
    for (entity, text_mesh, mut mesh_handle) in query.iter_mut() {
        // 1. Try to get the font data
        let Some(font_asset) = font_assets.get(&text_mesh.font) else {
            // Font not loaded yet, skip this frame
            continue;
        };

        // 2. Parse font directly (no caching needed as parsing is lightweight)
        let Ok(face) = fontmesh::Face::parse(&font_asset.data, 0) else {
            // Failed to parse font, skip this entity
            continue;
        };

        // 3. Generate combined mesh
        let mut all_vertices = Vec::new();
        let mut all_normals = Vec::new();
        let mut all_indices = Vec::new();

        let mut cursor = Vec3::ZERO;
        let mut index_offset = 0;

        let line_height =
            fontmesh::ascender(&face) - fontmesh::descender(&face) + fontmesh::line_gap(&face);

        // Bounds tracking
        let mut min_bound = Vec3::splat(f32::MAX);
        let mut max_bound = Vec3::splat(f32::MIN);

        // Split text into lines for justification
        for line in text_mesh.text.split('\n') {
            // Calculate line width and X offset based on justification
            let line_width = calculate_line_width(line, &face);
            cursor.x = calculate_justification_offset(text_mesh.style.justify, line_width);

            // Generate mesh for line
            for ch in line.chars() {
                if ch.is_whitespace() {
                    cursor.x += get_glyph_advance(ch, &face);
                    continue;
                }

                // Use pure function to generate mesh
                let mesh_res = fontmesh::char_to_mesh_3d(
                    &face,
                    ch,
                    text_mesh.style.depth,
                    text_mesh.style.subdivision,
                );

                if let Ok(mesh) = mesh_res {
                    // Extend vertices and update bounds
                    all_vertices.extend(mesh.vertices.iter().map(|v| {
                        let pos = Vec3::new(v.x + cursor.x, v.y + cursor.y, v.z);
                        min_bound = min_bound.min(pos);
                        max_bound = max_bound.max(pos);
                        [pos.x, pos.y, pos.z]
                    }));

                    // Extend normals
                    all_normals.extend(mesh.normals.iter().map(|n| [n.x, n.y, n.z]));

                    // Extend indices with offset
                    all_indices.extend(mesh.indices.iter().map(|i| i + index_offset));

                    index_offset += mesh.vertices.len() as u32;
                    cursor.x += get_glyph_advance(ch, &face);
                }
            }

            // Move to next line
            cursor.y -= line_height;
        }

        // 4. Apply Anchor Offset
        if !all_vertices.is_empty() {
            let offset = calculate_anchor_offset(text_mesh.style.anchor, min_bound, max_bound);
            all_vertices.iter_mut().for_each(|v| {
                v[0] += offset.x;
                v[1] += offset.y;
                v[2] += offset.z;
            });
        }

        // 5. Create and assign Bevy Mesh
        let new_mesh = create_mesh_from_data(all_vertices, all_normals, all_indices);
        mesh_handle.0 = meshes.add(new_mesh);

        // 7. Mark as computed
        commands.entity(entity).insert(TextMeshComputed);
    }
}

type TextMeshGlyphsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static TextMeshGlyphs,
        &'static MeshMaterial3d<StandardMaterial>,
    ),
    Or<(Changed<TextMeshGlyphs>, Without<TextMeshGlyphsComputed>)>,
>;

/// System to generate per-character mesh entities for [`TextMeshGlyphs`] components.
///
/// This system spawns a separate child entity for each character in the text,
/// allowing for per-character styling, animations, and interactions.
pub fn update_glyph_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    font_assets: Res<Assets<FontMesh>>,
    query: TextMeshGlyphsQuery,
    children_query: Query<&Children>,
    glyph_query: Query<Entity, With<GlyphMesh>>,
) {
    for (entity, text_glyphs, default_material) in query.iter() {
        // 1. Try to get the font data
        let Some(font_asset) = font_assets.get(&text_glyphs.font) else {
            // Font not loaded yet, skip this frame
            continue;
        };

        // 2. Parse font directly (no caching needed as parsing is lightweight)
        let Ok(face) = fontmesh::Face::parse(&font_asset.data, 0) else {
            // Failed to parse font, skip this entity
            continue;
        };

        // 3. Despawn existing glyph children
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if glyph_query.contains(child) {
                    commands.entity(child).despawn();
                }
            }
        }

        // 4. Calculate line widths for justification
        let line_height =
            fontmesh::ascender(&face) - fontmesh::descender(&face) + fontmesh::line_gap(&face);
        let lines: Vec<&str> = text_glyphs.text.split('\n').collect();

        let line_widths: Vec<f32> = lines
            .iter()
            .map(|line| calculate_line_width(line, &face))
            .collect();

        // 5. Spawn glyph entities
        let mut char_index = 0;

        commands.entity(entity).with_children(|parent| {
            for (line_index, line) in lines.iter().enumerate() {
                let line_width = line_widths[line_index];
                let mut cursor_x =
                    calculate_justification_offset(text_glyphs.style.justify, line_width);
                let cursor_y = -(line_index as f32) * line_height;

                for ch in line.chars() {
                    let advance = get_glyph_advance(ch, &face);

                    // Skip whitespace but still count it
                    if ch.is_whitespace() {
                        cursor_x += advance;
                        char_index += 1;
                        continue;
                    }

                    // Generate mesh for this character using pure function
                    let mesh_res = fontmesh::char_to_mesh_3d(
                        &face,
                        ch,
                        text_glyphs.style.depth,
                        text_glyphs.style.subdivision,
                    );

                    if let Ok(glyph_mesh_data) = mesh_res {
                        let vertices: Vec<_> = glyph_mesh_data
                            .vertices
                            .iter()
                            .map(|v| [v.x, v.y, v.z])
                            .collect();

                        let normals: Vec<_> = glyph_mesh_data
                            .normals
                            .iter()
                            .map(|n| [n.x, n.y, n.z])
                            .collect();

                        let mesh = create_mesh_from_data(
                            vertices,
                            normals,
                            glyph_mesh_data.indices.clone(),
                        );
                        let mesh_handle = meshes.add(mesh);

                        // Spawn glyph entity as child
                        parent.spawn((
                            GlyphMesh {
                                char_index,
                                line_index,
                                character: ch,
                            },
                            Mesh3d(mesh_handle),
                            default_material.clone(),
                            Transform::from_xyz(cursor_x, cursor_y, 0.0),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ));
                    }

                    cursor_x += advance;
                    char_index += 1;
                }

                // Account for newline character in char_index
                char_index += 1;
            }
        });

        // 6. Mark as computed
        commands.entity(entity).insert(TextMeshGlyphsComputed);
    }
}

/// Helper function to generate a mesh for a single character.
///
/// This can be used to create individual glyph meshes outside of the system,
/// for example when you need to update a specific character's material.
pub fn generate_glyph_mesh(
    face: &fontmesh::Face,
    character: char,
    depth: f32,
    subdivision: u8,
) -> Option<Mesh> {
    let mesh_res = fontmesh::char_to_mesh_3d(face, character, depth, subdivision);

    mesh_res.ok().map(|glyph_mesh_data| {
        let vertices: Vec<_> = glyph_mesh_data
            .vertices
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();

        let normals: Vec<_> = glyph_mesh_data
            .normals
            .iter()
            .map(|n| [n.x, n.y, n.z])
            .collect();

        create_mesh_from_data(vertices, normals, glyph_mesh_data.indices)
    })
}
