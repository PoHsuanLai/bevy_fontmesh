use bevy::prelude::*;
use bevy_fontmesh::{FontMeshPlugin, TextMesh, TextMeshBundle, TextMeshStyle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - Zoomed in and lower, slightly left
    commands.spawn(Camera3d::default()).insert(
        Transform::from_xyz(-0.5, -1.0, 5.0).looking_at(Vec3::new(-0.24, -0.5, -0.2), Vec3::Y),
    );

    // Key Light
    commands
        .spawn(PointLight {
            intensity: 5000.0,
            shadows_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    // Fill Light (Blue-ish)
    commands
        .spawn(PointLight {
            intensity: 2000.0,
            color: Color::srgb(0.5, 0.5, 1.0),
            ..default()
        })
        .insert(Transform::from_xyz(-4.0, 2.0, 4.0));

    // Add AmbientLight
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0, // Significantly brighter to act as "environment" lighting substitute
        affects_lightmapped_meshes: true,
    });

    // Text
    commands.spawn(TextMeshBundle {
        text_mesh: TextMesh {
            text: "FontMesh".to_string(),
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            style: TextMeshStyle {
                depth: 1.0,
                subdivision: 20,
                ..default()
            },
        },
        material: MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.8), // Blueish metallic
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        transform: Transform::from_xyz(-2.5, 0.0, 0.0),
        ..default()
    });
}
