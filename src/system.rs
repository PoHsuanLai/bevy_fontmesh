use crate::component::{
    GlyphMesh, JustifyText, ScreenSize, ScreenSizeCamera, TextAnchor, TextMesh, TextMeshGlyphs,
};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::MessageReader;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::mesh::Indices;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::text::{CosmicFontSystem, Font};
use cosmic_text::{fontdb, Attrs, Buffer, Family, Metrics, Shaping, Wrap};
use fontmesh::{glyph_to_mesh_3d, parse_font, FontRef, GlyphId};

// Cosmic-text shapes in pixel space. We pick a fixed pixel size and divide
// back out so positions land in the same em-normalized space (1 em = 1.0)
// that fontmesh emits outlines in.
const SHAPING_FONT_SIZE: f32 = 64.0;
const LINE_HEIGHT_FACTOR: f32 = 1.2;

// ── Font registration cache ──────────────────────────────────────────────────

/// Tracks which `Handle<Font>` we've already pushed into cosmic-text's font
/// database, plus the family name we need to pass back into `Attrs` so cosmic
/// picks that exact face. Without this we'd re-register the same bytes every
/// frame, ballooning the db.
#[derive(Resource, Default)]
pub struct FontMeshRegistry {
    registered: HashMap<AssetId<Font>, RegisteredFont>,
}

struct RegisteredFont {
    family: String,
}

impl FontMeshRegistry {
    fn ensure_registered(
        &mut self,
        handle: &Handle<Font>,
        fonts: &Assets<Font>,
        font_system: &mut cosmic_text::FontSystem,
    ) -> Option<&RegisteredFont> {
        let id = handle.id();
        if !self.registered.contains_key(&id) {
            let font_asset = fonts.get(id)?;
            let data = font_asset.data.clone();
            let face_ids = font_system
                .db_mut()
                .load_font_source(fontdb::Source::Binary(data));
            let face_id = *face_ids.last()?;
            let face = font_system.db().face(face_id)?;
            let family = face.families.first()?.0.clone();
            self.registered.insert(id, RegisteredFont { family });
        }
        self.registered.get(&id)
    }

    fn invalidate(&mut self, id: AssetId<Font>) {
        self.registered.remove(&id);
    }
}

pub fn on_font_asset_event(
    mut events: MessageReader<AssetEvent<Font>>,
    mut registry: ResMut<FontMeshRegistry>,
    mut cache: ResMut<GlyphMeshCache>,
) {
    for ev in events.read() {
        match ev {
            &AssetEvent::Modified { id }
            | &AssetEvent::Removed { id }
            | &AssetEvent::Unused { id } => {
                registry.invalidate(id);
                cache.invalidate_font(id);
            }
            AssetEvent::Added { .. } | AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
}

// ── Glyph mesh cache ──────────────────────────────────────────────────────────

/// Caches generated per-glyph meshes keyed by `(font, glyph id, depth, subdivision)`.
///
/// Cosmic-text returns the same glyph id many times across a paragraph (every
/// 'e' in "the", etc.), and the layout system also runs every time the text
/// changes; without a cache we'd retessellate identical glyphs over and over.
#[derive(Resource, Default)]
pub struct GlyphMeshCache {
    entries: HashMap<GlyphMeshKey, CachedGlyph>,
}

#[derive(Clone)]
struct CachedGlyph {
    handle: Handle<Mesh>,
    /// Axis-aligned bounds of the cached mesh in the glyph's local frame
    /// (before per-glyph layout offset). Pre-computed so per-glyph layout
    /// can derive paragraph bounds without going through `Assets<Mesh>`.
    min: Vec3,
    max: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphMeshKey {
    font: AssetId<Font>,
    glyph_id: u16,
    /// Depth quantized to 4 decimal places to keep the key hashable while
    /// still letting users tweak depth freely.
    depth_q: i32,
    subdivision: u8,
}

impl GlyphMeshCache {
    fn key(font: AssetId<Font>, glyph_id: u16, depth: f32, subdivision: u8) -> GlyphMeshKey {
        GlyphMeshKey {
            font,
            glyph_id,
            depth_q: (depth * 10_000.0).round() as i32,
            subdivision,
        }
    }

    fn invalidate_font(&mut self, font: AssetId<Font>) {
        self.entries.retain(|k, _| k.font != font);
    }
}

fn get_or_build_glyph_mesh(
    cache: &mut GlyphMeshCache,
    meshes: &mut Assets<Mesh>,
    font_ref: &FontRef,
    font_id: AssetId<Font>,
    glyph_id: u16,
    depth: f32,
    subdivision: u8,
) -> Option<CachedGlyph> {
    let key = GlyphMeshCache::key(font_id, glyph_id, depth, subdivision);
    if let Some(cached) = cache.entries.get(&key) {
        return Some(cached.clone());
    }
    let mesh_data =
        glyph_to_mesh_3d(font_ref, GlyphId::new(glyph_id as u32), depth, subdivision).ok()?;
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for v in &mesh_data.vertices {
        let p = Vec3::new(v.x, v.y, v.z);
        min = min.min(p);
        max = max.max(p);
    }
    let handle = meshes.add(fontmesh_to_bevy(&mesh_data));
    let cached = CachedGlyph { handle, min, max };
    cache.entries.insert(key, cached.clone());
    Some(cached)
}

// ── Mesh helpers ──────────────────────────────────────────────────────────────

fn fontmesh_to_bevy(mesh_data: &fontmesh::Mesh3D) -> Mesh {
    let vertices: Vec<[f32; 3]> = mesh_data.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let normals: Vec<[f32; 3]> = mesh_data.normals.iter().map(|n| [n.x, n.y, n.z]).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(mesh_data.indices.clone()));
    mesh
}

// ── Layout: cosmic-text → fontmesh ────────────────────────────────────────────

#[derive(Clone, Copy)]
struct ShapedGlyph {
    glyph_id: u16,
    char_index: usize,
    line_index: usize,
    /// First scalar from the source cluster, used for the per-glyph
    /// `GlyphMesh.character` field. For multi-char clusters (ligatures,
    /// combining marks) this is the leading character.
    character: char,
    /// Em-normalized position (cosmic coordinates / SHAPING_FONT_SIZE).
    position: Vec2,
}

/// Shape `text` using cosmic-text against the font registered as `face_id`,
/// returning em-normalized glyph positions and the y range we'll need for
/// anchor offset later.
fn shape_text(
    text: &str,
    family: &str,
    justify: JustifyText,
    font_system: &mut cosmic_text::FontSystem,
) -> (Vec<ShapedGlyph>, f32, f32) {
    let metrics = Metrics::new(SHAPING_FONT_SIZE, SHAPING_FONT_SIZE * LINE_HEIGHT_FACTOR);
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_wrap(font_system, Wrap::None);
    buffer.set_size(font_system, None, None);

    let attrs = Attrs::new().family(Family::Name(family)).metrics(metrics);
    let cosmic_align = Some(match justify {
        JustifyText::Left => cosmic_text::Align::Left,
        JustifyText::Center => cosmic_text::Align::Center,
        JustifyText::Right => cosmic_text::Align::Right,
    });
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced, cosmic_align);
    buffer.shape_until_scroll(font_system, false);

    let scale = 1.0 / SHAPING_FONT_SIZE;
    let line_height_em = LINE_HEIGHT_FACTOR;

    let mut shaped = Vec::new();
    let mut max_y_top = f32::NEG_INFINITY;
    let mut min_y_bottom = f32::INFINITY;

    for (line_index, run) in buffer.layout_runs().enumerate() {
        // Cosmic uses pixel-down y. Flip so line 0 is on top.
        let line_y_em = -(line_index as f32) * line_height_em;
        let line_text = run.text;
        for glyph in run.glyphs {
            let glyph_x_em = (glyph.x + glyph.x_offset) * scale;
            let glyph_y_em = line_y_em - glyph.y_offset * scale;

            // For ligature clusters this only exposes the leading codepoint.
            let character = line_text
                .get(glyph.start..)
                .and_then(|s| s.chars().next())
                .unwrap_or('\0');

            shaped.push(ShapedGlyph {
                glyph_id: glyph.glyph_id,
                char_index: glyph.start,
                line_index,
                character,
                position: Vec2::new(glyph_x_em, glyph_y_em),
            });
        }
        let line_top_em = line_y_em + run.line_height * scale * 0.5;
        let line_bottom_em = line_y_em - run.line_height * scale * 0.5;
        max_y_top = max_y_top.max(line_top_em);
        min_y_bottom = min_y_bottom.min(line_bottom_em);
    }

    (shaped, min_y_bottom, max_y_top)
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

// ── Combined-mesh builder (single Mesh3d output) ─────────────────────────────

fn combine_shaped_meshes(glyphs: &[(ShapedGlyph, fontmesh::Mesh3D)], anchor_offset: Vec3) -> Mesh {
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    let mut all_normals: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut index_offset = 0u32;

    for (glyph, mesh_data) in glyphs {
        let ox = glyph.position.x + anchor_offset.x;
        let oy = glyph.position.y + anchor_offset.y;
        let oz = anchor_offset.z;
        all_vertices.extend(
            mesh_data
                .vertices
                .iter()
                .map(|v| [v.x + ox, v.y + oy, v.z + oz]),
        );
        all_normals.extend(mesh_data.normals.iter().map(|n| [n.x, n.y, n.z]));
        all_indices.extend(mesh_data.indices.iter().map(|i| i + index_offset));
        index_offset += mesh_data.vertices.len() as u32;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, all_vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, all_normals);
    mesh.insert_indices(Indices::U32(all_indices));
    mesh
}

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TextMeshComputed;

#[derive(Component)]
pub struct TextMeshGlyphsComputed;

// ── Systems ───────────────────────────────────────────────────────────────────

/// Shared resources used by both mesh-generation systems. Bundled into a single
/// [`SystemParam`] so the per-system signatures stay under clippy's
/// `too_many_arguments` threshold without scattering related state across many
/// arguments.
#[derive(SystemParam)]
pub struct FontMeshResources<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    fonts: Res<'w, Assets<Font>>,
    font_system: ResMut<'w, CosmicFontSystem>,
    registry: ResMut<'w, FontMeshRegistry>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct TextMeshData {
    entity: Entity,
    text_mesh: &'static TextMesh,
    mesh: &'static mut Mesh3d,
}

type TextMeshFilter = Or<(Changed<TextMesh>, Without<TextMeshComputed>)>;

pub fn update_text_meshes(
    mut commands: Commands,
    mut res: FontMeshResources,
    mut query: Query<TextMeshData, TextMeshFilter>,
) {
    for TextMeshDataItem {
        entity,
        text_mesh,
        mesh: mut mesh_handle,
    } in query.iter_mut()
    {
        let family = match res.registry.ensure_registered(
            &text_mesh.font,
            &res.fonts,
            &mut res.font_system.0,
        ) {
            Some(r) => r.family.clone(),
            None => continue,
        };
        let Some(font_asset) = res.fonts.get(&text_mesh.font) else {
            continue;
        };
        let Ok(font_ref) = parse_font(&font_asset.data) else {
            continue;
        };

        let (shaped, _y_min, _y_max) = shape_text(
            &text_mesh.text,
            &family,
            text_mesh.style.justify,
            &mut res.font_system.0,
        );

        // Combined-mesh path: every TextMesh produces a single Mesh3d, so
        // we skip the per-glyph cache and tessellate inline.
        let mut per_glyph: Vec<(ShapedGlyph, fontmesh::Mesh3D)> = Vec::with_capacity(shaped.len());
        let mut min_bound = Vec3::splat(f32::MAX);
        let mut max_bound = Vec3::splat(f32::MIN);
        for g in shaped {
            let Ok(mesh_data) = glyph_to_mesh_3d(
                &font_ref,
                GlyphId::new(g.glyph_id as u32),
                text_mesh.style.depth,
                text_mesh.style.subdivision,
            ) else {
                continue;
            };
            for v in &mesh_data.vertices {
                let pos = Vec3::new(v.x + g.position.x, v.y + g.position.y, v.z);
                min_bound = min_bound.min(pos);
                max_bound = max_bound.max(pos);
            }
            per_glyph.push((g, mesh_data));
        }

        let anchor_offset = if per_glyph.is_empty() {
            Vec3::ZERO
        } else {
            calculate_anchor_offset(text_mesh.style.anchor, min_bound, max_bound)
        };

        let combined = combine_shaped_meshes(&per_glyph, anchor_offset);
        mesh_handle.0 = res.meshes.add(combined);
        commands.entity(entity).insert(TextMeshComputed);
    }
}

#[derive(QueryData)]
pub struct TextMeshGlyphsData<M: Material> {
    entity: Entity,
    text_glyphs: &'static TextMeshGlyphs,
    default_material: &'static MeshMaterial3d<M>,
}

type TextMeshGlyphsFilter = Or<(Changed<TextMeshGlyphs>, Without<TextMeshGlyphsComputed>)>;

/// System to generate per-character mesh entities for [`TextMeshGlyphs`].
pub fn update_glyph_meshes<M: Material>(
    mut commands: Commands,
    mut res: FontMeshResources,
    mut cache: ResMut<GlyphMeshCache>,
    query: Query<TextMeshGlyphsData<M>, TextMeshGlyphsFilter>,
    children_query: Query<&Children>,
    glyph_query: Query<Entity, With<GlyphMesh>>,
) {
    for TextMeshGlyphsDataItem {
        entity,
        text_glyphs,
        default_material,
    } in query.iter()
    {
        let family = match res.registry.ensure_registered(
            &text_glyphs.font,
            &res.fonts,
            &mut res.font_system.0,
        ) {
            Some(r) => r.family.clone(),
            None => continue,
        };
        let Some(font_asset) = res.fonts.get(&text_glyphs.font) else {
            continue;
        };
        let Ok(font_ref) = parse_font(&font_asset.data) else {
            continue;
        };

        despawn_existing_glyphs(&mut commands, entity, &children_query, &glyph_query);

        let (shaped, _, _) = shape_text(
            &text_glyphs.text,
            &family,
            text_glyphs.style.justify,
            &mut res.font_system.0,
        );

        let font_id = text_glyphs.font.id();

        // Bounds first so the anchor offset matches the combined-mesh path.
        // Forces a cache populate now since we need per-glyph extents.
        let mut min_bound = Vec3::splat(f32::MAX);
        let mut max_bound = Vec3::splat(f32::MIN);
        let mut to_spawn: Vec<(ShapedGlyph, Handle<Mesh>)> = Vec::with_capacity(shaped.len());

        for g in shaped {
            let Some(cached) = get_or_build_glyph_mesh(
                &mut cache,
                &mut res.meshes,
                &font_ref,
                font_id,
                g.glyph_id,
                text_glyphs.style.depth,
                text_glyphs.style.subdivision,
            ) else {
                continue;
            };
            let offset = Vec3::new(g.position.x, g.position.y, 0.0);
            min_bound = min_bound.min(cached.min + offset);
            max_bound = max_bound.max(cached.max + offset);
            to_spawn.push((g, cached.handle));
        }

        let anchor_offset = if to_spawn.is_empty() {
            Vec3::ZERO
        } else {
            calculate_anchor_offset(text_glyphs.style.anchor, min_bound, max_bound)
        };

        commands.entity(entity).with_children(|parent| {
            for (g, mesh_handle) in to_spawn {
                parent.spawn((
                    GlyphMesh {
                        char_index: g.char_index,
                        line_index: g.line_index,
                        character: g.character,
                    },
                    Mesh3d(mesh_handle),
                    default_material.clone(),
                    Transform::from_xyz(
                        g.position.x + anchor_offset.x,
                        g.position.y + anchor_offset.y,
                        anchor_offset.z,
                    ),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        });

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

// ── ScreenSize: keep text at a target on-screen pixel height ─────────────────

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
