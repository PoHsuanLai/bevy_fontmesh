use bevy::prelude::*;
use bevy_fontmesh::{FontMeshPlugin, TextAnchor, TextMesh, TextMeshBundle, TextMeshStyle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Camera
    commands
        .spawn(Camera3d::default())
        .insert(Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y));

    // Light
    commands
        .spawn(PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let base_material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.1,
        ..default()
    }));

    // Pivot marker assets
    let pivot_mesh = meshes.add(Sphere::new(0.1));
    let pivot_mat = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // Red
        unlit: true,
        ..default()
    }));

    // Helper closure to spawn text with pivot
    let mut spawn_example = |text: &str, anchor: TextAnchor, pos: Vec3| {
        // Text
        commands.spawn(TextMeshBundle {
            text_mesh: TextMesh {
                text: text.to_string(),
                font: font.clone(),
                style: TextMeshStyle {
                    depth: 0.1,
                    subdivision: 20,
                    anchor,
                    ..default()
                },
            },
            material: base_material.clone(),
            transform: Transform::from_translation(pos),
            ..default()
        });

        // Pivot Marker
        commands.spawn((
            Mesh3d(pivot_mesh.clone()),
            pivot_mat.clone(),
            Transform::from_translation(pos),
        ));
    };

    // Example 1: Multiline with TopLeft anchor (Text hangs below pivot)
    spawn_example(
        "Hello\nWorld",
        TextAnchor::TopLeft,
        Vec3::new(-10.0, 1.3, 0.0),
    );

    // Example 2: Multiline with Center anchor (Text centered on pivot)
    spawn_example("Hello\nWorld", TextAnchor::Center, Vec3::new(0.0, 2.5, 0.0));

    // Example 3: Multiline with BottomRight anchor (Text sits above pivot)
    spawn_example(
        "Hello\nWorld",
        TextAnchor::BottomRight,
        Vec3::new(10.0, 3.7, 0.0),
    );

    // Example 4: Longer multiline text with Center anchor
    spawn_example(
        "Hello\nfrom\nFontMesh",
        TextAnchor::Center,
        Vec3::new(0.0, -4.0, 0.0),
    );
}
