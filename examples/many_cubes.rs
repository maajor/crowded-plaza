use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(text_update_system)
        .add_system(rotate_cube)
        .run();
}

#[derive(Component)]
struct CubeComponent {}

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    const WIDTH: usize = 200;
    const HEIGHT: usize = 200;
    const DEPTH: usize = 50;
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            // introduce spaces to break any kind of moiré pattern
            if x % 10 == 0 || y % 10 == 0 {
                continue;
            }
            for z in 0..DEPTH {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh.clone_weak(),
                        material: material.clone_weak(),
                        transform: Transform::from_xyz(
                            (x as f32) * 2.5,
                            (y as f32) * 2.5,
                            (z as f32) * 2.5,
                        ),
                        ..default()
                    })
                    .insert(CubeComponent {});
            }
        }
    }
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(WIDTH as f32, HEIGHT as f32, WIDTH as f32),
        ..default()
    });

    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(DirectionalLightBundle { ..default() });

    commands.spawn_bundle(PbrBundle {
        mesh,
        material,
        transform: Transform {
            translation: Vec3::new(0.0, HEIGHT as f32 * 2.5, 0.0),
            scale: Vec3::splat(5.0),
            ..default()
        },
        ..default()
    });

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 40.0,
                            color: Color::RED,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 40.0,
                            color: Color::RED,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(FpsText);
}

fn rotate_cube(mut cube_query: Query<(&CubeComponent, &mut Transform)>) {
    {
        // creates a span and starts the timer
        let _my_span = info_span!("rotate_cube", name = "rotate_cube").entered();
        for (_, mut tr) in cube_query.iter_mut() {
            tr.rotate(Quat::from_rotation_z(0.1))
        }
    }
}

#[derive(Deref, DerefMut)]
struct PrintingTimer(Timer);

impl Default for PrintingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, true))
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}
