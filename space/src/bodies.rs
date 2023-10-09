use crate::config::*;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use rand::*;

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_transform.after(update_velocity),
                update_acceleration,
                update_velocity.after(update_acceleration),
            ),
        )
        .insert_resource(SimulationSpeed(1.));
    }
}

#[derive(Resource)]
pub struct SimulationSpeed(pub f32);

pub fn new_body(
    mass: f32,
    density: f32,
    position: Vec2,
    velocity: Vec2,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> BodyBundle {
    let radius = calculate_radius(mass, density);
    BodyBundle {
        object_bundle: ObjectBundle {
            mass: Mass(mass),
            velocity: Velocity(velocity),
            ..default()
        },
        material_bundle: MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(radius).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.)),
            ..default()
        },
        radius: Radius(radius),
    }
}

fn calculate_radius(mass: f32, density: f32) -> f32 {
    let volume = mass / density;
    let radius = (3.0 * volume / (4.0 * std::f32::consts::PI)).powf(1.0 / 3.0);
    radius
}

pub fn new_orbiting_body(
    orbitee_mass: &f32,
    orbitee_position: &Vec2,
    orbitee_velocity: &Vec2,
    mass: f32,
    density: f32,
    speed: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> BodyBundle {
    //let radius = radius + orbitee_radius; // radius from outside of circle, for easier mental model
    //let speed = ((GRAVITY * orbitee_mass) / radius * TIME_STEP).powf(1. / 2.);

    let radius = calculate_radius_off_speed(orbitee_mass, &speed) * SIMULATION_CONSTANT;
    let angle: f32 = random::<f32>() * 6.28319;
    let angle_rotated: f32 = angle + 1.5715 + (random::<f32>() * 2.0).floor() * 3.14159;
    let position = Vec2::new(radius * angle.cos(), radius * angle.sin()) + *orbitee_position;
    let velocity =
        (Vec2::new(angle_rotated.cos(), angle_rotated.sin()) * speed) + *orbitee_velocity;

    println!(
        "Spawning object with orbit radius: {}, mass: {}, speed: {}, position: {:?}, velocity: {:?}, orbitee_velocity: {:?}",
        radius, mass, speed, position, velocity, orbitee_velocity
    );

    new_body(mass, density, position, velocity, meshes, materials)
}

fn calculate_radius_off_speed(mass_central: &f32, speed: &f32) -> f32 {
    let radius = GRAVITY * mass_central / (speed * speed);
    radius
}

pub fn update_transform(
    time: Res<Time>,
    sim_speed: Res<SimulationSpeed>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    let dt_sq = time.delta_seconds_f64() * sim_speed.0 as f64 / SIMULATION_CONSTANT as f64;
    for (mut transform, velocity) in &mut query {
        transform.translation.x += (velocity.0.x as f64 * dt_sq) as f32;
        transform.translation.y += (velocity.0.y as f64 * dt_sq) as f32;
    }
}

fn update_velocity(mut query: Query<(&mut Acceleration, &mut Velocity), Without<StaticObject>>) {
    for (mut acceleration, mut velocity) in &mut query {
        velocity.0 += acceleration.0;
        acceleration.0 = Vec2::ZERO;
    }
}

fn update_acceleration(
    mut query1: Query<(&Transform, &mut Acceleration, &Mass)>,
    //query2: Query<(&Transform, &Velocity, &Mass, Entity), With<Object>>,
) {
    let mut combinations = query1.iter_combinations_mut();
    while let Some(
        [(transform1, mut acceleration1, mass1), (transform2, mut acceleration2, mass2)],
    ) = combinations.fetch_next()
    {
        let distance = transform2.translation - transform1.translation;
        let distance_mult = distance.length().powi(3);
        let force_unit_mass = (GRAVITY * distance) / distance_mult;
        //*DELTA_TIME;
        //let direction = (transform1.translation - transform2.translation).normalize();

        acceleration1.0.x += force_unit_mass.x * mass2.0;
        acceleration1.0.y += force_unit_mass.y * mass2.0;
        acceleration2.0.x -= force_unit_mass.x * mass1.0;
        acceleration2.0.y -= force_unit_mass.y * mass1.0;
    }
}

#[derive(Component)]
pub struct Object;

#[derive(Component)]
pub struct StaticObject {}

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Acceleration(pub Vec2);

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration(Vec2::ZERO)
    }
}

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Bundle)]
pub struct ObjectBundle {
    pub mass: Mass,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

impl Default for ObjectBundle {
    fn default() -> Self {
        Self {
            mass: Mass(1.),
            velocity: Velocity(Vec2::ZERO),
            acceleration: Acceleration::default(),
        }
    }
}

#[derive(Bundle)]
pub struct BodyBundle {
    pub object_bundle: ObjectBundle,
    pub material_bundle: MaterialMesh2dBundle<ColorMaterial>,
    pub radius: Radius,
}
