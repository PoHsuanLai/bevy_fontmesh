//! Per-glyph text rendering example
//!
//! This example demonstrates the TextMeshGlyphs component which spawns
//! a separate entity for each character, allowing per-character styling.

use bevy::prelude::*;
use bevy_fontmesh::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, color_glyphs)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        .spawn(Camera3d::default())
        .insert(Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y));

    // Key Light
    commands
        .spawn(PointLight {
            intensity: 5000.0,
            shadows_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    // Ambient Light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: true,
    });

    // Spawn text with per-glyph entities
    // Each character will be its own entity that can be styled independently
    commands.spawn(TextMeshGlyphsBundle {
        text_glyphs: TextMeshGlyphs {
            text: "Hello\nWorld".to_string(),
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            style: TextMeshStyle {
                depth: 0.3,
                subdivision: 20,
                ..default()
            },
        },
        material: MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        })),
        transform: Transform::from_xyz(-2.0, 1.0, 0.0),
        ..default()
    });
}

/// System to color each glyph differently based on its character
fn color_glyphs(
    mut commands: Commands,
    glyph_query: Query<(Entity, &GlyphMesh), Added<GlyphMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, glyph) in glyph_query.iter() {
        // Create a unique color based on the character
        let hue = (glyph.character as u32 % 26) as f32 / 26.0;
        let color = Color::hsl(hue * 360.0, 0.8, 0.5);

        let material = materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.7,
            perceptual_roughness: 0.3,
            ..default()
        });

        commands.entity(entity).insert(MeshMaterial3d(material));
    }
}
