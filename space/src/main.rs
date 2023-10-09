mod bodies;
mod config;

use crate::bodies::*;
use crate::config::*;

use bevy::window::WindowTheme;
use bevy_pancam::{PanCam, PanCamPlugin};

use std::time::Duration;

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use bevy::window::PrimaryWindow;

use rand::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am a window!".into(),
                //resolution: (800., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                //present_mode: PresentMode::Immediate,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                resizable: true,
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
        .add_systems(Last, (update_follow_text, update_camera_follow))
        .add_systems(Update, update_space)
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

    //let speed = 25000.0;
    //let window_width = window.width();
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
        Vec2::new(10000., 0.),
        meshes.as_mut(),
        materials.as_mut(),
    );
    let orbitee_radius = body_bundle.radius.0;
    let orbitee_position = body_bundle.material_bundle.transform.translation;
    let orbitee_position = Vec2::new(orbitee_position.x, orbitee_position.y);
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

    commands.spawn(camera_bundle).insert(PanCam {
        grab_buttons: [].to_vec(),
        ..default()
    });

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 11.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::Center;
    let name = Name::new("Sun");
    let name_str = name.clone().to_string();
    let style = Style {
        //position_type: PositionType::Absolute,
        width: Val::Px(32.0),
        height: Val::Px(32.0),
        align_content: AlignContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };

    let node_bundle = NodeBundle {
        style: style.clone(),
        ..default()
    };

    let text_bundle = TextBundle {
        text: Text::from_section(name_str, text_style.clone()).with_alignment(text_alignment),
        ..default()
    };

    let parent = commands.spawn((body_bundle, name, FollowText)).id();

    commands.insert_resource(CameraFollowEntity(parent.clone()));

    let follow_entity = FollowEntity(parent);
    commands
        .spawn((follow_entity, node_bundle))
        .with_children(|parent| {
            parent.spawn(text_bundle);
        });

    let mut yeet = |density: f32,
                    mass: f32,
                    speed: f32,
                    name: String,
                    orbitee_mass: &f32,
                    orbitee_position: &Vec2,
                    orbitee_velocity: &Vec2| {
        //let radius: f32 = random::<f32>() * 1000000. + orbitee_radius;

        let name = Name::new(name);
        let name_str = name.clone().to_string();

        println!("Spawning object");
        let body_bundle = new_orbiting_body(
            &orbitee_mass,
            &orbitee_position,
            &orbitee_velocity,
            mass,
            density,
            speed,
            meshes.as_mut(),
            materials.as_mut(),
        );

        //let orbitee_radius = body_bundle.radius.0;
        let orbitee_position = body_bundle.material_bundle.transform.translation;
        let orbitee_position = Vec2::new(orbitee_position.x, orbitee_position.y);
        let orbitee_mass = body_bundle.object_bundle.mass.0;
        let orbitee_velocity = body_bundle.object_bundle.velocity.0;

        let node_bundle = NodeBundle {
            style: style.clone(),
            ..default()
        };

        let text_bundle = TextBundle {
            text: Text::from_section(name_str, text_style.clone()).with_alignment(text_alignment),
            ..default()
        };

        let parent = commands.spawn((body_bundle, name, FollowText)).id();

        let follow_entity = FollowEntity(parent);
        commands
            .spawn((follow_entity, node_bundle))
            .with_children(|parent| {
                parent.spawn(text_bundle);
            });
        (orbitee_position, orbitee_mass, orbitee_velocity)
    };

    //spawn objects with random positions and velocities
    for i in 0..10 {
        let density: f32 = 1.;
        let mass: f32 = random::<f32>() * 10000. + 1000.; //20000. + 1000000.;
        let speed: f32 = random::<f32>() * 10000. + 1000.;

        let (orbitee_position, orbitee_mass, orbitee_velocity) = yeet(
            density,
            mass,
            speed,
            "Object".to_string() + &i.to_string(),
            &orbitee_mass,
            &orbitee_position,
            &orbitee_velocity,
        );

        yeet(
            1.,
            100.,
            100.,
            "Object baby ".to_string() + &i.to_string(),
            &orbitee_mass,
            &orbitee_position,
            &orbitee_velocity,
        );
    }
}

fn update_camera_follow(
    following_entity: Res<CameraFollowEntity>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    obj_query: Query<&Transform, Without<Camera2d>>,
) {
    for mut camera_transform in &mut query {
        let entity = obj_query.get(following_entity.0);
        match entity {
            Ok(entity_transform) => {
                camera_transform.translation = entity_transform.translation;
            }
            _ => {
                println!("No entity following!");
            }
        }
    }
}

fn update_follow_text(
    mut query: Query<(&mut Style, &FollowEntity), Without<FollowText>>,
    obj_query: Query<(&Transform, &Radius, &Name), With<FollowText>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for (camera, global_transform) in &camera_query {
        //println!("camera");
        for (mut style, follow_entity) in &mut query {
            //println!("query!");
            let entity = obj_query.get(follow_entity.0);

            match entity {
                Ok((entity_transform, entity_radius, entity_name)) => {
                    if let Some(position) =
                        camera.world_to_viewport(global_transform, entity_transform.translation)
                    {
                        println!("name: {:?}", entity_name);
                        //set style position centered on entity posititon
                        style.position_type = PositionType::Absolute;
                        let width = match style.width {
                            Val::Px(w) => w,
                            _ => 0.,
                        };
                        let height = match style.height {
                            Val::Px(h) => h,
                            _ => 0.,
                        };
                        style.left = bevy::ui::Val::Px(position.x - width / 2.);
                        style.top = bevy::ui::Val::Px(position.y - height / 2. - entity_radius.0);
                        //println!("width: {:?}, height: {:?}", style.width, style.height);
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

#[derive(Resource)]
struct CameraFollowEntity(Entity);

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

//on pressing space, pick a random object to follow
fn update_space(
    mut camera_follow: ResMut<CameraFollowEntity>,
    keyboard: Res<Input<KeyCode>>,
    query: Query<Entity, With<FollowText>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let mut query_iter = query.iter();
        let len = query_iter.len();
        let mut rng = rand::thread_rng();
        let entity_index = rng.gen_range(0..len);
        let entity = query_iter.nth(entity_index).unwrap();
        camera_follow.0 = entity;
    }
}
