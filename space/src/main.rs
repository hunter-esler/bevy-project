mod bodies;
mod config;

use crate::bodies::*;
use crate::config::*;

use bevy::text::Text2dBounds;
use bevy::window::WindowTheme;
use bevy_pancam::{PanCam, PanCamPlugin};

use std::time::Duration;

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use bevy::window::PrimaryWindow;

use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am a window!".into(),
                resolution: (800., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                //present_mode: PresentMode::Immediate,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                resizable: false,
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 3 frames.
                // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                ..default()
            }),
            ..default()
        }))
        //.add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin)
        .add_plugins(BodyPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // Configure how frequently our gameplay systems are run
        .insert_resource(FixedTime::new_from_secs(DELTA_TIME))
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            print_frames.run_if(on_timer(Duration::from_secs(FPS_CHECK_INTERVAL.into()))),
        )
        .add_systems(FixedUpdate, test)
        //.add_systems(FixedUpdate, zoom_2d)
        .run();
}

// impl<T: Material2d> Object<T> {
//     fn new(mut meshes: &Res<Assets<Mesh>>, mut materials: &Res<Assets<ColorMaterial>>) -> Self {
//         Self {
//             mesh: MaterialMesh2dBundle {
//                 mesh: meshes.add(shape::Circle::new(50.).into()).into(),
//                 material: materials.add(ColorMaterial::from(Color::PURPLE)),
//                 transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
//                 ..default()
//             },
//         }
//     }
// }

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<(&Window, With<PrimaryWindow>)>,
    asset_server: Res<AssetServer>,
) {
    let window = window.get_single().unwrap().0;
    println!("Setting up");
    //commands.spawn(Object::new(&meshes, &materials));

    // commands.insert_resource(WindowResizeConstraints {
    //     min_width: 800,
    //     min_height: 600,
    //     max_width: 800,
    //     max_height: 600,
    //     ..default()
    // });
    commands.insert_resource(FPS::default());

    let speed = 25000.0;
    let window_width = window.width();
    let window_height = window.height();

    // commands.spawn((
    //     ObjectBundle {
    //         mass: Mass(5000.),
    //         velocity: Velocity(Vec2::new(0., 0.)),
    //         acceleration: Acceleration(Vec2::new(0., 0.)),
    //     },
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(shape::Circle::new(5.51 * 5000.).into()).into(),
    //         material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //         transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    //         ..default()
    //     },
    //     StaticObject {},
    // ));

    let body_bundle = new_body(
        20000000000.,
        1.,
        Vec2::new(0., 0.),
        Vec2::new(5000., 0.),
        meshes.as_mut(),
        materials.as_mut(),
    );
    let orbitee_radius = body_bundle.radius.0;
    let orbitee_position = body_bundle.material_bundle.transform.translation;
    let orbitee_mass = body_bundle.object_bundle.mass.0;
    let orbitee_velocity = body_bundle.object_bundle.velocity.0;
    println!("orbitee radius: {}", orbitee_radius);

    let camera_bundle = Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        projection: OrthographicProjection {
            scale: orbitee_radius * 10.0 / window_height,
            far: 1000.,
            near: -1000.,
            ..default()
        },
        ..default()
    };

    commands.spawn(camera_bundle).insert(PanCam::default());

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 20.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::Center;
    let name = Name::new("Sun");
    let name_str = name.clone().to_string();

    let node_bundle = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },

        ..default()
    };

    let text_bundle = TextBundle {
        text: Text::from_section(name_str, text_style.clone()).with_alignment(text_alignment),
        style: Style {
            //align_self: AlignSelf::Center,
            ..default()
        },
        ..default()
    };

    let parent = commands.spawn((body_bundle, name, FollowText)).id();

    let follow_entity = FollowEntity(parent);
    commands
        .spawn((follow_entity, node_bundle))
        .with_children(|parent| {
            parent.spawn(text_bundle);
        });

    //spawn objects with random positions and velocities
    for i in 0..100 {
        let radius: f32 = random::<f32>() * 1000000. + orbitee_radius;
        let density: f32 = 1.;
        let mass: f32 = random::<f32>() * 1000. + 1000.; //20000. + 1000000.;

        let name = Name::new("Object".to_string() + &i.to_string());
        let name_str = name.clone().to_string();

        println!("Spawning object");
        commands
            .spawn(new_orbiting_body(
                orbitee_radius,
                orbitee_mass,
                orbitee_velocity,
                Vec2::new(orbitee_position.x, orbitee_position.y),
                mass,
                density,
                radius,
                meshes.as_mut(),
                materials.as_mut(),
            ))
            .with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(name_str, text_style.clone())
                        .with_alignment(text_alignment),
                    transform: Transform::from_xyz(0., 100., 0.),
                    ..default()
                });
            });
    }
}

fn test(
    mut query: Query<(&Transform, &mut Style, &FollowEntity), Without<FollowText>>,
    obj_query: Query<&Transform, (With<FollowText>)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for (camera, global_transform) in &camera_query {
        println!("camera");
        for (transform, mut style, follow_entity) in &mut query {
            println!("query!");
            let entity = obj_query.get(follow_entity.0);

            match entity {
                Ok(entity_transform) => {
                    if let Some(position) =
                        camera.world_to_viewport(global_transform, entity_transform.translation)
                    {
                        //set style position centered on entity posititon
                        style.position_type = PositionType::Absolute;
                        style.left = bevy::ui::Val::Px(position.x);
                        style.top = bevy::ui::Val::Px(position.y);
                        println!("width: {:?}, height: {:?}", style.width, style.height);
                    }
                }
                Err(_) => {
                    continue;
                }
            }
            // println!(
            //     "test: {:?}",
            //     camera.world_to_viewport(global_transform, transform.translation)
            // );
        }
    }
}

#[derive(Component)]
struct FollowEntity(Entity);

#[derive(Component)]
struct FollowText;

// fn zoom_2d(mut query: Query<&mut OrthographicProjection, With<Camera2d>>) {
//     for mut projection in &mut query {
//         projection.scale += 0.01;
//     }
// }

#[derive(Resource)]
struct FPS {
    last_frame: u32,
}

impl FPS {
    fn get_fps(&mut self, time: Res<FrameCount>) -> u32 {
        let delta_time: u32 = time.0;
        let fps = (delta_time - self.last_frame) / FPS_CHECK_INTERVAL;
        self.last_frame = delta_time;
        fps
    }
}

impl Default for FPS {
    fn default() -> Self {
        Self { last_frame: 0 }
    }
}

fn print_frames(mut fps: ResMut<FPS>, time: Res<FrameCount>) {
    println!("FPS: {}", fps.get_fps(time));
}
