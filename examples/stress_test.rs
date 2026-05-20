use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_fontmesh::{FontMeshPlugin, TextAnchor, TextMesh, TextMeshBundle, TextMeshStyle};
use rand::prelude::*;
use std::time::Duration;

const SPAWN_INTERVAL_MS: u64 = 200;
const UPDATE_INTERVAL_MS: u64 = 0; // Every frame if 0

#[derive(Resource)]
struct SceneState {
    font: Handle<bevy::text::Font>,
    text_count: usize,
}

#[derive(Resource)]
struct StressTimer {
    spawn_timer: Timer,
    update_timer: Timer,
}

#[derive(Component)]
struct StressText;

#[derive(Component)]
struct StatsText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Camera
    commands
        .spawn(Camera3d::default())
        .insert(Transform::from_xyz(0.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y));

    // Light
    commands
        .spawn(PointLight {
            intensity: 2000.0,
            ..default()
        })
        .insert(Transform::from_xyz(4.0, 8.0, 4.0));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.95, 1.0))), // Light cool white floor
        Transform::default(),
    ));

    // Shared resources
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.insert_resource(SceneState {
        font: font.clone(),
        text_count: 0,
    });

    // Stats Text
    commands
        .spawn(TextMeshBundle {
            text_mesh: TextMesh {
                text: "FPS: 0\nCount: 0".to_string(),
                font: font.clone(),
                style: TextMeshStyle {
                    depth: 0.1,
                    subdivision: 4,
                    anchor: TextAnchor::TopLeft,
                    ..default()
                },
            },
            material: MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::BLACK, // Black FPS text
                unlit: true,              // Make stats text unlit for better visibility
                double_sided: true,
                cull_mode: None,
                ..default()
            })),
            transform: Transform::from_xyz(-10.0, 10.0, 0.0).with_scale(Vec3::splat(1.0)), // Use scale for size
            ..default()
        })
        .insert(StatsText);
}

fn spawn_stress_text(
    mut commands: Commands,
    mut state: ResMut<SceneState>,
    time: Res<Time>,
    mut timer: ResMut<StressTimer>,
    mut materials: ResMut<Assets<StandardMaterial>>, // Need to add materials here
) {
    timer.spawn_timer.tick(time.delta());
    if timer.spawn_timer.just_finished() {
        let mut rng = rand::rng();

        // Spawn a batch
        for _ in 0..5 {
            let pos = Vec3::new(
                (rng.random::<f32>() - 0.5) * 20.0, // Increased spawn area
                rng.random::<f32>() * 8.0,          // Increased spawn height
                (rng.random::<f32>() - 0.5) * 20.0, // Increased spawn area
            );

            let scale = 0.5 + rng.random::<f32>() * 0.5;

            // Random color for each spawned text
            let random_color = Color::srgb(rng.random(), rng.random(), rng.random());
            let text_material = MeshMaterial3d(materials.add(StandardMaterial {
                base_color: random_color,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            }));

            commands
                .spawn(TextMeshBundle {
                    text_mesh: TextMesh {
                        text: format!("{:.1}", time.elapsed_secs()),
                        font: state.font.clone(),
                        style: TextMeshStyle {
                            depth: 0.2,
                            subdivision: 20, // Higher detail for better look
                            anchor: TextAnchor::Center,
                            ..default()
                        },
                    },
                    material: text_material, // Use generated random material
                    transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
                    ..default()
                })
                .insert(StressText);

            state.text_count += 1;
        }
    }
}

fn update_stress_text(
    mut query: Query<&mut TextMesh, With<StressText>>,
    time: Res<Time>,
    mut timer: ResMut<StressTimer>,
) {
    timer.update_timer.tick(time.delta());
    if timer.update_timer.just_finished() {
        // Update ALL text meshes every frame to stress the generator
        let time_str = format!("{:.1}", time.elapsed_secs());
        for mut text_mesh in query.iter_mut() {
            if text_mesh.text != time_str {
                text_mesh.text = time_str.clone();
            }
        }
    }
}

fn update_stats(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextMesh, With<StatsText>>,
    state: Res<SceneState>,
    camera_query: Query<&Transform, (With<Camera>, Without<StatsText>)>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    for mut text_mesh in query.iter_mut() {
        text_mesh.text = format!("FPS: {:.0}\nCount: {}", fps, state.text_count);

        // Billboarding stats
        if let Some(_cam_transform) = camera_query.iter().next() {
            // Just face camera roughly? Or strict billboard?
            // Actually, StatsText is in world space.
        }
    }
}

fn rotate_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let angle = time.elapsed_secs() * 0.2;
        let radius = 25.0; // Increased radius
        transform.translation.x = angle.cos() * radius;
        transform.translation.z = angle.sin() * radius;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::srgb(1.0, 0.7, 0.8))) // Pink background
        .insert_resource(StressTimer {
            spawn_timer: Timer::new(
                Duration::from_millis(SPAWN_INTERVAL_MS),
                TimerMode::Repeating,
            ),
            update_timer: Timer::new(
                Duration::from_millis(UPDATE_INTERVAL_MS),
                TimerMode::Repeating,
            ),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_stress_text,
                update_stress_text,
                update_stats,
                rotate_camera,
            ),
        )
        .run();
}
