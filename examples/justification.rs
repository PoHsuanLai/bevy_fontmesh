use bevy::prelude::*;
use bevy_fontmesh::{FontMeshPlugin, JustifyText, TextAnchor, TextMesh, TextMeshStyle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_text)
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
        .insert(Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y));

    // Light
    commands
        .spawn(PointLight {
            intensity: 1500.0,
            shadow_maps_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let base_material = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.1,
        double_sided: true,
        cull_mode: None,
        ..default()
    }));

    // Example 1: Left Justified
    commands.spawn((
        TextMesh {
            text: "Left\nJustified\nText".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.1,
                subdivision: 20,
                anchor: TextAnchor::Center,
                justify: JustifyText::Left,
            },
        },
        base_material.clone(),
        Transform::from_xyz(-5.0, 3.0, 0.0),
    ));

    // Example 2: Center Justified
    commands.spawn((
        TextMesh {
            text: "Center\nJustified\nText".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.1,
                subdivision: 20,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
            },
        },
        base_material.clone(),
        Transform::from_xyz(0.0, 3.0, 0.0),
    ));

    // Example 3: Right Justified
    commands.spawn((
        TextMesh {
            text: "Right\nJustified\nText".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.1,
                subdivision: 20,
                anchor: TextAnchor::Center,
                justify: JustifyText::Right,
            },
        },
        base_material.clone(),
        Transform::from_xyz(5.0, 3.0, 0.0),
    ));
}

fn rotate_text(time: Res<Time>, mut query: Query<&mut Transform, With<TextMesh>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(time.delta_secs() * 0.2);
    }
}
