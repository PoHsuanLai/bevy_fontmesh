use crate::component::{GlyphMesh, JustifyText, TextAnchor, TextMesh, TextMeshGlyphs};
use crate::FontMesh;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use fontmesh::Font;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper function to calculate the width of a line of text
#[inline]
fn calculate_line_width(line: &str, font: &Font) -> f32 {
    line.chars().map(|ch| get_glyph_advance(ch, font)).sum()
}

/// Helper function to get the advance width for a character
#[inline]
fn get_glyph_advance(ch: char, font: &Font) -> f32 {
    font.glyph_by_char(ch)
        .map(|g| g.advance())
        .unwrap_or_else(|_| {
            if ch.is_whitespace() {
                // Use font metrics for a proportional fallback space width
                // Typically ~25% of the font height is a good space width
                (font.ascender() - font.descender()) * 0.25
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

/// Cached font entry containing owned data and parsed font
type CachedFont = (Arc<Vec<u8>>, Arc<Font<'static>>);

/// Cache for parsed font data to avoid re-parsing every frame.
///
/// This resource stores owned font data (as `Vec<u8>`) and parsed `Font` instances
/// indexed by their asset ID, allowing multiple entities to share the same parsed
/// font data without reparsing from bytes every frame.
#[derive(Resource, Default)]
pub struct ParsedFontCache {
    fonts: HashMap<AssetId<FontMesh>, CachedFont>,
}

impl ParsedFontCache {
    /// Get or parse a font from the cache.
    ///
    /// If the font is already cached, returns a clone of the Arc.
    /// Otherwise, clones the font data, parses it, and caches both for future use.
    pub fn get_or_parse(
        &mut self,
        id: AssetId<FontMesh>,
        data: &[u8],
    ) -> Option<Arc<Font<'static>>> {
        if let Some((_, font)) = self.fonts.get(&id) {
            return Some(Arc::clone(font));
        }

        // Clone data to get owned bytes, then leak it to get 'static lifetime
        let owned_data = Arc::new(data.to_vec());
        let static_slice: &'static [u8] =
            unsafe { std::slice::from_raw_parts(owned_data.as_ptr(), owned_data.len()) };

        // Parse and cache the font
        let font = Font::from_bytes(static_slice).ok()?;
        let font_arc = Arc::new(font);
        self.fonts.insert(id, (owned_data, Arc::clone(&font_arc)));
        Some(font_arc)
    }

    /// Clear cached fonts that are no longer loaded in the asset server.
    ///
    /// This prevents memory leaks when fonts are unloaded.
    pub fn cleanup(&mut self, font_assets: &Assets<FontMesh>) {
        self.fonts.retain(|id, _| font_assets.contains(*id));
    }
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
    mut font_cache: ResMut<ParsedFontCache>,
    mut query: TextMeshQuery,
) {
    for (entity, text_mesh, mut mesh_handle) in query.iter_mut() {
        // 1. Try to get the font data
        let Some(font_asset) = font_assets.get(&text_mesh.font) else {
            // Font not loaded yet, skip this frame
            continue;
        };

        // 2. Get or parse font from cache
        let Some(font) = font_cache.get_or_parse(text_mesh.font.id(), &font_asset.data) else {
            // Failed to parse font, skip this entity
            continue;
        };

        // 3. Generate combined mesh
        let mut all_vertices = Vec::new();
        let mut all_normals = Vec::new();
        let mut all_indices = Vec::new();

        let mut cursor = Vec3::ZERO;
        let mut index_offset = 0;

        let line_height = font.ascender() - font.descender() + font.line_gap();

        // Bounds tracking
        let mut min_bound = Vec3::splat(f32::MAX);
        let mut max_bound = Vec3::splat(f32::MIN);

        // Split text into lines for justification
        for line in text_mesh.text.split('\n') {
            // Calculate line width and X offset based on justification
            let line_width = calculate_line_width(line, &font);
            cursor.x = calculate_justification_offset(text_mesh.style.justify, line_width);

            // Generate mesh for line
            for ch in line.chars() {
                if ch.is_whitespace() {
                    cursor.x += get_glyph_advance(ch, &font);
                    continue;
                }

                let mesh_res = font.glyph_by_char(ch).and_then(|g| {
                    g.with_subdivisions(text_mesh.style.subdivision)
                        .to_mesh_3d(text_mesh.style.depth)
                });

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
                    cursor.x += get_glyph_advance(ch, &font);
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

type TextMeshGlyphsQuery<'w, 's, M> = Query<
    'w,
    's,
    (Entity, &'static TextMeshGlyphs, &'static MeshMaterial3d<M>),
    Or<(Changed<TextMeshGlyphs>, Without<TextMeshGlyphsComputed>)>,
>;

/// System to generate per-character mesh entities for [`TextMeshGlyphs`] components.
///
/// This system spawns a separate child entity for each character in the text,
/// allowing for per-character styling, animations, and interactions.
pub fn update_glyph_meshes<M: Material>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    font_assets: Res<Assets<FontMesh>>,
    mut font_cache: ResMut<ParsedFontCache>,
    query: TextMeshGlyphsQuery<M>,
    children_query: Query<&Children>,
    glyph_query: Query<Entity, With<GlyphMesh>>,
) {
    for (entity, text_glyphs, default_material) in query.iter() {
        // 1. Try to get the font data
        let Some(font_asset) = font_assets.get(&text_glyphs.font) else {
            // Font not loaded yet, skip this frame
            continue;
        };

        // 2. Get or parse font from cache
        let Some(font) = font_cache.get_or_parse(text_glyphs.font.id(), &font_asset.data) else {
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
        let line_height = font.ascender() - font.descender() + font.line_gap();
        let lines: Vec<&str> = text_glyphs.text.split('\n').collect();

        let line_widths: Vec<f32> = lines
            .iter()
            .map(|line| calculate_line_width(line, &font))
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
                    let advance = get_glyph_advance(ch, &font);

                    // Skip whitespace but still count it
                    if ch.is_whitespace() {
                        cursor_x += advance;
                        char_index += 1;
                        continue;
                    }

                    // Generate mesh for this character
                    let mesh_res = font.glyph_by_char(ch).and_then(|g| {
                        g.with_subdivisions(text_glyphs.style.subdivision)
                            .to_mesh_3d(text_glyphs.style.depth)
                    });

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

/// System to cleanup the font cache, removing fonts that are no longer loaded.
///
/// This runs in `PostUpdate` to prevent memory leaks from unloaded fonts.
pub fn cleanup_font_cache(
    mut font_cache: ResMut<ParsedFontCache>,
    font_assets: Res<Assets<FontMesh>>,
) {
    font_cache.cleanup(&font_assets);
}

/// Helper function to generate a mesh for a single character.
///
/// This can be used to create individual glyph meshes outside of the system,
/// for example when you need to update a specific character's material.
pub fn generate_glyph_mesh(
    font: &Font,
    character: char,
    depth: f32,
    subdivision: u8,
) -> Option<Mesh> {
    let mesh_res = font
        .glyph_by_char(character)
        .and_then(|g| g.with_subdivisions(subdivision).to_mesh_3d(depth));

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
