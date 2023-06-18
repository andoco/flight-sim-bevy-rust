use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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
                    compute_angle_of_attack,
                    apply_thrust,
                    apply_drag,
                    apply_lift,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct Plane;

#[derive(Component, Default)]
pub struct PlaneFlight {
    pub thrust: f32,
    pub angle_of_attack: f32,
}

fn setup_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Plane,
            PlaneFlight::default(),
            SpatialBundle::from_transform(Transform::from_xyz(0., 20., 0.)),
            RigidBody::Dynamic,
            Velocity {
                linvel: -Vec3::Z * 3.2,
                ..default()
            },
            ExternalForce::default(),
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
                Collider::cuboid(5.0, 0.1, 1.0),
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
            &mut ExternalForce,
            &mut PlaneFlight,
        ),
        With<Plane>,
    >,
    time: Res<Time>,
) {
    let Ok((action_state, global_tx, mut external_force, mut flight)) = query.get_single_mut() else {
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
}

fn compute_angle_of_attack(
    mut query: Query<(&GlobalTransform, &Velocity, &mut PlaneFlight), With<Plane>>,
) {
    for (global_tx, velocity, mut flight) in query.iter_mut() {
        flight.angle_of_attack = velocity
            .linvel
            .normalize()
            .dot(global_tx.forward())
            .powf(2.);
    }
}

fn apply_thrust(
    mut query: Query<(&GlobalTransform, &PlaneFlight, &mut ExternalForce), With<Plane>>,
) {
    for (global_tx, flight, mut external_force) in query.iter_mut() {
        external_force.force = global_tx.forward() * flight.thrust;
    }
}

fn apply_drag(mut query: Query<(&GlobalTransform, &Velocity, &mut ExternalForce), With<Plane>>) {
    for (global_tx, velocity, mut external_force) in query.iter_mut() {
        let drag = 10.0 * velocity.linvel.length();
        let drag_force = -velocity.linvel.normalize() * drag;
        external_force.force += drag_force;
    }
}

fn apply_lift(
    mut query: Query<
        (
            &GlobalTransform,
            &Velocity,
            &PlaneFlight,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
) {
    for (global_tx, velocity, flight, mut external_force) in query.iter_mut() {
        let lift_power = 40.0 * velocity.linvel.length_squared();

        let lift_direction = global_tx.up();

        let lift_force = lift_direction * lift_power * flight.angle_of_attack;

        external_force.force += lift_force;
    }
}
