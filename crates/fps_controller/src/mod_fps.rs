use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{input::mouse::MouseMotion, math::Vec3Swizzles, prelude::*};
use bevy_rapier3d::prelude::*;

#[derive(Default)]
pub struct FPSControllerPlugin;

impl Plugin for FPSControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((controller_input, controller_move, controller_render));
    }
}

#[derive(Component)]
pub struct LogicalPlayer;

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component, Default)]
pub struct FpsControllerInput {
    pub fly: bool,
    pub sprint: bool,
    pub jump: bool,
    pub crouch: bool,
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
}

#[derive(PartialEq, Default)]
pub enum MoveMode {
    #[default]
    Ground,
    Noclip,
}

#[derive(Component)]
pub struct FpsController {
    pub move_mode: MoveMode,
    pub radius: f32,
    pub gravity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    /// If the dot product (alignment) of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, ground movement is applied
    pub traction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub jump_speed: f32,
    pub fly_speed: f32,
    pub crouched_speed: f32,
    pub crouch_speed: f32,
    pub uncrouch_speed: f32,
    pub height: f32,
    pub upright_height: f32,
    pub crouch_height: f32,
    pub fast_fly_speed: f32,
    pub fly_friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub ground_tick: u8,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub enable_input: bool,
    pub step_offset: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_sprint: KeyCode,
    pub key_jump: KeyCode,
    pub key_fly: KeyCode,
    pub key_crouch: KeyCode,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            move_mode: MoveMode::Ground,
            radius: 0.5,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 9.0,
            run_speed: 14.0,
            forward_speed: 30.0,
            side_speed: 30.0,
            air_speed_cap: 2.0,
            air_acceleration: 20.0,
            max_air_speed: 15.0,
            crouched_speed: 5.0,
            crouch_speed: 6.0,
            uncrouch_speed: 8.0,
            height: 1.0,
            upright_height: 2.0,
            crouch_height: 1.0,
            acceleration: 10.0,
            friction: 10.0,
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            fly_friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            ground_tick: 0,
            stop_speed: 1.0,
            jump_speed: 10.5,
            step_offset: 0.0,
            enable_input: true,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_sprint: KeyCode::LShift,
            key_jump: KeyCode::Space,
            key_fly: KeyCode::F,
            key_crouch: KeyCode::C,
            sensitivity: 0.005,
        }
    }
}

const ANGLE_EPSILON: f32 = 0.001953125;

pub fn controller_input(
    key_input: Res<Input<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&mut FpsController, &mut FpsControllerInput)>,
) {
    for (mut controller, mut input) in query.iter_mut() {
        if !controller.enable_input {
            continue;
        }

        let mut mouse_delta = Vec2::ZERO;
        for mouse_event in mouse_events.iter() {
            mouse_delta += mouse_event.delta;
        }
        mouse_delta *= controller.sensitivity;

        input.pitch = (input.pitch - mouse_delta.y).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
        input.yaw -= mouse_delta.x;
        if input.yaw.abs() > PI {
            input.yaw = input.yaw.rem_euclid(TAU);
        }

        controller.pitch = input.pitch;
        controller.yaw = input.yaw;

        input.movement = Vec3::new(
            get_axis(&key_input, controller.key_right, controller.key_left),
            get_axis(&key_input, controller.key_up, controller.key_down),
            get_axis(&key_input, controller.key_forward, controller.key_back),
        );
        input.sprint = key_input.pressed(controller.key_sprint);
        input.jump = key_input.just_pressed(controller.key_jump);
        input.crouch = key_input.pressed(controller.key_crouch);
    }
}

pub fn controller_move(
    time: Res<Time>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &FpsControllerInput,
        &mut FpsController,
        &mut Collider,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    let dt = time.delta_seconds();

    for (entity, input, mut controller, mut collider, mut transform, mut velocity) in query.iter_mut() {
        if let Some(capsule) = collider.as_capsule() {
            // Capsule cast downwards to find ground
            // Better than a ray cast as it handles when you are near the edge of a surface
            let capsule = capsule.raw;
            let cast_capsule =
                Collider::capsule(capsule.segment.a.into(), capsule.segment.b.into(), capsule.radius * 0.9);
            // Avoid self collisions
            let filter = QueryFilter::default().exclude_rigid_body(entity).exclude_sensors();
            let ground_cast = physics_context.cast_shape(
                transform.translation,
                transform.rotation,
                -Vec3::Y,
                &cast_capsule,
                0.125,
                filter,
            );

            let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, input.yaw);
            move_to_world.z_axis *= -1.0; // Forward is -Z

            let speeds = Vec3::new(controller.side_speed, 0.0, controller.forward_speed);
            let mut wish_direction = move_to_world * (input.movement * speeds);
            let mut wish_speed = wish_direction.length();
            if wish_speed > f32::EPSILON {
                // Avoid division by zero
                wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
            }
            let max_speed = if input.crouch {
                controller.crouched_speed
            } else if input.sprint {
                controller.run_speed
            } else {
                controller.walk_speed
            };
            wish_speed = f32::min(wish_speed, max_speed);

            if let Some((_, toi)) = ground_cast {
                let has_traction = Vec3::dot(toi.normal1, Vec3::Y) > controller.traction_normal_cutoff;

                // Only apply friction after at least one tick, allows b-hopping without losing speed
                if controller.ground_tick >= 1 && has_traction {
                    let lateral_speed = velocity.linvel.xz().length();
                    if lateral_speed > controller.friction_speed_cutoff {
                        let control = f32::max(lateral_speed, controller.stop_speed);
                        let drop = control * controller.friction * dt;
                        let new_speed = f32::max((lateral_speed - drop) / lateral_speed, 0.0);
                        velocity.linvel.x *= new_speed;
                        velocity.linvel.z *= new_speed;
                    } else {
                        velocity.linvel = Vec3::ZERO;
                    }
                    if controller.ground_tick == 1 {
                        velocity.linvel.y = -toi.toi;
                    }
                }

                let mut add = acceleration(wish_direction, wish_speed, controller.acceleration, velocity.linvel, dt);
                if !has_traction {
                    add.y -= controller.gravity * dt;
                }
                velocity.linvel += add;

                if has_traction {
                    let linvel = velocity.linvel;
                    velocity.linvel -= Vec3::dot(linvel, toi.normal1) * toi.normal1;

                    if input.jump {
                        velocity.linvel.y = controller.jump_speed;
                    }
                }

                // Increment ground tick but cap at max value
                controller.ground_tick = controller.ground_tick.saturating_add(1);
            } else {
                controller.ground_tick = 0;
                wish_speed = f32::min(wish_speed, controller.air_speed_cap);

                let mut add = acceleration(
                    wish_direction,
                    wish_speed,
                    controller.air_acceleration,
                    velocity.linvel,
                    dt,
                );
                add.y = -controller.gravity * dt;
                velocity.linvel += add;

                let air_speed = velocity.linvel.xz().length();
                if air_speed > controller.max_air_speed {
                    let ratio = controller.max_air_speed / air_speed;
                    velocity.linvel.x *= ratio;
                    velocity.linvel.z *= ratio;
                }
            }

            // Crouching
            let crouch_height = controller.crouch_height;
            let upright_height = controller.upright_height;

            let crouch_speed = if input.crouch { -controller.crouch_speed } else { controller.uncrouch_speed };
            controller.height += dt * crouch_speed;
            controller.height = controller.height.clamp(crouch_height, upright_height);

            if let Some(mut capsule) = collider.as_capsule_mut() {
                // capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5);
                capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5 * (controller.height - 1.0));
            }

            // Step offset
            if controller.step_offset > f32::EPSILON && controller.ground_tick >= 1 {
                let cast_offset = velocity.linvel.normalize_or_zero() * controller.radius * 1.0625;
                let cast = physics_context.cast_ray_and_get_normal(
                    transform.translation + cast_offset + Vec3::Y * controller.step_offset * 1.0625,
                    -Vec3::Y,
                    controller.step_offset * 0.9375,
                    false,
                    filter,
                );
                if let Some((_, hit)) = cast {
                    transform.translation.y += controller.step_offset * 1.0625 - hit.toi;
                    transform.translation += cast_offset;
                }
            }
        }
    }
}

fn acceleration(wish_direction: Vec3, wish_speed: f32, acceleration: f32, velocity: Vec3, dt: f32) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
}

fn get_pressed(key_input: &Res<Input<KeyCode>>, key: KeyCode) -> f32 {
    if key_input.pressed(key) {
        1.0
    } else {
        0.0
    }
}

fn get_axis(key_input: &Res<Input<KeyCode>>, key_pos: KeyCode, key_neg: KeyCode) -> f32 {
    get_pressed(key_input, key_pos) - get_pressed(key_input, key_neg)
}

pub fn controller_render(
    logical_query: Query<(&Transform, &Collider, &FpsController), With<LogicalPlayer>>,
    mut render_query: Query<&mut Transform, (With<RenderPlayer>, Without<LogicalPlayer>)>,
) {
    // TODO: inefficient O(N^2) loop, use hash map?
    for (logical_transform, collider, controller) in logical_query.iter() {
        if let Some(capsule) = collider.as_capsule() {
            for mut render_transform in render_query.iter_mut() {
                // TODO: let this be more configurable
                let camera_height = capsule.segment().b().y + capsule.radius() * 0.75;
                render_transform.translation = logical_transform.translation + Vec3::Y * camera_height;
                render_transform.rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
            }
        }
    }
}
