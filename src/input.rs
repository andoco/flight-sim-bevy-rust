use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap, SingleAxis},
    Actionlike, InputManagerBundle,
};

use crate::{
    camera::{self, Follow},
    plane::{spec::PlaneSpec, Plane, PlaneControl, Thrust},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlaneAction>::default())
            .add_systems(Startup, add_plane_input)
            .add_systems(Update, (handle_keyboard_input, handle_gamepad_input));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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

fn add_plane_input(mut commands: Commands) {
    info!("Adding input");

    commands.spawn(InputManagerBundle::<PlaneAction> {
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
            .insert(KeyCode::F1, PlaneAction::FollowBehind)
            .insert(KeyCode::F2, PlaneAction::FollowAbove)
            .insert(KeyCode::F3, PlaneAction::FollowInside)
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

fn handle_keyboard_input(
    mut action_query: Query<&ActionState<PlaneAction>>,
    mut plane_query: Query<(&PlaneSpec, &mut PlaneControl, &mut Thrust), With<Plane>>,
    time: Res<Time>,
) {
    let Ok(action_state  ) = action_query.get_single_mut() else {
        return
    };
    let Ok((spec, mut control, mut thrust)) = plane_query.get_single_mut() else {
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
        control.rudder = -10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::YawRight) {
        control.rudder = 10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::PitchUp) {
        control.elevators = 10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::PitchDown) {
        control.elevators = -10_f32.to_radians();
    }
    if action_state.pressed(PlaneAction::ThrustUp) {
        thrust.0 += 50.0 * time.delta_seconds();
    }
    if action_state.pressed(PlaneAction::ThrustDown) {
        thrust.0 -= 50.0 * time.delta_seconds();
    }

    thrust.0 = thrust.0.clamp(0., spec.thrust);
}

fn handle_gamepad_input(
    mut commands: Commands,
    mut action_query: Query<&ActionState<PlaneAction>>,
    mut plane_query: Query<
        (
            Entity,
            &PlaneSpec,
            &mut ExternalForce,
            &mut PlaneControl,
            &mut Thrust,
        ),
        With<Plane>,
    >,
    time: Res<Time>,
) {
    let Ok(action_state  ) = action_query.get_single_mut() else {
        return
    };
    let Ok((entity, spec, mut external_force,  mut control, mut thrust)) = plane_query.get_single_mut() else {
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
        thrust.0 = thrust.0.clamp(0., spec.thrust);
    }
    if action_state.pressed(PlaneAction::Rudder) {
        control.rudder = (action_state.clamped_value(PlaneAction::Rudder) * 1.0).to_radians();
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
