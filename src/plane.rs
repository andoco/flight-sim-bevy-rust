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
            .add_systems((handle_keyboard_input, compute_forces, apply_forces).chain());
    }
}

#[derive(Component)]
pub struct Plane;

fn setup_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Plane,
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
        ),
        With<Plane>,
    >,
) {
    let Ok((action_state, global_tx, mut external_force)) = query.get_single_mut() else {
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
        external_force.torque = global_tx.right() * -0.1;
    }
    if action_state.pressed(PlaneAction::PitchDown) {
        external_force.torque = global_tx.right() * 0.1;
    }
    if action_state.pressed(PlaneAction::ThrustUp) {
        external_force.force += global_tx.forward() * 0.1;
    }
    if action_state.pressed(PlaneAction::ThrustDown) {
        external_force.force -= global_tx.forward() * 0.1;
    }
}

#[derive(Component)]
struct PlaneForce {
    lift: f32,
}

fn compute_forces(mut commands: Commands, query: Query<(Entity, &Velocity), With<Plane>>) {
    for (entity, velocity) in query.iter() {
        let lift = 40.0 * velocity.linvel.length_squared();
        commands.entity(entity).insert(PlaneForce { lift });
    }
}

fn apply_forces(
    mut query: Query<(&GlobalTransform, &PlaneForce, &mut ExternalForce), With<Plane>>,
) {
    for (global_tx, plane_force, mut external_force) in query.iter_mut() {
        external_force.force = global_tx.up() * plane_force.lift;
    }
}

// fn stabilise(mut query: Query<(&GlobalTransform, &mut ExternalForce), With<Plane>>) {
//     for (global_tx, mut external_force) in query.iter_mut() {
//         let (_, rotation, _) = global_tx.to_scale_rotation_translation();
//         let (x, y, z) = rotation.to_euler(EulerRot::XYZ);

//         info!("{}", z.to_degrees());

//         // let x_factor = 0.1 / 10.0 * x.abs().to_degrees().min(10.0) * -x.signum();
//         let z_factor = 0.1 / 10.0 * z.abs().to_degrees().min(10.0) * -z.signum();

//         // external_force.torque = ((global_tx.right() * x_factor).normalize_or_zero()
//         //     + (global_tx.forward() * z_factor).normalize_or_zero())
//         //     * 0.1;

//         // let rot = Quat::from_rotation_arc(global_tx.forward(), Vec3::Z);

//         external_force.torque = global_tx.forward() * z_factor;
//     }
// }
