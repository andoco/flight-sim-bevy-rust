use core::f32;
use std::{iter, ops::AddAssign};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enterpolation::{linear::Linear, Curve};
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap, SingleAxis},
    Actionlike, InputManagerBundle,
};

use crate::{
    camera::{self, Follow},
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
                    update_airfoils_rotations,
                    update_airspeed,
                    update_thrust_forces,
                    update_airfoil_forces,
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
    pub wing_offset_z: f32,
    pub lift_coefficient_samples: Vec<f32>,
}

#[derive(Component, Default)]
pub struct PlaneControl {
    pub ailerons: f32,
    pub elevators: f32,
    pub rudder: f32,
}

impl PlaneControl {
    fn clear(&mut self) {
        *self = Default::default();
    }
}

#[derive(Component, Default)]
pub struct Thrust(pub f32);

#[derive(Component, Default)]
pub struct Airspeed(pub f32);

#[derive(Component, Default)]
pub struct Altitude(pub f32);

#[derive(Component, Default)]
pub struct Lift(pub f32);

#[derive(Component, Default)]
pub struct PlaneFlight {
    pub angle_of_attack: f32,
    pub lift: f32,
    pub weight: f32,
    pub drag: f32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AirfoilPosition {
    Wings,
    HorizontalTailLeft,
    HorizontalTailRight,
    VerticalTail,
}

#[derive(Component)]
pub struct Airfoil {
    pub position: AirfoilPosition,
    pub area: f32,
    pub lift_coefficient_samples: Vec<f32>,
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
        fuselage: Vec3::new(1.12, 2.0, 8.3),
        wings: Vec2::new(11.0, 1.5),
        wing_offset_z: -0.0,
        lift_coefficient_samples: lift_coefficient_samples.clone(),
    };

    let tail_wing_lift_coefficient_curve = Linear::builder()
        .elements([-0.25, -0.25, 0.0, 0.25, 0.25])
        .knots([-90.0, -10.0, 0.0, 10.0, 90.0])
        .build()
        .unwrap();

    commands
        .spawn((
            Plane,
            PlaneControl::default(),
            limits.clone(),
            PlaneFlight::default(),
            Thrust(0.0),
            Airspeed::default(),
            Altitude::default(),
            SpatialBundle::from_transform(Transform::from_xyz(
                world::SPACING as f32 * 0.5,
                1.1,
                0.,
            )),
            RigidBody::Dynamic,
            Velocity::zero(),
            ExternalForce::default(),
            ReadMassProperties::default(),
            camera::Follow(camera::FollowKind::Behind),
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
                Airfoil {
                    position: AirfoilPosition::Wings,
                    area: limits.wings.x * limits.wings.y,
                    lift_coefficient_samples: lift_coefficient_samples.clone(),
                },
                Lift::default(),
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        limits.wings.x,
                        0.2,
                        limits.wings.y,
                    ))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(0., 0.0, limits.wing_offset_z),
                    ..default()
                },
                Collider::cuboid(limits.wings.x * 0.5, 0.1, limits.wings.y * 0.5),
            ));

            // vertical tail fin
            let tail_width = 0.2;
            let tail_height = limits.fuselage.y;
            let tail_length = limits.fuselage.y;

            parent.spawn((
                Airfoil {
                    position: AirfoilPosition::VerticalTail,
                    area: tail_height * tail_length,
                    lift_coefficient_samples: iter::repeat(0.0).take(180).collect(),
                },
                Lift::default(),
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        tail_width,
                        tail_height,
                        tail_length,
                    ))),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(
                        0.,
                        limits.fuselage.y * 0.5 + tail_height * 0.5,
                        limits.fuselage.z * 0.5 - tail_length * 0.5,
                    ),
                    ..default()
                },
                // Collider::cuboid(tail_width * 0.5, tail_height * 0.5, tail_length * 0.5),
            ));

            // horizontal tail wings
            for position in [
                AirfoilPosition::HorizontalTailLeft,
                AirfoilPosition::HorizontalTailRight,
            ] {
                // tail
                let tail_width = 1.0;
                let tail_height = 0.1;
                let tail_length = 0.7;

                let offset = match position {
                    AirfoilPosition::HorizontalTailLeft => -1.0,
                    AirfoilPosition::HorizontalTailRight => 1.0,
                    _ => panic!("Not a horizontal tail position"),
                };

                parent.spawn((
                    Airfoil {
                        position,
                        area: tail_width * tail_length,
                        lift_coefficient_samples: tail_wing_lift_coefficient_curve
                            .take(180)
                            .collect(),
                    },
                    Lift::default(),
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(
                            tail_width,
                            tail_height,
                            tail_length,
                        ))),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_xyz(
                            (limits.fuselage.x * 0.5 + tail_width * 0.5) * offset,
                            0.0,
                            limits.fuselage.z * 0.5 - tail_length * 0.5,
                        ),
                        ..default()
                    },
                    // Collider::cuboid(tail_width * 0.5, tail_height * 0.5, tail_length * 0.5),
                ));
            }
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

    // View
    FollowBehind,
    FollowAbove,
    FollowInside,
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
                    .insert(GamepadButtonType::DPadDown, PlaneAction::FollowBehind)
                    .insert(GamepadButtonType::DPadUp, PlaneAction::FollowAbove)
                    .insert(GamepadButtonType::DPadLeft, PlaneAction::FollowInside)
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
            &mut PlaneControl,
            &mut Thrust,
        ),
        With<Plane>,
    >,
    time: Res<Time>,
) {
    let Ok((action_state, global_tx, limits, mut external_force,  mut control, mut thrust)) = query.get_single_mut() else {
        return
    };

    if action_state.just_released(PlaneAction::PitchUp)
        || action_state.just_released(PlaneAction::PitchDown)
        || action_state.just_released(PlaneAction::RollLeft)
        || action_state.just_released(PlaneAction::RollRight)
        || action_state.just_released(PlaneAction::YawLeft)
        || action_state.just_released(PlaneAction::YawRight)
    {
        info!("Clearing torque and controls");
        external_force.torque = Vec3::ZERO;
        control.clear();
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
        control.elevators = 10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::PitchDown) {
        control.elevators = -10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::ThrustUp) {
        thrust.0 += 10.0 * time.delta_seconds();
    }
    if action_state.pressed(PlaneAction::ThrustDown) {
        thrust.0 -= 10.0 * time.delta_seconds();
    }

    control.elevators = control
        .elevators
        .clamp(-45_f32.to_radians(), 45_f32.to_radians());

    thrust.0 = thrust.0.clamp(0., limits.thrust);
}

fn handle_gamepad_input(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ActionState<PlaneAction>,
            &GlobalTransform,
            &PlaneLimits,
            &mut ExternalForce,
            &mut PlaneControl,
            &mut Thrust,
        ),
        With<Plane>,
    >,
    time: Res<Time>,
) {
    let Ok((entity, action_state, global_tx, limits, mut external_force,  mut control, mut thrust)) = query.get_single_mut() else {
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
        control.elevators = action_state.clamped_value(PlaneAction::Pitch) * 10.0;
    }
    if action_state.pressed(PlaneAction::Roll) {
        external_force.torque += global_tx.forward()
            * action_state.clamped_value(PlaneAction::Roll)
            * time.delta_seconds()
            * 20.0;
    }
    if action_state.pressed(PlaneAction::Throttle) {
        thrust.0 += action_state.clamped_value(PlaneAction::Throttle) * time.delta_seconds() * 10.0;
        thrust.0 = thrust.0.clamp(0., limits.thrust);
    }
    if action_state.pressed(PlaneAction::Rudder) {
        external_force.torque += global_tx.up()
            * -action_state.clamped_value(PlaneAction::Rudder)
            * time.delta_seconds()
            * 10.0;
    }

    if action_state.just_pressed(PlaneAction::FollowAbove) {
        commands
            .entity(entity)
            .insert(Follow(camera::FollowKind::Above));
    }
    if action_state.just_pressed(PlaneAction::FollowBehind) {
        commands
            .entity(entity)
            .insert(Follow(camera::FollowKind::Behind));
    }
    if action_state.just_pressed(PlaneAction::FollowInside) {
        commands
            .entity(entity)
            .insert(Follow(camera::FollowKind::Inside));
    }
}

fn angle_of_attack_signed(global_tx: &GlobalTransform, velocity: Vec3) -> f32 {
    let a1 = Vec3::Y.angle_between(global_tx.forward());
    let a2 = Vec3::Y.angle_between(velocity.normalize());

    a2 - a1
}

fn update_airfoils_rotations(
    query: Query<(&PlaneControl, &Children)>,
    mut airfoil_query: Query<(&Airfoil, &mut Transform)>,
) {
    for (control, children) in query.iter() {
        for child in children.iter() {
            if let Ok((airfoil, mut airfoil_tx)) = airfoil_query.get_mut(*child) {
                match airfoil.position {
                    AirfoilPosition::HorizontalTailLeft | AirfoilPosition::HorizontalTailRight => {
                        airfoil_tx.rotation = Quat::from_rotation_x(control.elevators);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn update_airspeed(mut plane_query: Query<(&GlobalTransform, &Velocity, &mut Airspeed)>) {
    for (global_tx, velocity, mut airspeed) in plane_query.iter_mut() {
        let local_velocity = (global_tx.translation() + velocity.linvel) - global_tx.translation();
        airspeed.0 = -local_velocity.z;
    }
}

fn update_thrust_forces(
    mut plane_query: Query<(&Thrust, &GlobalTransform, &mut ExternalForce), With<Plane>>,
) {
    for (Thrust(thrust), global_tx, mut external_force) in plane_query.iter_mut() {
        external_force.force = global_tx.forward() * *thrust;
    }
}

fn update_airfoil_forces(
    mut plane_query: Query<
        (
            &mut PlaneFlight,
            &PlaneLimits,
            &GlobalTransform,
            &Airspeed,
            &Velocity,
            &ReadMassProperties,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
    airfoil_query: Query<(&Airfoil, &GlobalTransform)>,
) {
    for (
        mut flight,
        limits,
        global_tx,
        Airspeed(airspeed),
        velocity,
        ReadMassProperties(mass_properties),
        mut external_force,
    ) in plane_query.iter_mut()
    {
        let air_density = 1.225; // 1.225 kg/m^3 at sea level
        let dynamic_pressure = 0.5 * air_density * airspeed * airspeed;

        for (airfoil, airfoil_global_tx) in airfoil_query.iter() {
            let angle_of_attack = angle_of_attack_signed(airfoil_global_tx, velocity.linvel);

            let lift_coefficient_index = (angle_of_attack.to_degrees() + 90.0) as usize;

            let lift_coefficient = airfoil
                .lift_coefficient_samples
                .get(lift_coefficient_index)
                .unwrap_or(&0.0);

            let lift = lift_coefficient * dynamic_pressure * airfoil.area;

            let tx = global_tx.compute_transform();
            let centre_of_mass =
                tx.translation + (tx.rotation * mass_properties.local_center_of_mass);

            let centre_of_mass_1 =
                global_tx.translation() + global_tx.forward() * limits.wing_offset_z;

            // info!("{:?}: lift={}", airfoil.position, lift);
            info!(
                "cog={:?} {:?} {:?}",
                mass_properties.local_center_of_mass, centre_of_mass, centre_of_mass_1
            );

            match airfoil.position {
                AirfoilPosition::Wings => {
                    external_force.add_assign(ExternalForce::at_point(
                        airfoil_global_tx.up() * lift,
                        airfoil_global_tx.translation()
                            + global_tx.forward() * limits.wing_offset_z,
                        centre_of_mass,
                    ));

                    let drag_coefficient = 0.032; // For Cessna 172 at sea level and 100 knots at 0 degrees angle of attack
                    let drag = drag_coefficient * dynamic_pressure * airfoil.area;
                    external_force.force += -velocity.linvel.normalize_or_zero() * drag;

                    flight.lift = lift;
                    flight.drag = drag;
                }
                AirfoilPosition::HorizontalTailLeft | AirfoilPosition::HorizontalTailRight => {
                    external_force.add_assign(ExternalForce::at_point(
                        airfoil_global_tx.up() * lift * 0.01,
                        airfoil_global_tx.translation(),
                        centre_of_mass,
                    ));
                }
                _ => {}
            }
        }
    }
}
