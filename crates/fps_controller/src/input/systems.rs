use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{prelude::*, window::CursorGrabMode};
use egui_helper::EguiHelperState;
use leafwing_input_manager::prelude::*;

use crate::ultrakill::FpsControllerState;

use super::components::*;

const ANGLE_EPSILON: f32 = 0.001953125;

pub(crate) fn setup(mut commands: Commands, q: Query<Entity, With<FpsPlayer>>) {
    let Ok(entity) = q.get_single() else { return; };

    let input_map = InputMap::default()
        .insert(VirtualDPad::wasd(), InputAction::Move)
        .insert(DualAxis::left_stick(), InputAction::Move)
        // look
        .insert(DualAxis::mouse_motion(), InputAction::MouseLook)
        .insert(DualAxis::right_stick(), InputAction::ControllerLook)
        // jump
        .insert(KeyCode::Space, InputAction::Jump)
        .insert(GamepadButtonType::South, InputAction::Jump)
        // slide/slam
        .insert(KeyCode::LControl, InputAction::Slide)
        .insert(GamepadButtonType::East, InputAction::Slide)
        // dash
        .insert(KeyCode::LShift, InputAction::Dash)
        .insert(GamepadButtonType::LeftThumb, InputAction::Dash)
        .insert(GamepadButtonType::West, InputAction::Dash)
        // shoot
        .insert(MouseButton::Left, InputAction::Shoot)
        .insert(GamepadButtonType::RightTrigger2, InputAction::Shoot)
        .build();

    commands.entity(entity).insert((
        FpsControllerInput::default(),
        FpsControllerInputConfig::default(),
        InputManagerBundle::<InputAction> {
            action_state: ActionState::default(),
            input_map,
        },
    ));
}

pub(crate) fn controller_input(
    time: Res<Time>,
    egui_state: Res<EguiHelperState>,
    mut query: Query<(
        &Transform,
        &FpsControllerInputConfig,
        &mut FpsControllerInput,
        &InputActions,
    )>,
) {
    for (tf, controller, mut input, actions) in query.iter_mut() {
        input.yaw = 0.0;
        input.pitch = 0.0;

        // ignore mouse input if egui wants input but still gather keyboard input to avoid stuck keys
        if !egui_state.wants_input {
            if actions.pressed(InputAction::MouseLook) {
                let camera_delta = actions.axis_pair(InputAction::MouseLook).unwrap();
                let camera_delta = camera_delta.xy() * controller.mouse_sensitivity * time.delta_seconds();

                input.yaw = camera_delta.x;
                input.pitch = camera_delta.y;
            }
        }

        if actions.pressed(InputAction::ControllerLook) {
            let camera_move = actions
                .clamped_axis_pair(InputAction::ControllerLook)
                .unwrap()
                .xy()
                .normalize();

            input.yaw = camera_move.x * controller.gamepad_sensitivity * time.delta_seconds();
            input.pitch = camera_move.y * controller.gamepad_sensitivity * time.delta_seconds();
        }

        input.slide.pressed = actions.just_pressed(InputAction::Slide);
        input.slide.down = actions.pressed(InputAction::Slide);
        input.slide.released = actions.just_released(InputAction::Slide);

        input.jump.pressed = actions.just_pressed(InputAction::Jump);
        input.jump.down = actions.pressed(InputAction::Jump);
        input.jump.released = actions.just_released(InputAction::Jump);

        input.dash.pressed = actions.just_pressed(InputAction::Dash);
        input.dash.down = actions.pressed(InputAction::Dash);
        input.dash.released = actions.just_released(InputAction::Dash);

        input.movement = if actions.pressed(InputAction::Move) {
            let axis_pair = actions.clamped_axis_pair(InputAction::Move).unwrap();
            let axis_pair = axis_pair.xy().normalize_or_zero();
            Vec3::new(axis_pair.x, 0.0, axis_pair.y)
        } else {
            Vec3::ZERO
        };

        input.movement_dir = tf.right() * input.movement.x + tf.forward() * input.movement.z;

        // store off the dodge/slide direction if starting a dodge/slide
        if input.slide.pressed || input.dash.pressed {
            input.dash_slide_dir = if input.movement == Vec3::ZERO { tf.forward() } else { input.movement_dir };
        }
    }
}

pub(crate) fn temp_input_test(q: Query<&InputActions, With<FpsPlayer>>) {
    let Ok(input) = q.get_single() else { return; };

    if input.pressed(InputAction::ControllerLook) {
        let _camera_pan_vector = input.axis_pair(InputAction::ControllerLook).unwrap();
        println!("controller cam: {:?}", _camera_pan_vector);
    }
}

#[allow(dead_code)]
pub(crate) fn manage_cursor(
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window>,
) {
    let mut window = window_query.single_mut();

    // if !egui_state.wants_input && !egui_state.enabled {
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
    // }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

/// syncs the yaw to the FpsPlayer and the pitch to the RenderPlayer
pub(crate) fn sync_rotation_input(
    egui_state: Res<EguiHelperState>,
    mut player_query: Query<(&mut Transform, &FpsControllerInput, &FpsControllerState), With<FpsPlayer>>,
    mut render_query: Query<&mut Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
    time: Res<Time>,
) {
    if egui_state.wants_input {
        return;
    };

    let Ok((mut player_tf, input, controller_state)) = player_query.get_single_mut() else { return };
    let Ok(mut render_tf) = render_query.get_single_mut() else { return };

    let (_, render_pitch, render_tilt) = render_tf.rotation.to_euler(EulerRot::YXZ);
    let (logical_yaw, _, _) = player_tf.rotation.to_euler(EulerRot::YXZ);

    let mut yaw = logical_yaw - input.yaw;
    let pitch = (render_pitch - input.pitch).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
    if yaw.abs() > PI {
        yaw = yaw.rem_euclid(TAU);
    }

    let tilt_multipler: f32 = if controller_state.boost { 5.0 } else { 1.0 };
    player_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    render_tf.rotation = Quat::from_euler(
        EulerRot::YXZ,
        0.0,
        pitch,
        move_towards(
            render_tilt,
            input.movement.x * -tilt_multipler.to_radians(),
            time.delta_seconds() * 0.7,
        ),
    );
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}
