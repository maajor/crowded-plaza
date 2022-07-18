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
use bevy_spatial::{KDTreeAccess2D, KDTreePlugin2D, SpatialAccess};
use rand::{thread_rng, Rng};
use std::{collections::HashMap, f32::consts::PI};
// https://crowdedcity.io/

#[derive(Component)]
struct Actor {
    faction: i32,
}

#[derive(Component)]
struct Pawn;
#[derive(Component)]
struct PlayerController;

#[derive(Component)]
struct OpponentController {
    direction: Vec3,
}

const STEP: f32 = 0.03;
const OPPONENT_MOVE_SCALE: f32 = 1.0;

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
        .add_plugin(KDTreePlugin2D::<Actor> { ..default() })
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

type ActorSpace = KDTreeAccess2D<Actor>; // type alias for later

#[derive(Clone)]
struct FactionMaterialHandles {
    faction_materials: HashMap<i32, Handle<StandardMaterial>>,
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

// system: PlayerController get input and move
fn player_move(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<PlayerController>>,
) {
    let mut player = player_query.single_mut();
    if keyboard_input.pressed(KeyCode::W) {
        player.translation.x += STEP;
    }
    if keyboard_input.pressed(KeyCode::A) {
        player.translation.y += STEP;
    }
    if keyboard_input.pressed(KeyCode::S) {
        player.translation.x -= STEP;
    }
    if keyboard_input.pressed(KeyCode::D) {
        player.translation.y -= STEP;
    }
}

// system: path finding for Pawn
fn leader_path_finding(mut opponent_query: Query<(&mut Transform, &mut OpponentController)>) {
    let mut rng = thread_rng();
    for mut actor in opponent_query.iter_mut() {
        let change_direction_random = rng.gen_range(0.0..1.0);
        if change_direction_random < 0.01 {
            let mut random_direction =
                Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
            random_direction = random_direction.normalize() * OPPONENT_MOVE_SCALE;
            let new_direction = actor.1.direction + random_direction;
            actor.1.direction = new_direction.normalize();
        };
        actor.0.translation += actor.1.direction * STEP;
    }
}

// system: actor follow and avoid collision
fn actor_follow(
    spatial_query: Res<ActorSpace>,
    mut actor_set: Query<(
        Entity,
        &Transform,
        &mut Actor,
        &mut Handle<StandardMaterial>,
    )>,
    faction_materials: Res<FactionMaterialHandles>,
) {
    // https://github.com/bevyengine/bevy/issues/2495

    let neighbor_threshold = 0.5;
    let mut entity_id_to_faction: HashMap<Entity, i32> = HashMap::new();
    // we cannot borrow actor_set twice as below we have to borrow it when do spatial query
    // so cache entity_id_to_faction in this loop
    for (entity, tr, _, _) in actor_set.iter() {
        let mut faction_to_count: HashMap<i32, i32> = HashMap::new();
        let neighbors = spatial_query.within_distance(tr.translation, neighbor_threshold);
        for (_, neighbor_entity) in neighbors.iter() {
            let (_, _, neighbor_actor, _) = actor_set.get(*neighbor_entity).unwrap();
            if neighbor_actor.faction == -1 {
                continue; // we skip neighbor no faction actor
            }
            if faction_to_count.contains_key(&neighbor_actor.faction) {
                faction_to_count
                    .entry(neighbor_actor.faction)
                    .and_modify(|e| *e += 1);
            } else {
                faction_to_count.insert(neighbor_actor.faction, 1);
            }
        }
        match faction_to_count.drain().max_by(|x, y| x.1.cmp(&y.1)) {
            Some((faction, _)) => {
                entity_id_to_faction.insert(entity, faction);
            }
            None => {}
        }
    }

    for (entity, _, mut actor, mut mat) in actor_set.iter_mut() {
        match entity_id_to_faction.get(&entity) {
            Some(fac) => {
                if *fac >= 0 {
                    actor.faction = *fac;
                    *mat = faction_materials
                        .faction_materials
                        .get(&fac)
                        .unwrap()
                        .clone();
                }
            }
            None => {}
        }
    }
}

// https://github.com/bevyengine/bevy/issues/2198
fn update_camera(
    player_query: Query<&Transform, (With<PlayerController>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerController>)>,
) {
    let pl = player_query.single();
    let mut cam = camera_query.single_mut();
    cam.translation = pl.translation + vec3(-5.0, 0.0, 10.0);
    cam.look_at(pl.translation, vec3(0.0, 0.0, 1.0));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let region: f32 = 10.0;
    let actor_count: i32 = 100;
    let opponent_count: i32 = 1;

    let mut faction_to_materials: HashMap<i32, Handle<StandardMaterial>> = HashMap::new();
    for fac in -1..5 {
        faction_to_materials.insert(
            fac,
            materials.add(StandardMaterial {
                base_color: get_color_by_faction(fac),
                perceptual_roughness: 0.8,
                ..default()
            }),
        );
    }
    let handles = FactionMaterialHandles {
        faction_materials: faction_to_materials,
    };

    commands.insert_resource(handles.clone());

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..default()
    });

    // actors
    let mut rng = thread_rng();
    for _ in 0..actor_count {
        let x = rng.gen_range(-region..region);
        let y = rng.gen_range(-region..region);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.1,
                    ..default()
                })),
                material: materials.add(StandardMaterial {
                    base_color: get_color_by_faction(-1),
                    perceptual_roughness: 0.8,
                    ..default()
                }),
                transform: Transform::from_xyz(x, y, 0.0)
                    .with_rotation(Quat::from_rotation_x(PI * 0.5)),
                ..default()
            })
            .insert(Actor { faction: -1 });
    }

    // opponents
    for fac in 1..(opponent_count + 1) {
        let x = rng.gen_range(-region..region);
        let y = rng.gen_range(-region..region);
        let mut direction = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        direction = direction.normalize();
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.1,
                    ..default()
                })),
                material: handles.faction_materials.get(&fac).unwrap().clone(),
                transform: Transform::from_xyz(x, y, 0.0)
                    .with_rotation(Quat::from_rotation_x(PI * 0.5)),
                ..default()
            })
            .insert(Actor { faction: fac })
            .insert(Pawn)
            .insert(OpponentController {
                direction: direction,
            });
    }

    // player
    let x = rng.gen_range(-5.0..5.0);
    let y = rng.gen_range(-5.0..5.0);
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.1,
                ..default()
            })),
            material: handles.faction_materials.get(&0).unwrap().clone(),
            transform: Transform::from_xyz(x, y, 0.0)
                .with_rotation(Quat::from_rotation_x(PI * 0.5)),
            ..default()
        })
        .insert(Actor { faction: 0 })
        .insert(Pawn)
        .insert(PlayerController);

    // ground
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            perceptual_roughness: 0.8,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(PI * 0.5)),
        ..default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 8.0),
        point_light: PointLight {
            intensity: 1600.0,
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
