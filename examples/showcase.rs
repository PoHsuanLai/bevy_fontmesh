//! Showcase example - designed for screenshots and GIFs
//!
//! Black background, "BEVY" text with per-glyph metallic materials,
//! animated color cycling, and a horizontally orbiting camera.

use bevy::prelude::*;
use bevy_fontmesh::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_fontmesh showcase".to_string(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (assign_glyph_materials, animate_glyphs, orbit_camera),
        )
        .run();
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    angle: f32,
    height: f32,
    speed: f32,
}

#[derive(Component)]
struct CameraLight;

#[derive(Component)]
struct GlyphAnimated {
    /// Index into METAL_COLORS — which metal this glyph cycles between
    metal_index: usize,
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ambient environment light — soft overall illumination
    commands.spawn(AmbientLight {
        color: Color::srgb(0.9, 0.92, 1.0),
        brightness: 800.0,
        ..default()
    });

    // Camera — lights are children so highlights stay fixed relative to viewer
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            OrbitCamera {
                radius: 12.0,
                angle: 0.0,
                height: 0.5,
                speed: 0.4,
            },
        ))
        .with_children(|parent| {
            // Key light — strong warm, upper right
            parent.spawn((
                PointLight {
                    intensity: 200_000.0,
                    shadows_enabled: true,
                    color: Color::srgb(1.0, 0.95, 0.85),
                    range: 40.0,
                    ..default()
                },
                Transform::from_xyz(3.0, 4.0, 2.0),
                CameraLight,
            ));
            // Fill light — left, neutral white
            parent.spawn((
                PointLight {
                    intensity: 100_000.0,
                    shadows_enabled: false,
                    color: Color::srgb(0.9, 0.92, 1.0),
                    range: 40.0,
                    ..default()
                },
                Transform::from_xyz(-4.0, 1.0, 2.0),
                CameraLight,
            ));
            // Top light — shows depth faces
            parent.spawn((
                PointLight {
                    intensity: 80_000.0,
                    shadows_enabled: false,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    range: 40.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 6.0, 0.0),
                CameraLight,
            ));
            // Rim light — catches back edges
            parent.spawn((
                PointLight {
                    intensity: 60_000.0,
                    shadows_enabled: false,
                    color: Color::srgb(0.7, 0.8, 1.0),
                    range: 40.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -8.0),
                CameraLight,
            ));
        });

    commands.spawn(TextMeshGlyphsBundle {
        text_glyphs: TextMeshGlyphs {
            text: "BEVY".to_string(),
            font: asset_server.load("fonts/Inter-Bold.otf"),
            style: TextMeshStyle {
                depth: 0.6,
                subdivision: 64,
                anchor: TextAnchor::Center,
                ..default()
            },
        },
        material: MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            // Render both sides so the back face stays visible when viewed
            // through the front face's hole-punches (the 'B' counters).
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        transform: Transform::from_scale(Vec3::splat(4.0)),
        ..default()
    });
}

// ── Metal palette ─────────────────────────────────────────────────────────────

/// Real-world metallic base colors (linear sRGB approximations).
/// Each entry is (color, roughness) — metals vary in both.
const METAL_COLORS: &[(Color, f32)] = &[
    (Color::srgb(0.72, 0.45, 0.20), 0.20), // bronze
    (Color::srgb(0.91, 0.91, 0.91), 0.10), // aluminum
    (Color::srgb(0.83, 0.69, 0.22), 0.15), // gold
    (Color::srgb(0.95, 0.93, 0.88), 0.08), // silver
    (Color::srgb(0.56, 0.57, 0.58), 0.25), // steel
    (Color::srgb(0.72, 0.26, 0.05), 0.30), // copper
];

// ── Systems ───────────────────────────────────────────────────────────────────

/// Assigns each glyph a distinct starting metal, offset so no two neighbours match.
fn assign_glyph_materials(
    mut commands: Commands,
    glyph_query: Query<(Entity, &GlyphMesh), Added<GlyphMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, glyph) in glyph_query.iter() {
        let metal_index = glyph.char_index % METAL_COLORS.len();
        let (color, roughness) = METAL_COLORS[metal_index];

        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 1.0,
                perceptual_roughness: roughness,
                reflectance: 1.0,
                double_sided: true,
                cull_mode: None,
                ..default()
            })),
            GlyphAnimated { metal_index },
        ));
    }
}

/// Slowly blends each glyph between its metal and the next, creating a living sheen.
fn animate_glyphs(
    time: Res<Time>,
    glyph_query: Query<(&GlyphAnimated, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for (anim, mat_handle) in glyph_query.iter() {
        let Some(mat) = materials.get_mut(mat_handle) else {
            continue;
        };

        // Slowly oscillate between this metal and the next (period ~6 s)
        let phase = (t * 0.5 + anim.metal_index as f32 * 1.2).sin() * 0.5 + 0.5;
        let next_index = (anim.metal_index + 1) % METAL_COLORS.len();
        let (Color::Srgba(a), ra) = (
            METAL_COLORS[anim.metal_index].0,
            METAL_COLORS[anim.metal_index].1,
        ) else {
            continue;
        };
        let (Color::Srgba(b), rb) = (METAL_COLORS[next_index].0, METAL_COLORS[next_index].1) else {
            continue;
        };

        mat.base_color = Color::srgb(
            a.red + (b.red - a.red) * phase,
            a.green + (b.green - a.green) * phase,
            a.blue + (b.blue - a.blue) * phase,
        );
        mat.perceptual_roughness = ra + (rb - ra) * phase;
    }
}

/// Orbits the camera horizontally around the world origin.
fn orbit_camera(time: Res<Time>, mut query: Query<(&mut Transform, &mut OrbitCamera)>) {
    for (mut transform, mut orbit) in query.iter_mut() {
        orbit.angle += orbit.speed * time.delta_secs();

        let x = orbit.angle.sin() * orbit.radius;
        let z = orbit.angle.cos() * orbit.radius;

        *transform = Transform::from_xyz(x, orbit.height, z).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
