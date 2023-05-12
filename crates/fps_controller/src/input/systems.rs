use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};
use egui_helper::EguiHelperState;

use super::components::*;

const ANGLE_EPSILON: f32 = 0.001953125;

pub(crate) fn setup(
    mut commands: Commands,
    q: Query<Entity, With<FpsPlayer>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for entity in q.iter() {
        commands
            .entity(entity)
            .insert((FpsControllerInput::default(), FpsControllerInputConfig::default()));

        commands.entity(entity).with_children(|builder| {
            builder.spawn(PolylineBundle {
                polyline: polylines.add(Polyline {
                    vertices: crate::get_circle_xz_pts(Vec3::new(0.0, -1., 0.0), 3.0, 4),
                }),
                material: polyline_materials.add(PolylineMaterial {
                    width: 12.0,
                    color: Color::YELLOW_GREEN,
                    perspective: true,
                    depth_bias: -0.0002,
                }),
                ..Default::default()
            });

            // velocity and wish direction
            builder.spawn(PolylineBundle {
                polyline: polylines.add(Polyline {
                    vertices: vec![Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, -1.0, -6.0)],
                }),
                material: polyline_materials.add(PolylineMaterial {
                    width: 25.0,
                    color: Color::INDIGO,
                    perspective: true,
                    depth_bias: -0.0002,
                }),
                ..Default::default()
            });

            builder.spawn(PolylineBundle {
                polyline: polylines.add(Polyline {
                    vertices: vec![Vec3::new(0.0, -1.0, 0.0), Vec3::new(1.0, -1.0, -6.0)],
                }),
                material: polyline_materials.add(PolylineMaterial {
                    width: 25.0,
                    color: Color::ORANGE_RED,
                    perspective: true,
                    depth_bias: -0.0002,
                }),
                ..Default::default()
            });
        });
    }
}

pub(crate) fn controller_input(
    time: Res<Time>,
    key_input: Res<Input<KeyCode>>,
    egui_state: Res<EguiHelperState>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&FpsControllerInputConfig, &mut FpsControllerInput)>,
) {
    if egui_state.wants_input {
        return;
    };

    for (controller, mut input) in query.iter_mut() {
        if !controller.enable_input {
            continue;
        }

        let mut mouse_delta: Vec2 = mouse_events
            .iter()
            .fold(Vec2::ZERO, |collector, evt| collector + evt.delta);
        mouse_delta *= controller.sensitivity * time.delta_seconds(); // is this correct calcuation

        input.pitch = mouse_delta.y;
        input.yaw = mouse_delta.x;

        input.sprint = key_input.pressed(controller.key_sprint);
        input.jump = key_input.just_pressed(controller.key_jump);
        input.fly = key_input.just_pressed(controller.key_fly);
        input.crouch = key_input.pressed(controller.key_crouch);

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
    }
}

pub(crate) fn calculate_movement(
    _time: Res<Time>,
    _query: Query<&FpsControllerInput>,
    _render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    // TODO: should this handle doing basic integration of input + frictions/accelerations?
}

#[allow(dead_code)]
pub(crate) fn manage_cursor(
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsControllerInputConfig>,
) {
    let mut window = window_query.single_mut();
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        for mut controller in &mut controller_query {
            controller.enable_input = true;
        }
    }
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
    mut logical_query: Query<(&mut Transform, &FpsControllerInput), With<FpsPlayer>>,
    mut render_query: Query<&mut Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    if egui_state.wants_input {
        return;
    };

    for (mut logical_tf, controller) in logical_query.iter_mut() {
        for mut render_tf in render_query.iter_mut() {
            let (_, render_pitch, _) = render_tf.rotation.to_euler(EulerRot::YXZ);
            let (logical_yaw, _, _) = logical_tf.rotation.to_euler(EulerRot::YXZ);

            let mut yaw = logical_yaw - controller.yaw;
            let pitch = (render_pitch - controller.pitch).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
            if yaw.abs() > PI {
                yaw = yaw.rem_euclid(TAU);
            }

            logical_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
            render_tf.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
        }
    }
}
