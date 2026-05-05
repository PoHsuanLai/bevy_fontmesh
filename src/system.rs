use crate::component::{
    GlyphMesh, JustifyText, ScreenSize, ScreenSizeCamera, TextAnchor, TextMesh, TextMeshGlyphs,
};
use crate::FontMesh;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

// ── Font helpers ──────────────────────────────────────────────────────────────

#[inline]
fn get_glyph_advance(ch: char, face: &fontmesh::Face) -> f32 {
    fontmesh::glyph_advance(face, ch).unwrap_or_else(|| {
        if ch.is_whitespace() {
            (fontmesh::ascender(face) - fontmesh::descender(face)) * 0.25
        } else {
            0.0
        }
    })
}

#[inline]
fn calculate_line_width(line: &str, face: &fontmesh::Face) -> f32 {
    line.chars().map(|ch| get_glyph_advance(ch, face)).sum()
}

#[inline]
fn calculate_justification_offset(justify: JustifyText, line_width: f32) -> f32 {
    match justify {
        JustifyText::Left => 0.0,
        JustifyText::Center => -line_width * 0.5,
        JustifyText::Right => -line_width,
    }
}

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

// ── Mesh helpers ──────────────────────────────────────────────────────────────

fn fontmesh_to_bevy(mesh_data: &fontmesh::types::Mesh3D) -> Mesh {
    let vertices: Vec<[f32; 3]> = mesh_data.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let normals: Vec<[f32; 3]> = mesh_data.normals.iter().map(|n| [n.x, n.y, n.z]).collect();
    create_mesh_from_data(vertices, normals, mesh_data.indices.clone())
}

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

// ── Glyph layout ─────────────────────────────────────────────────────────────

/// A single laid-out glyph ready to be turned into a mesh or child entity.
struct PositionedGlyph {
    char_index: usize,
    line_index: usize,
    character: char,
    /// Position of this glyph relative to the text root (before anchor offset).
    position: Vec2,
    mesh_data: fontmesh::types::Mesh3D,
}

/// Lay out all glyphs for `text` using the given face and style settings.
/// Returns the list of positioned glyphs and the overall bounding box.
fn layout_glyphs(
    text: &str,
    face: &fontmesh::Face,
    depth: f32,
    subdivision: u8,
    justify: JustifyText,
) -> (Vec<PositionedGlyph>, Vec3, Vec3) {
    let line_height =
        fontmesh::ascender(face) - fontmesh::descender(face) + fontmesh::line_gap(face);
    let lines: Vec<&str> = text.split('\n').collect();

    let mut glyphs = Vec::new();
    let mut min_bound = Vec3::splat(f32::MAX);
    let mut max_bound = Vec3::splat(f32::MIN);
    let mut char_index = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_width = calculate_line_width(line, face);
        let mut cursor_x = calculate_justification_offset(justify, line_width);
        let cursor_y = -(line_idx as f32) * line_height;

        for ch in line.chars() {
            let advance = get_glyph_advance(ch, face);

            if ch.is_whitespace() {
                cursor_x += advance;
                char_index += 1;
                continue;
            }

            if let Ok(mesh_data) = fontmesh::char_to_mesh_3d(face, ch, depth, subdivision) {
                for v in &mesh_data.vertices {
                    let pos = Vec3::new(v.x + cursor_x, v.y + cursor_y, v.z);
                    min_bound = min_bound.min(pos);
                    max_bound = max_bound.max(pos);
                }

                glyphs.push(PositionedGlyph {
                    char_index,
                    line_index: line_idx,
                    character: ch,
                    position: Vec2::new(cursor_x, cursor_y),
                    mesh_data,
                });
            }

            cursor_x += advance;
            char_index += 1;
        }

        char_index += 1; // newline
    }

    (glyphs, min_bound, max_bound)
}

/// Combines positioned glyphs into a single Bevy mesh with anchor offset applied.
fn combine_glyph_meshes(glyphs: Vec<PositionedGlyph>, anchor_offset: Vec3) -> Mesh {
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    let mut all_normals: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut index_offset = 0u32;

    for glyph in glyphs {
        let ox = glyph.position.x + anchor_offset.x;
        let oy = glyph.position.y + anchor_offset.y;
        let oz = anchor_offset.z;

        all_vertices.extend(
            glyph
                .mesh_data
                .vertices
                .iter()
                .map(|v| [v.x + ox, v.y + oy, v.z + oz]),
        );
        all_normals.extend(glyph.mesh_data.normals.iter().map(|n| [n.x, n.y, n.z]));
        all_indices.extend(glyph.mesh_data.indices.iter().map(|i| i + index_offset));
        index_offset += glyph.mesh_data.vertices.len() as u32;
    }

    create_mesh_from_data(all_vertices, all_normals, all_indices)
}

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker component indicating that a [`TextMesh`] has been processed.
#[derive(Component)]
pub struct TextMeshComputed;

/// Marker component indicating that a [`TextMeshGlyphs`] has been processed.
#[derive(Component)]
pub struct TextMeshGlyphsComputed;

// ── Systems ───────────────────────────────────────────────────────────────────

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
        let Some(font_asset) = font_assets.get(&text_mesh.font) else {
            continue;
        };
        let Ok(face) = fontmesh::Face::parse(&font_asset.data, 0) else {
            continue;
        };

        let (glyphs, min_bound, max_bound) = layout_glyphs(
            &text_mesh.text,
            &face,
            text_mesh.style.depth,
            text_mesh.style.subdivision,
            text_mesh.style.justify,
        );

        let anchor_offset = if !glyphs.is_empty() {
            calculate_anchor_offset(text_mesh.style.anchor, min_bound, max_bound)
        } else {
            Vec3::ZERO
        };

        let combined_mesh = combine_glyph_meshes(glyphs, anchor_offset);
        mesh_handle.0 = meshes.add(combined_mesh);
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
pub fn update_glyph_meshes<M: Material>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    font_assets: Res<Assets<FontMesh>>,
    query: TextMeshGlyphsQuery<M>,
    children_query: Query<&Children>,
    glyph_query: Query<Entity, With<GlyphMesh>>,
) {
    for (entity, text_glyphs, default_material) in query.iter() {
        let Some(font_asset) = font_assets.get(&text_glyphs.font) else {
            continue;
        };
        let Ok(face) = fontmesh::Face::parse(&font_asset.data, 0) else {
            continue;
        };

        despawn_existing_glyphs(&mut commands, entity, &children_query, &glyph_query);

        let (glyphs, min_bound, max_bound) = layout_glyphs(
            &text_glyphs.text,
            &face,
            text_glyphs.style.depth,
            text_glyphs.style.subdivision,
            text_glyphs.style.justify,
        );

        let anchor_offset = if !glyphs.is_empty() {
            calculate_anchor_offset(text_glyphs.style.anchor, min_bound, max_bound)
        } else {
            Vec3::ZERO
        };

        spawn_glyph_children(
            &mut commands,
            &mut meshes,
            entity,
            glyphs,
            anchor_offset,
            default_material,
        );

        commands.entity(entity).insert(TextMeshGlyphsComputed);
    }
}

fn despawn_existing_glyphs(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
    glyph_query: &Query<Entity, With<GlyphMesh>>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if glyph_query.contains(child) {
                commands.entity(child).despawn();
            }
        }
    }
}

fn spawn_glyph_children<M: Material>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    entity: Entity,
    glyphs: Vec<PositionedGlyph>,
    anchor_offset: Vec3,
    default_material: &MeshMaterial3d<M>,
) {
    commands.entity(entity).with_children(|parent| {
        for glyph in glyphs {
            let mesh_handle = meshes.add(fontmesh_to_bevy(&glyph.mesh_data));
            parent.spawn((
                GlyphMesh {
                    char_index: glyph.char_index,
                    line_index: glyph.line_index,
                    character: glyph.character,
                },
                Mesh3d(mesh_handle),
                default_material.clone(),
                Transform::from_xyz(
                    glyph.position.x + anchor_offset.x,
                    glyph.position.y + anchor_offset.y,
                    anchor_offset.z,
                ),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    });
}

/// Generate a Bevy mesh for a single character. Useful for manual glyph updates.
pub fn generate_glyph_mesh(
    face: &fontmesh::Face,
    character: char,
    depth: f32,
    subdivision: u8,
) -> Option<Mesh> {
    fontmesh::char_to_mesh_3d(face, character, depth, subdivision)
        .ok()
        .as_ref()
        .map(fontmesh_to_bevy)
}

// ── ScreenSize: keep text at a target on-screen pixel height ─────────────────

/// For each entity with [`ScreenSize`], rewrite its `Transform.scale` so
/// that one line of text at unit scale covers `pixel_height` pixels of
/// the active camera's logical target.
///
/// Picks a camera by preference: any with [`ScreenSizeCamera`] if such a
/// camera exists; otherwise the first camera the query returns. For a
/// perspective camera, world-per-pixel is measured at the entity's
/// world-space depth from the camera.
pub fn scale_screen_size(
    cam_marked: Query<(&Camera, &GlobalTransform, &Projection), With<ScreenSizeCamera>>,
    cam_any: Query<(&Camera, &GlobalTransform, &Projection), Without<ScreenSizeCamera>>,
    mut targets: Query<(&ScreenSize, &GlobalTransform, &mut Transform)>,
) {
    let (camera, cam_xform, projection) = match cam_marked.single() {
        Ok(c) => c,
        Err(_) => match cam_any.iter().next() {
            Some(c) => c,
            None => return,
        },
    };
    let Some(target_size) = camera.logical_target_size() else {
        return;
    };
    let target_h = target_size.y.max(1.0);

    for (size, gxform, mut transform) in targets.iter_mut() {
        let world_per_px = match projection {
            Projection::Orthographic(ortho) => orthographic_world_per_px(ortho, target_h),
            Projection::Perspective(persp) => {
                perspective_world_per_px(persp, cam_xform, gxform, target_h)
            }
            _ => continue,
        };
        let s = size.pixel_height * world_per_px;
        if s.is_finite() && s > 0.0 {
            transform.scale = Vec3::splat(s);
        }
    }
}

#[inline]
fn orthographic_world_per_px(ortho: &OrthographicProjection, target_h: f32) -> f32 {
    let viewport_h = match ortho.scaling_mode {
        bevy::camera::ScalingMode::FixedVertical { viewport_height } => viewport_height,
        bevy::camera::ScalingMode::FixedHorizontal { viewport_width } => {
            let aspect = ortho.area.width() / ortho.area.height().max(f32::EPSILON);
            viewport_width / aspect.max(f32::EPSILON)
        }
        bevy::camera::ScalingMode::WindowSize => ortho.area.height(),
        _ => ortho.area.height(),
    };
    viewport_h / target_h
}

#[inline]
fn perspective_world_per_px(
    persp: &PerspectiveProjection,
    cam_xform: &GlobalTransform,
    target_xform: &GlobalTransform,
    target_h: f32,
) -> f32 {
    let depth = (target_xform.translation() - cam_xform.translation()).length();
    let visible_h = 2.0 * depth * (persp.fov * 0.5).tan();
    visible_h / target_h
}
