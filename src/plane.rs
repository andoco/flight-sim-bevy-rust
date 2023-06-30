use core::f32;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enterpolation::{linear::Linear, Curve};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap, SingleAxis},
    Actionlike, InputManagerBundle,
};

use crate::{
    camera,
    world::{self, BlockPos},
};

pub struct PlanePlugin;

impl Plugin for PlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlaneAction>::default())
            .add_startup_system(setup_plane)
            .add_system(add_plane_input)
            .add_systems(
                (
                    handle_keyboard_input,
                    handle_gamepad_input,
                    compute_flight_dynamics,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct Plane;

#[derive(Component, Clone)]
pub struct PlaneLimits {
    pub thrust: f32,
    pub fuselage: Vec3,
    pub wings: Vec2,
    pub lift_coefficient_samples: Vec<f32>,
}

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
    let lift_coefficient_curve = Linear::builder()
        .elements([0.0, 0.0, 0.35, 1.4, 0.8, 0.0])
        .knots([-90.0, -5.0, 0.0, 10.0, 15.0, 90.0])
        .build()
        .unwrap();

    let lift_coefficient_samples: Vec<_> = lift_coefficient_curve.take(180).collect();

    let limits = PlaneLimits {
        thrust: 150.0,
        fuselage: Vec3::new(2.0, 2.0, 10.0),
        wings: Vec2::new(15.0, 2.0),
        lift_coefficient_samples,
    };

    commands
        .spawn((
            Plane,
            limits.clone(),
            PlaneFlight::default(),
            SpatialBundle::from_transform(Transform::from_xyz(
                world::SPACING as f32 * 0.5,
                1.1,
                0.,
            )),
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
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        limits.fuselage.x,
                        limits.fuselage.y,
                        limits.fuselage.z,
                    ))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    ..default()
                },
                Friction::new(0.01),
                Collider::cuboid(
                    limits.fuselage.x * 0.5,
                    limits.fuselage.y * 0.5,
                    limits.fuselage.z * 0.5,
                ),
            ));

            // wing
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        limits.wings.x,
                        0.2,
                        limits.wings.y,
                    ))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(0., 1.0, 0.),
                    ..default()
                },
                Collider::cuboid(limits.wings.x * 0.5, 0.1, limits.wings.y * 0.5),
            ));
        });
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlaneAction {
    // Keyboard
    RollLeft,
    RollRight,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    ThrustUp,
    ThrustDown,

    // Gamepad
    Pitch,
    Roll,
    Throttle,
    Rudder,
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
                    .insert(KeyCode::Q, PlaneAction::YawLeft)
                    .insert(KeyCode::W, PlaneAction::YawRight)
                    .insert(KeyCode::A, PlaneAction::ThrustUp)
                    .insert(KeyCode::Z, PlaneAction::ThrustDown)
                    .insert(
                        SingleAxis::symmetric(GamepadAxisType::LeftStickY, 0.25),
                        PlaneAction::Pitch,
                    )
                    .insert(
                        SingleAxis::symmetric(GamepadAxisType::LeftStickX, 0.25),
                        PlaneAction::Roll,
                    )
                    .insert(
                        SingleAxis::symmetric(GamepadAxisType::RightStickY, 0.25),
                        PlaneAction::Throttle,
                    )
                    .insert(
                        SingleAxis::symmetric(GamepadAxisType::RightStickX, 0.25),
                        PlaneAction::Rudder,
                    )
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
        || action_state.just_released(PlaneAction::YawLeft)
        || action_state.just_released(PlaneAction::YawRight)
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
    if action_state.pressed(PlaneAction::YawLeft) {
        external_force.torque = global_tx.up() * 10.;
    }
    if action_state.pressed(PlaneAction::YawRight) {
        external_force.torque = global_tx.up() * -10.;
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

fn handle_gamepad_input(
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

    if action_state.just_released(PlaneAction::Pitch)
        || action_state.just_released(PlaneAction::Roll)
        || action_state.just_released(PlaneAction::Rudder)
    {
        info!("Resetting torque");
        external_force.torque = Vec3::ZERO;
    }

    if action_state.pressed(PlaneAction::Pitch) {
        external_force.torque += global_tx.right()
            * -action_state.clamped_value(PlaneAction::Pitch)
            * time.delta_seconds()
            * 10.0;
    }
    if action_state.pressed(PlaneAction::Roll) {
        external_force.torque += global_tx.forward()
            * action_state.clamped_value(PlaneAction::Roll)
            * time.delta_seconds()
            * 20.0;
    }
    if action_state.pressed(PlaneAction::Throttle) {
        flight.thrust +=
            action_state.clamped_value(PlaneAction::Throttle) * time.delta_seconds() * 10.0;
    }
    if action_state.pressed(PlaneAction::Rudder) {
        external_force.torque += global_tx.up()
            * -action_state.clamped_value(PlaneAction::Rudder)
            * time.delta_seconds()
            * 10.0;
    }
}

fn angle_of_attack_signed(global_tx: &GlobalTransform, velocity: Vec3) -> f32 {
    let a1 = Vec3::Y.angle_between(global_tx.forward());
    let a2 = Vec3::Y.angle_between(velocity.normalize());

    a2 - a1
}

fn compute_flight_dynamics(
    mut query: Query<
        (
            &GlobalTransform,
            &Velocity,
            &ReadMassProperties,
            &PlaneLimits,
            &mut PlaneFlight,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
    rapier_config: Res<RapierConfiguration>,
) {
    for (
        global_tx,
        velocity,
        ReadMassProperties(mass_props),
        limits,
        mut flight,
        mut external_force,
    ) in query.iter_mut()
    {
        let local_velocity = (global_tx.translation() + velocity.linvel) - global_tx.translation();
        let airspeed = -local_velocity.z;

        // Angle between the chord line of the wing (front edge to back edge) and the velocity
        // of the air flowing over the wing.
        let angle_of_attack = angle_of_attack_signed(global_tx, velocity.linvel);

        let air_density = 1.225; // 1.225 kg/m^3 at sea level
        let dynamic_pressure = 0.5 * air_density * airspeed * airspeed;
        let wing_area = limits.wings.x * limits.wings.y;

        let lift_coefficient_index = (angle_of_attack.to_degrees() + 90.0) as usize;

        let lift_coefficient = limits
            .lift_coefficient_samples
            .get(lift_coefficient_index)
            .unwrap_or(&0.0);

        let lift = lift_coefficient * dynamic_pressure * wing_area;

        let drag_coefficient = 0.032; // For Cessna 172 at sea level and 100 knots at 0 degrees angle of attack
        let drag = drag_coefficient * dynamic_pressure * wing_area;

        flight.angle_of_attack = angle_of_attack;
        flight.airspeed = airspeed;
        flight.lift = lift;
        flight.weight = rapier_config.gravity.y.abs() * mass_props.mass;
        flight.drag = drag;

        external_force.force = global_tx.forward() * flight.thrust;
        external_force.force += -velocity.linvel.normalize_or_zero() * flight.drag;
        external_force.force += global_tx.up() * flight.lift;
    }
}
