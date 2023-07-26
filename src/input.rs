use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap, SingleAxis},
    Actionlike, InputManagerBundle,
};

use crate::{
    camera::{self, Follow},
    plane::{Plane, PlaneControl, PlaneLimits, Thrust},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlaneAction>::default())
            .add_system(add_plane_input)
            .add_system(handle_keyboard_input)
            .add_system(handle_gamepad_input);
    }
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
        control.clear();
    }

    if action_state.pressed(PlaneAction::RollLeft) {
        control.ailerons = -1_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::RollRight) {
        control.ailerons = 1_f32.to_radians();
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
        control.elevators = (action_state.clamped_value(PlaneAction::Pitch) * 10.0).to_radians();
    }
    if action_state.pressed(PlaneAction::Roll) {
        control.ailerons = (action_state.clamped_value(PlaneAction::Roll) * 1.0).to_radians();
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
