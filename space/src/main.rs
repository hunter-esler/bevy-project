//! A simplified implementation of the classic game "Breakout".

use std::time::Duration;

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use bevy::time::common_conditions::on_timer;
use bevy::ui::update;
use bevy::window::PresentMode;
use bevy::window::PrimaryWindow;
use bevy::window::WindowTheme;

use rand::random;

pub const BACKGROUND_COLOR: Color = Color::BLACK;

pub const FPS_CHECK_INTERVAL: u32 = 2;
pub const DELTA_TIME: f32 = 1.0 / 60.;
pub const GRAVITY: f32 = 5.0;

fn main() {
    App::new()
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     primary_window: Some(Window {
        //         title: "I am a window!".into(),
        //         resolution: (800., 600.).into(),
        //         present_mode: PresentMode::Fifo,
        //         position: WindowPosition::Centered(MonitorSelection::Primary),
        //         //present_mode: PresentMode::Immediate,
        //         // Tells wasm to resize the window according to the available canvas
        //         fit_canvas_to_parent: true,
        //         // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
        //         prevent_default_event_handling: false,
        //         window_theme: Some(WindowTheme::Dark),
        //         resizable: false,
        //         // This will spawn an invisible window
        //         // The window will be made visible in the make_visible() system after 3 frames.
        //         // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
        //         ..default()
        //     }),
        //     ..default()
        // }))
        .add_plugins(DefaultPlugins)
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
        .add_systems(
            FixedUpdate,
            (
                update_transform.after(update_velocity),
                update_acceleration,
                update_velocity.after(update_acceleration),
            ),
        )
        //.add_systems(FixedUpdate, zoom_2d)
        .run();
}

#[derive(Bundle)]
struct ObjectBundle {
    mass: Mass,
    velocity: Velocity,
    acceleration: Acceleration,
}

#[derive(Component)]
struct Object {}

#[derive(Component)]
struct StaticObject {}

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

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

fn update_transform(mut query: Query<(&mut Transform, &Velocity, &Mass)>) {
    let dt_sq = DELTA_TIME;
    for (mut transform, velocity, mass) in &mut query {
        transform.translation.x += velocity.0.x * dt_sq;
        transform.translation.y += velocity.0.y * dt_sq;
    }
}

fn update_velocity(mut query: Query<(&mut Acceleration, &mut Velocity), Without<StaticObject>>) {
    for (mut acceleration, mut velocity) in &mut query {
        velocity.0 += acceleration.0;
        acceleration.0 = Vec2::ZERO;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<(&Window, With<PrimaryWindow>)>,
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

    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        // projection: OrthographicProjection {
        //     scale: 1.0,
        //     ..default()
        // },
        ..default()
    });

    let speed = 50.0;
    let half_window_width = window.width() / 2.0;
    let half_window_height = window.height() / 2.0;

    commands.spawn((
        ObjectBundle {
            mass: Mass(5000.),
            velocity: Velocity(Vec2::new(0., 0.)),
            acceleration: Acceleration(Vec2::new(0., 0.)),
        },
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        StaticObject {},
    ));

    //spawn objects with random positions and velocities
    for _ in 0..5 {
        let x_pos = random::<f32>() * half_window_width - half_window_width / 2.0;
        let y_pos = random::<f32>() * half_window_height - half_window_height / 2.0;
        let x_vel = random::<f32>() * 2.0 - 1.0;
        let y_vel = random::<f32>() * 2.0 - 1.0;
        println!(
            "Spawning object at ({}, {}) with Velocity ({}, {})",
            x_pos, y_pos, x_vel, y_vel
        );
        commands.spawn((
            ObjectBundle {
                mass: Mass(50.),
                velocity: Velocity(Vec2::new(x_vel, y_vel).normalize() * speed),
                acceleration: Acceleration(Vec2::new(0., 0.)),
            },
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(Vec3::new(x_pos, y_pos, 0.)),
                ..default()
            },
        ));
    }
}

fn update_acceleration(
    mut query1: Query<(&Transform, &mut Acceleration, &Mass, Entity)>,
    //query2: Query<(&Transform, &Velocity, &Mass, Entity), With<Object>>,
) {
    let mut combinations = query1.iter_combinations_mut();
    while let Some(
        [(transform1, mut acceleration1, mass1, entity1), (transform2, mut acceleration2, mass2, entity2)],
    ) = combinations.fetch_next()
    {
        //update object velocity based on object2 mass and position (gravity formula)
        let delta = transform2.translation - transform1.translation;
        let distance_sq = delta.length_squared();
        let force = GRAVITY / distance_sq;
        let force_unit_mass = delta * force * DELTA_TIME;
        //let direction = (transform1.translation - transform2.translation).normalize();

        acceleration1.0.x += force_unit_mass.x * mass2.0;
        acceleration1.0.y += force_unit_mass.y * mass2.0;
        acceleration2.0.x -= force_unit_mass.x * mass1.0;
        acceleration2.0.y -= force_unit_mass.y * mass1.0;
    }

    // for (transform, mut velocity1, mass1, entity) in &mut query1 {
    //     let query2 = query2.iter();
    //     for (transform2, velocity2, mass2, entity2) in query2 {
    //         if entity != entity2 {
    //             //update object velocity based on object2 mass and position (gravity formula)
    //             let distance = (transform2.translation - transform.translation).length();
    //             let force = mass1.0 * mass1.0 / (distance * distance);
    //             let direction = (transform.translation - transform2.translation).normalize();
    //             velocity1.0.x = velocity2.0.x + direction.x * force;
    //             velocity1.0.y = velocity2.0.y + direction.y * force;
    //         }
    //     }
    // }
}

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
