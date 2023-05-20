use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use egui_helper::EguiHelperState;

use super::components::*;

const ANGLE_EPSILON: f32 = 0.001953125;

pub(crate) fn setup(mut commands: Commands, q: Query<Entity, With<FpsPlayer>>) {
    let Ok(entity) = q.get_single() else { return; };
    commands
        .entity(entity)
        .insert((FpsControllerInput::default(), FpsControllerInputConfig::default()));
}

pub(crate) fn controller_input(
    time: Res<Time>,
    key_input: Res<Input<KeyCode>>,
    egui_state: Res<EguiHelperState>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&Transform, &FpsControllerInputConfig, &mut FpsControllerInput)>,
) {
    for (tf, controller, mut input) in query.iter_mut() {
        if !controller.enable_input {
            continue;
        }

        // ignore mouse input if egui wants input but still gather keyboard input to avoid stuck keys
        if !egui_state.wants_input {
            let mut mouse_delta: Vec2 = mouse_events
                .iter()
                .fold(Vec2::ZERO, |collector, evt| collector + evt.delta);
            mouse_delta *= controller.sensitivity * time.delta_seconds(); // is this correct calcuation

            input.pitch = mouse_delta.y;
            input.yaw = mouse_delta.x;
        }

        input.slide.pressed = key_input.just_pressed(controller.key_slide);
        input.slide.down = key_input.pressed(controller.key_slide);
        input.slide.released = key_input.just_released(controller.key_slide);

        input.jump.pressed = key_input.just_pressed(controller.key_jump);
        input.jump.down = key_input.pressed(controller.key_jump);
        input.jump.released = key_input.just_released(controller.key_jump);

        input.dash.pressed = key_input.just_pressed(controller.key_dash);
        input.dash.down = key_input.pressed(controller.key_dash);
        input.dash.released = key_input.just_released(controller.key_dash);

        fn get_axis(key_input: &Res<Input<KeyCode>>, key_pos: KeyCode, key_neg: KeyCode) -> f32 {
            let get_pressed = |b| if b { 1.0 } else { 0.0 };
            get_pressed(key_input.pressed(key_pos)) - get_pressed(key_input.pressed(key_neg))
        }

        input.movement = Vec3::new(
            get_axis(&key_input, controller.key_right, controller.key_left),
            0.0,
            get_axis(&key_input, controller.key_forward, controller.key_back),
        )
        .normalize_or_zero();

        input.movement_dir = tf.forward() * input.movement.z + tf.right() * input.movement.x;

        // store off the dodge/slide direction if starting a dodge/slide
        if input.slide.pressed || input.dash.pressed {
            input.dash_slide_dir = if input.movement == Vec3::ZERO { tf.forward() } else { input.movement_dir };
        }
    }
}

#[allow(dead_code)]
pub(crate) fn manage_cursor(
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsControllerInputConfig>,
) {
    let mut window = window_query.single_mut();

    // if !egui_state.wants_input && !egui_state.enabled {
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        for mut controller in &mut controller_query {
            controller.enable_input = true;
        }
    }
    // }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        for mut controller in &mut controller_query {
            controller.enable_input = false;
        }
    }
}

/// syncs the yaw to the FpsPlayer and the pitch to the RenderPlayer
pub(crate) fn sync_rotation_input(
    egui_state: Res<EguiHelperState>,
    mut player_query: Query<(&mut Transform, &FpsControllerInput), With<FpsPlayer>>,
    mut render_query: Query<&mut Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    if egui_state.wants_input {
        return;
    };

    let Ok((mut player_tf, controller)) = player_query.get_single_mut() else { return };
    let Ok(mut render_tf) = render_query.get_single_mut() else { return };

    let (_, render_pitch, render_tilt) = render_tf.rotation.to_euler(EulerRot::YXZ);
    let (logical_yaw, _, _) = player_tf.rotation.to_euler(EulerRot::YXZ);

    let mut yaw = logical_yaw - controller.yaw;
    let pitch = (render_pitch - controller.pitch).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
    if yaw.abs() > PI {
        yaw = yaw.rem_euclid(TAU);
    }

    player_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    render_tf.rotation = Quat::from_euler(
        EulerRot::YXZ,
        0.0,
        pitch,
        move_towards(render_tilt, controller.tilt, 1.0),
    );
    // TODO: speed * dt
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}
