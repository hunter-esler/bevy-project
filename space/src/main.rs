//! A simplified implementation of the classic game "Breakout".

use std::time::Duration;

use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use bevy::time::common_conditions::on_timer;
use bevy::window::PresentMode;
use bevy::window::WindowTheme;

use rand::random;

pub const BACKGROUND_COLOR: Color = Color::BLACK;

pub const FPS_CHECK_INTERVAL: u32 = 2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am a window!".into(),
                resolution: (800., 600.).into(),
                present_mode: PresentMode::AutoVsync,
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
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // Configure how frequently our gameplay systems are run
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            print_frames.run_if(on_timer(Duration::from_secs(FPS_CHECK_INTERVAL.into()))),
        )
        .add_systems(Update, update_transform)
        //.add_systems(FixedUpdate, zoom_2d)
        .run();
}

#[derive(Bundle)]
struct Object<T: Material2d> {
    mesh: MaterialMesh2dBundle<T>,
    mass: Mass,
    velocity: Velocity,
}

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Velocity(Vec2);

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

fn update_transform(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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

    commands.spawn(Object {
        mesh: MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        mass: Mass(50.),
        velocity: Velocity(Vec2::new(10., 0.)),
    });

    let speed = 10.0;
    //spawn objects with random positions and velocities
    for i in 0..2 {
        commands.spawn(Object {
            mesh: MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..default()
            },
            mass: Mass(50.),
            velocity: Velocity(Vec2::new(random::<f32>() * speed, random::<f32>() * speed)),
        });
    }
}

fn zoom_2d(mut query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    for mut projection in &mut query {
        projection.scale += 0.01;
    }
}

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
