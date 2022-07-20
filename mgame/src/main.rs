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
use rand::{prelude::ThreadRng, thread_rng, Rng};
use std::{collections::HashMap, f32::consts::PI};
// https://crowdedcity.io/

#[derive(Component)]
struct Actor {
    faction: i32,
    velocity: Vec3,
    accleration: Vec3,
}

#[derive(Component)]
struct Pawn {
    alive: bool,
}
#[derive(Component)]
struct PlayerController;

#[derive(Component)]
struct OpponentController;

const OPPONENT_MOVE_SCALE: f32 = 1.0;
const PAWN_SPEED: f32 = 0.02;
const WANDER_SPEED: f32 = 0.005;
const NEIGHBOR_THRESHOLD: f32 = 0.5;
const REPULSION_THRESHOLD: f32 = 0.2;
const REPULSION_FACTOR: f32 = 0.001;
const ALIGN_FACTOR: f32 = 0.01;
const ATTRACT_FACTOR: f32 = 0.0003;

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
        .add_system(move_player_system)
        .add_system(change_direction_actor_system)
        .add_system(change_direction_pawn_system)
        .add_system(move_actor_system)
        .add_system(move_pawn_system)
        .add_system(change_faction_actor_system)
        .add_system(update_camera_lookat_system)
        .add_system(follow_pawn_system)
        .add_system(repulse_actor_system)
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
fn move_player_system(
    mouse_input: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: ResMut<Windows>,
    mut player_query: Query<&mut Actor, With<PlayerController>>,
) {
    let window = windows.primary();
    let width = window.width();
    let height = window.height();
    let window_size = Vec2::new(width, height) / 2.0;
    let mut player = player_query.single_mut();
    if mouse_input.pressed(MouseButton::Left) {
        for event in cursor_moved_events.iter() {
            let move_direction = Vec2::new(
                event.position.y - window_size.y,
                -event.position.x + window_size.x,
            );
            player.velocity = move_direction.normalize().extend(0.0) * PAWN_SPEED;
        }
    }
}

fn random_change_direction(mut actor: &mut Actor, rng: &mut ThreadRng, speed: f32) {
    let change_direction_random = rng.gen_range(0.0..1.0);
    if change_direction_random < 0.01 {
        let mut random_direction =
            Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        random_direction = random_direction.normalize() * OPPONENT_MOVE_SCALE;
        let new_direction = actor.velocity + random_direction;
        actor.velocity = new_direction.normalize() * speed;
    };
}

fn change_direction_actor_system(mut actor_query: Query<&mut Actor, Without<Pawn>>) {
    let mut rng = thread_rng();
    for mut actor in actor_query.iter_mut() {
        if actor.faction != -1 {
            continue;
        }
        random_change_direction(&mut actor, &mut rng, WANDER_SPEED);
    }
}

fn change_direction_pawn_system(
    mut opponent_query: Query<(&mut Actor, &Pawn), With<OpponentController>>,
) {
    let mut rng = thread_rng();
    for (mut actor, pawn) in opponent_query.iter_mut() {
        if !pawn.alive {
            continue;
        }
        random_change_direction(&mut actor, &mut rng, PAWN_SPEED);
    }
}

fn move_actor_system(mut actor_query: Query<(&mut Transform, &mut Actor), Without<Pawn>>) {
    for (mut tr, mut actor) in actor_query.iter_mut() {
        tr.translation += actor.velocity;
        let acc = actor.accleration;
        actor.velocity += acc;
        if actor.velocity.length() > PAWN_SPEED {
            actor.velocity = actor.velocity.normalize() * PAWN_SPEED;
        }
        actor.accleration = acc * 0.5; // damping
    }
}

fn move_pawn_system(mut actor_query: Query<(&mut Transform, &Actor), With<Pawn>>) {
    for (mut tr, actor) in actor_query.iter_mut() {
        tr.translation += actor.velocity;
    }
}

fn follow_pawn_system(
    mut actor_query: Query<(&mut Actor, &Transform), Without<Pawn>>,
    pawn_query: Query<(&Actor, &Pawn, &Transform)>,
) {
    let mut faction_to_velocity: HashMap<i32, Vec3> = HashMap::new();
    let mut faction_to_position: HashMap<i32, Vec3> = HashMap::new();
    for (pawn_actor, pawn, tr) in pawn_query.iter() {
        if pawn.alive {
            faction_to_velocity.insert(pawn_actor.faction, pawn_actor.velocity);
            faction_to_position.insert(pawn_actor.faction, tr.translation);
        }
    }
    for (mut actor, tr) in actor_query.iter_mut() {
        if actor.faction != -1 {
            match faction_to_velocity.get(&actor.faction) {
                Some(v) => {
                    let acc = *v - actor.velocity;
                    actor.accleration += acc * ALIGN_FACTOR;

                    let toward_pawn =
                        *faction_to_position.get(&actor.faction).unwrap() - tr.translation;
                    actor.accleration += toward_pawn * ATTRACT_FACTOR;
                }
                None => {}
            }
        }
    }
}

// follow leader direction
// seperation
// move to leader position

// system: actor follow and avoid collision
fn change_faction_actor_system(
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
    let mut entity_id_to_faction: HashMap<Entity, i32> = HashMap::new();
    // we cannot borrow actor_set twice as below we have to borrow it when do spatial query
    // so cache entity_id_to_faction in this loop
    for (entity, tr, _, _) in actor_set.iter() {
        let mut faction_to_count: HashMap<i32, i32> = HashMap::new();
        let neighbors = spatial_query.within_distance(tr.translation, NEIGHBOR_THRESHOLD);
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
        match faction_to_count
            .drain()
            .max_by(|(_, count1), (_, count2)| count1.cmp(&count2))
        {
            Some((faction, _)) => {
                entity_id_to_faction.insert(entity, faction);
            }
            None => {}
        }
    }

    for (entity, _, mut actor, mut mat) in actor_set.iter_mut() {
        match entity_id_to_faction.get(&entity) {
            Some(fac) => {
                if *fac >= 0 && *fac != actor.faction {
                    actor.faction = *fac;
                    actor.velocity = Vec3::new(0.0, 0.0, 0.0);
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

// system: actor follow and avoid collision
fn repulse_actor_system(
    spatial_query: Res<ActorSpace>,
    mut actor_set: Query<(Entity, &Transform, &mut Actor)>,
) {
    let mut entity_id_to_repulse: HashMap<Entity, Vec3> = HashMap::new();
    for (entity, tr, actor) in actor_set.iter() {
        // neighbor query include self
        for (neighbor_pos, neighbor_entity) in spatial_query.k_nearest_neighbour(tr.translation, 2)
        {
            if neighbor_entity.id() != entity.id() {
                match actor_set.get(neighbor_entity) {
                    Ok((_, _, neighbor_actor)) => {
                        if neighbor_actor.faction == -1 {
                            continue; // we skip neighbor no faction actor
                        }
                        if neighbor_actor.faction == actor.faction
                            && neighbor_pos.distance(tr.translation) < REPULSION_THRESHOLD
                        {
                            entity_id_to_repulse
                                .insert(entity, (neighbor_pos - tr.translation).normalize());
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }

    for (entity, _, mut actor) in actor_set.iter_mut() {
        match entity_id_to_repulse.get(&entity) {
            Some(repul) => {
                actor.accleration -= *repul * REPULSION_FACTOR;
            }
            None => {}
        }
    }
}

// https://github.com/bevyengine/bevy/issues/2198
fn update_camera_lookat_system(
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
        let dir = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
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
            .insert(Actor {
                faction: -1,
                velocity: dir.normalize() * WANDER_SPEED,
                accleration: Vec3::new(0.0, 0.0, 0.0),
            });
    }

    // opponents
    for fac in 1..(opponent_count + 1) {
        let x = rng.gen_range(-region..region);
        let y = rng.gen_range(-region..region);
        let dir = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.12,
                    depth: 1.2,
                    ..default()
                })),
                material: handles.faction_materials.get(&fac).unwrap().clone(),
                transform: Transform::from_xyz(x, y, 0.0)
                    .with_rotation(Quat::from_rotation_x(PI * 0.5)),
                ..default()
            })
            .insert(Actor {
                faction: fac,
                velocity: dir.normalize() * PAWN_SPEED,
                accleration: Vec3::new(0.0, 0.0, 0.0),
            })
            .insert(Pawn { alive: true })
            .insert(OpponentController {});
    }

    // player
    let x = rng.gen_range(-5.0..5.0);
    let y = rng.gen_range(-5.0..5.0);
    let dir = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.12,
                depth: 1.2,
                ..default()
            })),
            material: handles.faction_materials.get(&0).unwrap().clone(),
            transform: Transform::from_xyz(x, y, 0.0)
                .with_rotation(Quat::from_rotation_x(PI * 0.5)),
            ..default()
        })
        .insert(Actor {
            faction: 0,
            velocity: dir.normalize() * PAWN_SPEED,
            accleration: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(Pawn { alive: true })
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

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 5.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            perceptual_roughness: 0.8,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(PI * 0.5)),
        ..default()
    });
}
