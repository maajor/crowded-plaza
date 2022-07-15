use bevy::{
    asset::AssetPlugin,
    core_pipeline::CorePipelinePlugin,
    input::InputPlugin,
    math::vec3,
    pbr::PbrPlugin,
    prelude::*,
    render::{camera::Camera3d, RenderPlugin},
    window::WindowPlugin,
    winit::WinitPlugin,
};
use rand::{thread_rng, Rng};
// https://crowdedcity.io/

#[derive(Component)]
struct Actor {
    faction: i32,
}

#[derive(Component)]
struct Pawn;

#[derive(Component)]
struct Leader;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(TransformPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(WinitPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(PbrPlugin::default())
        .insert_resource(AmbientLight {
            brightness: 0.03,
            ..default()
        })
        .add_startup_system(setup)
        .add_system(leader_path_finding)
        .add_system(player_move)
        .add_system(actor_follow)
        .add_system(update_camera)
        .run();
}

fn get_color_by_faction(faction: i32) -> Color {
    match faction {
        -1 => Color::WHITE,
        0 => Color::RED,
        1 => Color::AZURE,
        2 => Color::BEIGE,
        3 => Color::GOLD,
        4 => Color::GREEN,
        5 => Color::CYAN,
        _ => Color::BLACK,
    }
}

// system: pawn get input and move
fn player_move(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Actor, &mut Transform), With<Pawn>>,
) {
    let mut player = query.single_mut();
    if keyboard_input.pressed(KeyCode::W) {
        player.1.translation.x += 1.0;
    } else if keyboard_input.just_pressed(KeyCode::A) {
        player.1.translation.z += 1.0;
    } else if keyboard_input.just_pressed(KeyCode::S) {
        player.1.translation.x -= 1.0;
    } else if keyboard_input.just_pressed(KeyCode::D) {
        player.1.translation.z -= 1.0;
    }
}

// system: path finding for leader
fn leader_path_finding(query: Query<&mut Actor, With<Leader>>) {
    for actor in query.iter() {
        // todo!();
    }
}

// system: actor follow and avoid collision
fn actor_follow(query: Query<&mut Actor, (Without<Pawn>, Without<Leader>)>) {
    for actor in query.iter() {
        // todo!();
    }
}

// https://github.com/bevyengine/bevy/issues/2198
fn update_camera(
    player: Query<&Transform, (With<Pawn>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Pawn>)>,
) {
    let pl = player.single();
    let mut cam = camera.single_mut();
    cam.translation = pl.translation + vec3(5.0, 10.0, 0.0);
    cam.look_at(pl.translation, vec3(1.0, 0.0, 0.0));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..default()
    });

    // actors
    let mut rng = thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(-5.0..5.0);
        let z = rng.gen_range(-5.0..5.0);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
                material: materials.add(StandardMaterial {
                    base_color: get_color_by_faction(-1),
                    ..default()
                }),
                transform: Transform::from_xyz(x, 0.0, z),
                ..default()
            })
            .insert(Actor { faction: -1 });
    }

    // leaders
    for fac in 1..5 {
        let x = rng.gen_range(-5.0..5.0);
        let z = rng.gen_range(-5.0..5.0);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
                material: materials.add(StandardMaterial {
                    base_color: get_color_by_faction(fac),
                    ..default()
                }),
                transform: Transform::from_xyz(x, 0.0, z),
                ..default()
            })
            .insert(Actor { faction: fac })
            .insert(Leader);
    }

    // player
    let x = rng.gen_range(-5.0..5.0);
    let z = rng.gen_range(-5.0..5.0);
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(StandardMaterial {
                base_color: get_color_by_faction(0), // player's faction
                ..default()
            }),
            transform: Transform::from_xyz(x, 0.0, z),
            ..default()
        })
        .insert(Actor { faction: 0 })
        .insert(Pawn);

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });
}