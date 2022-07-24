use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(rotate_cube)
        .run();
}

#[derive(Component)]
struct CubeComponent {}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: 1.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.8,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(CubeComponent {});

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 1.0, 10.0),
        ..default()
    });

    commands.spawn_bundle(DirectionalLightBundle { ..default() });
}

fn rotate_cube(mut cube_query: Query<(&CubeComponent, &mut Transform)>) {
    {
        let _my_span = info_span!("rotate_cube", name = "rotate_cube").entered();
        for (_, mut tr) in cube_query.iter_mut() {
            tr.rotate(Quat::from_rotation_y(0.01))
        }
    }
}
