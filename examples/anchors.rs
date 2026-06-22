use bevy::prelude::*;
use bevy_fontmesh::{FontMeshPlugin, TextAnchor, TextMesh, TextMeshStyle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_anchors)
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
        .insert(Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y));

    // Light
    commands
        .spawn(PointLight {
            intensity: 1500.0,
            shadow_maps_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let mat = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        perceptual_roughness: 0.1,
        double_sided: true,
        cull_mode: None,
        ..default()
    }));

    // Create a grid of anchors
    let anchors = [
        (TextAnchor::TopLeft, "TopLeft", Vec3::new(-4.0, 4.0, 0.0)),
        (TextAnchor::TopCenter, "TopCenter", Vec3::new(0.0, 4.0, 0.0)),
        (TextAnchor::TopRight, "TopRight", Vec3::new(4.0, 4.0, 0.0)),
        (
            TextAnchor::CenterLeft,
            "CenterLeft",
            Vec3::new(-4.0, 0.0, 0.0),
        ),
        (TextAnchor::Center, "Center", Vec3::new(0.0, 0.0, 0.0)),
        (
            TextAnchor::CenterRight,
            "CenterRight",
            Vec3::new(4.0, 0.0, 0.0),
        ),
        (
            TextAnchor::BottomLeft,
            "BottomLeft",
            Vec3::new(-4.0, -4.0, 0.0),
        ),
        (
            TextAnchor::BottomCenter,
            "BottomCenter",
            Vec3::new(0.0, -4.0, 0.0),
        ),
        (
            TextAnchor::BottomRight,
            "BottomRight",
            Vec3::new(4.0, -4.0, 0.0),
        ),
    ];

    let pivot_mesh = meshes.add(Sphere::new(0.1));
    let pivot_mat = MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // Red pivot points
        unlit: true,
        ..default()
    }));

    for (anchor, label, pos) in anchors {
        // Spawn text
        commands.spawn((
            TextMesh {
                text: format!("{}\n(Pivot)", label),
                font: font.clone(),
                style: TextMeshStyle {
                    depth: 0.2,
                    subdivision: 30,
                    anchor,
                    ..default()
                },
            },
            mat.clone(),
            Transform::from_translation(pos),
        ));

        // Spawn a red sphere at the actual Transform position (the pivot)
        // The text should rotate/position around this red dot based on its anchor
        commands.spawn((
            Mesh3d(pivot_mesh.clone()),
            pivot_mat.clone(),
            Transform::from_translation(pos),
        ));
    }
}

fn rotate_anchors(time: Res<Time>, mut query: Query<&mut Transform, With<TextMesh>>) {
    for mut transform in query.iter_mut() {
        // Rotate around local Z to show the pivot point in action
        transform.rotate_z(time.delta_secs() * 0.5);
    }
}
