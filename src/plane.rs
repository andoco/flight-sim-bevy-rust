use core::f32;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enterpolation::{linear::Linear, Curve};
use leafwing_input_manager::{
    prelude::{ActionState, DualAxis, InputManagerPlugin, InputMap},
    Actionlike, InputManagerBundle,
};

use crate::{camera, world::BlockPos};

pub struct PlanePlugin;

impl Plugin for PlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlaneAction>::default())
            .add_startup_system(setup_plane)
            .add_system(add_plane_input)
            .add_systems(
                (
                    handle_keyboard_input,
                    compute_flight_dynamics,
                    compute_lookup_table_flight_dynamics,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct Plane;

#[derive(Component)]
pub struct PlaneLimits {
    pub thrust: f32,
}

#[derive(Component)]
pub struct EquationFlightDynamics;

#[derive(Component)]
pub struct LookupTableFlightDynamics;

#[derive(Component, Default)]
pub struct PlaneFlight {
    pub thrust: f32,
    pub angle_of_attack: f32,
    pub airspeed: f32,
    pub lift: f32,
    pub weight: f32,
    pub drag: f32,
}

fn setup_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Plane,
            LookupTableFlightDynamics,
            PlaneLimits { thrust: 150.0 },
            PlaneFlight::default(),
            SpatialBundle::from_transform(Transform::from_xyz(10., 1.1, 0.)),
            RigidBody::Dynamic,
            Velocity::zero(),
            ExternalForce::default(),
            ReadMassProperties::default(),
            camera::Follow,
            BlockPos(0, 0),
        ))
        .with_children(|parent| {
            // fuselage
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 2.0, 10.0))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    ..default()
                },
                Friction::new(0.01),
                Collider::cuboid(1.0, 1.0, 5.0),
            ));

            // wing
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(15.0, 0.2, 2.0))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(0., 1.0, 0.),
                    ..default()
                },
                Collider::cuboid(7.5, 0.1, 1.0),
            ));
        });
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlaneAction {
    RollLeft,
    RollRight,
    PitchUp,
    PitchDown,
    ThrustUp,
    ThrustDown,
    PitchRoll,
}

fn add_plane_input(mut commands: Commands, query: Query<Entity, Added<Plane>>) {
    if let Some(entity) = query.get_single().ok() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<PlaneAction> {
                action_state: ActionState::default(),
                input_map: InputMap::default()
                    .insert(KeyCode::Up, PlaneAction::PitchUp)
                    .insert(KeyCode::Down, PlaneAction::PitchDown)
                    .insert(KeyCode::Left, PlaneAction::RollLeft)
                    .insert(KeyCode::Right, PlaneAction::RollRight)
                    .insert(KeyCode::A, PlaneAction::ThrustUp)
                    .insert(KeyCode::Z, PlaneAction::ThrustDown)
                    .insert(DualAxis::left_stick(), PlaneAction::PitchRoll)
                    .build(),
            });
    }
}

fn handle_keyboard_input(
    mut query: Query<
        (
            &ActionState<PlaneAction>,
            &GlobalTransform,
            &PlaneLimits,
            &mut ExternalForce,
            &mut PlaneFlight,
        ),
        With<Plane>,
    >,
    time: Res<Time>,
) {
    let Ok((action_state, global_tx, limits, mut external_force, mut flight)) = query.get_single_mut() else {
        return
    };

    if action_state.just_released(PlaneAction::PitchUp)
        || action_state.just_released(PlaneAction::PitchDown)
        || action_state.just_released(PlaneAction::RollLeft)
        || action_state.just_released(PlaneAction::RollRight)
    {
        info!("Clearing torque");
        external_force.torque = Vec3::ZERO;
    }

    if action_state.just_released(PlaneAction::ThrustUp)
        || action_state.just_released(PlaneAction::ThrustDown)
    {
        info!("Clearing force");
        external_force.force = Vec3::ZERO;
    }

    if action_state.pressed(PlaneAction::RollLeft) {
        external_force.torque = global_tx.forward() * -100.;
    }
    if action_state.pressed(PlaneAction::RollRight) {
        external_force.torque = global_tx.forward() * 100.;
    }
    if action_state.pressed(PlaneAction::PitchUp) {
        external_force.torque = global_tx.right() * -100.;
    }
    if action_state.pressed(PlaneAction::PitchDown) {
        external_force.torque = global_tx.right() * 100.;
    }
    if action_state.pressed(PlaneAction::ThrustUp) {
        flight.thrust += 10.0 * time.delta_seconds();
    }
    if action_state.pressed(PlaneAction::ThrustDown) {
        flight.thrust -= 10.0 * time.delta_seconds();
    }

    flight.thrust = flight.thrust.clamp(0., limits.thrust);
}

fn compute_flight_dynamics(
    mut query: Query<
        (
            &GlobalTransform,
            &Velocity,
            &ReadMassProperties,
            &mut PlaneFlight,
            &mut ExternalForce,
        ),
        With<EquationFlightDynamics>,
    >,
    rapier_config: Res<RapierConfiguration>,
) {
    for (global_tx, velocity, ReadMassProperties(mass_props), mut flight, mut external_force) in
        query.iter_mut()
    {
        let local_velocity = (global_tx.translation() + velocity.linvel) - global_tx.translation();
        let airspeed = -local_velocity.z;

        // Angle between the chord line of the wing (front edge to back edge) and the velocity
        // of the air flowing over the wing.
        let angle_of_attack = global_tx.forward().angle_between(velocity.linvel);

        let air_density = 1.225; // 1.225 kg/m^3 at sea level
        let dynamic_pressure = 0.5 * air_density * airspeed * airspeed;
        let wing_area = 15.0 * 2.0; // length * width = area m^2

        // Approximate lift coefficients For Cessna 172
        let lift_coefficient = match angle_of_attack.to_degrees() as i32 {
            0..=5 => 0.35,
            6..=10 => 1.2,
            11..=14 => 1.4,
            15..=18 => 1.2,
            _ => 0.0,
        };

        let lift = lift_coefficient * dynamic_pressure * wing_area;

        let drag_coefficient = 0.032; // For Cessna 172 at sea level and 100 knots at 0 degrees angle of attack
        let drag = drag_coefficient * dynamic_pressure * wing_area;

        flight.angle_of_attack = angle_of_attack;
        flight.airspeed = airspeed;
        flight.lift = lift;
        flight.weight = rapier_config.gravity.y.abs() * mass_props.mass;
        flight.drag = drag;

        info!(
            "v={}, aoa={}, l={}, d={}, lv={}, airspeed={}",
            velocity.linvel.length(),
            angle_of_attack.to_degrees(),
            lift,
            drag,
            local_velocity.length(),
            airspeed
        );

        external_force.force = global_tx.forward() * flight.thrust;
        external_force.force += -velocity.linvel.normalize_or_zero() * flight.drag;
        external_force.force += global_tx.up() * flight.lift;
    }
}

fn compute_lookup_table_flight_dynamics(
    mut query: Query<
        (
            &GlobalTransform,
            &Velocity,
            &ReadMassProperties,
            &mut PlaneFlight,
            &mut ExternalForce,
        ),
        With<LookupTableFlightDynamics>,
    >,
    rapier_config: Res<RapierConfiguration>,
) {
    for (global_tx, velocity, ReadMassProperties(mass_props), mut flight, mut external_force) in
        query.iter_mut()
    {
        let local_velocity = (global_tx.translation() + velocity.linvel) - global_tx.translation();
        let airspeed = -local_velocity.z;

        // Angle between the chord line of the wing (front edge to back edge) and the velocity
        // of the air flowing over the wing.
        let angle_of_attack = global_tx.forward().angle_between(velocity.linvel);

        let air_density = 1.225; // 1.225 kg/m^3 at sea level
        let dynamic_pressure = 0.5 * air_density * airspeed * airspeed;
        let wing_area = 15.0 * 2.0; // length * width = area m^2

        let lin = Linear::builder()
            .elements([0.35, 0.7, 0.0])
            .knots([0.0, 0.4, 1.0])
            .build()
            .unwrap();

        let samples: Vec<_> = lin.take(10).collect();

        let lift_coefficient = samples
            .get((angle_of_attack.to_degrees() / 9.0) as usize)
            .unwrap_or(&0.0);

        let weight = rapier_config.gravity.y.abs() * mass_props.mass;

        let lift = (lift_coefficient * dynamic_pressure * wing_area).min(weight * 2.0);

        let drag_coefficient = 0.032; // For Cessna 172 at sea level and 100 knots at 0 degrees angle of attack
        let drag = drag_coefficient * dynamic_pressure * wing_area;

        flight.angle_of_attack = angle_of_attack;
        flight.airspeed = airspeed;
        flight.lift = lift;
        flight.weight = weight;
        flight.drag = drag;

        info!(
            "aoa={}, lc={}",
            angle_of_attack.to_degrees(),
            lift_coefficient,
        );

        external_force.force = global_tx.forward() * flight.thrust;
        external_force.force += -velocity.linvel.normalize_or_zero() * flight.drag;
        external_force.force += global_tx.up() * flight.lift;
    }
}
