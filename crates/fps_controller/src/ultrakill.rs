use crate::{
    camera_shake::Shake3d,
    input::{FpsControllerInput, FpsControllerStages},
    time_controller::TimeScaleModificationEvent,
};
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;
use egui_helper::bevy_inspector_egui::{
    bevy_egui::EguiContext,
    egui::{self, DragValue, Pos2},
};

#[derive(Default)]
pub struct UltrakillControllerPlugin;

impl Plugin for UltrakillControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FpsControllerState>()
            .add_system(controller_move.in_set(FpsControllerStages::Logic))
            .add_system(debug_ui.run_if(egui_helper::run_if_egui_enabled));
    }
}

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsController {
    pub radius: f32,
    pub gravity: f32,

    pub walk_speed: f32,
    pub slide_speed: f32,
    pub dash_speed: f32,
    pub jump_speed: f32,
    /// additional force applied while jumping if jump is still pressed
    pub jump_down_speed: f32,
    /// how long to wait before stopping a jump by setting vel.y = 0. A jump_time of 0 will turn off variable height jumps.
    pub jump_time: f32,
    /// the amount of force to apply downwards when the jump button is released prior to jump_time expiring.
    pub jump_stop_force: f32,
    pub slide_jump_speed: f32,
    pub dash_jump_speed: f32,
    pub wall_jump_speed: f32,
    pub crouch_speed: f32,
    pub uncrouch_speed: f32,

    pub jump_buffer_duration: f32,
    pub coyote_timer_duration: f32,

    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub ground_slam_speed: f32,
    pub max_fall_velocity: f32,
    pub friction: f32,
    /// If the dot product (alignment) of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, ground movement is applied
    pub traction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub height: f32,
    pub upright_height: f32,
    pub crouch_height: f32,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub enable_input: bool,
    pub step_offset: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            radius: 0.5,
            gravity: 23.0,

            walk_speed: 20.0 * 30.0,
            slide_speed: 35.0 * 30.0,
            dash_speed: 150.0 * 30.0,
            jump_speed: 10.5, // * 2.6 in UK
            jump_down_speed: 0.0,
            jump_time: 0.0,
            jump_stop_force: 0.0,
            slide_jump_speed: 8.0, // * 2.0 in UK
            dash_jump_speed: 8.0,  // * 1.5 in UK
            wall_jump_speed: 15.0,
            crouch_speed: 50.0,
            uncrouch_speed: 8.0,

            jump_buffer_duration: 0.10,
            coyote_timer_duration: 0.2,

            air_speed_cap: 2.0,
            air_acceleration: 50.0,
            ground_slam_speed: 50.0,
            max_fall_velocity: -100.0,
            max_air_speed: 15.0,
            height: 1.0,
            upright_height: 2.0,
            crouch_height: 1.0,
            acceleration: 10.0,
            friction: 10.0,
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            stop_speed: 1.0,
            step_offset: 0.0,
            enable_input: true,
            sensitivity: 0.005,
        }
    }
}

#[derive(Default, Reflect)]
pub struct CooldownTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub finished: bool,
    pub finished_this_tick: bool,
}

impl CooldownTimer {
    pub fn new(duration: f32) -> CooldownTimer {
        CooldownTimer {
            elapsed: 0.0,
            duration,
            ..Default::default()
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.finished {
            self.finished_this_tick = false;
            return;
        }

        self.elapsed += dt;

        if self.is_complete() {
            self.finished = true;
            self.finished_this_tick = true;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed > self.duration
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    pub fn reset_with_duration(&mut self, duration: f32) {
        self.reset();
        self.duration = duration;
    }
}

#[derive(Component, Default, Reflect)]
pub struct FpsControllerState {
    // states
    pub jumping: bool,
    pub sliding: bool,
    pub heavy_fall: bool,
    pub falling: bool,
    pub boost: bool,
    // live data
    pub boost_charge: f32,
    pub fall_time: f32,
    pub fall_speed: f32,
    pub slam_force: f32,
    pub slam_storage: bool,
    pub super_jump_chance: f32,
    pub extra_jump_chance: f32,
    // slide
    pub pre_slide_delay: f32,
    pub pre_slide_speed: f32,
    pub slide_safety_timer: f32,
    pub slide_length: f32,
    pub standing: bool,
    // jump/wall jump
    pub jump_cooldown: CooldownTimer,
    pub not_jumping_cooldown: CooldownTimer,
    pub jump_timer: f32,
    pub jump_buffer_timer: f32,
    pub coyote_timer: f32,
    pub current_wall_jumps: u8,
    pub cling_fade: f32,
    // dash/dodge
    pub boost_duration: f32,
    pub boost_left: f32,
    pub dash_storage: f32,
    pub slide_ending_this_frame: bool, // same as slideEnding
}

impl FpsControllerState {
    pub fn new() -> Self {
        Self {
            boost_charge: 300.0,
            jump_cooldown: CooldownTimer::new(0.2),
            not_jumping_cooldown: CooldownTimer::new(0.25),
            boost_duration: 0.15,
            ..Default::default()
        }
    }

    pub fn tick_timers(&mut self, dt: f32) {
        self.jump_cooldown.tick(dt);
        self.not_jumping_cooldown.tick(dt);

        if self.not_jumping_cooldown.finished_this_tick {
            self.jumping = false;
        }

        if self.super_jump_chance > 0.0 {
            self.super_jump_chance = move_towards(self.super_jump_chance, 0.0, dt);
            self.extra_jump_chance = 0.15;
        }

        if self.extra_jump_chance > 0.0 {
            self.extra_jump_chance = move_towards(self.extra_jump_chance, 0.0, dt);
            if self.extra_jump_chance <= 0.0 {
                self.slam_force = 0.0;
            }
        }
    }
}

pub fn controller_move(
    time: Res<Time>,
    mut _lines: ResMut<DebugLines>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &FpsControllerInput,
        &mut FpsControllerState,
        &mut FpsController,
        &mut Collider,
        &mut Transform,
        &mut Velocity,
    )>,
    mut shake_q: Query<&mut Shake3d>,
    mut _evt_time_mod: EventWriter<TimeScaleModificationEvent>,
) {
    let dt = time.delta_seconds();
    let mut shake = shake_q.single_mut();

    for (entity, input, mut state, mut controller, mut collider, mut transform, mut velocity) in query.iter_mut() {
        let Some(capsule) = collider.as_capsule() else { return };

        state.tick_timers(dt);

        // Capsule cast downwards to find ground
        let capsule = capsule.raw;
        let cast_capsule = Collider::capsule(capsule.segment.a.into(), capsule.segment.b.into(), capsule.radius * 0.9);

        let filter = QueryFilter::only_fixed().exclude_rigid_body(entity).exclude_sensors();
        let ground_cast = physics_context.cast_shape(
            transform.translation,
            transform.rotation,
            -Vec3::Y,
            &cast_capsule,
            0.125,
            filter,
        );
        let on_ground = ground_cast.is_some();

        // wall intersection check, we use a cylinder that is shorter but wider than the player
        let cast_cylinder = Collider::cylinder(0.4, 0.6);
        let filter = QueryFilter::only_fixed().exclude_rigid_body(entity).exclude_sensors();
        let mut on_wall = false;
        let mut closest_pt = Vec3::splat(f32::MAX);
        let mut closest_dist = f32::MAX;
        physics_context.intersections_with_shape(
            transform.translation,
            transform.rotation,
            &cast_cylinder,
            filter,
            |entity| {
                let player_pos = transform.translation;

                let predicate = &|e| e == entity;
                let inner_filter = QueryFilter::default().predicate(&predicate);
                if let Some(pt_res) = physics_context.project_point(transform.translation, true, inner_filter) {
                    let dist = player_pos.distance_squared(pt_res.1.point);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_pt = pt_res.1.point;
                    }
                }

                on_wall = true;
                true
            },
        );

        if on_ground {
            state.fall_time = 0.0;
            state.cling_fade = 0.0;
            state.coyote_timer = controller.coyote_timer_duration;
        } else {
            state.coyote_timer = (state.coyote_timer - dt).max(0.0);
            state.jump_buffer_timer = if input.jump.pressed {
                controller.jump_buffer_duration
            } else {
                (state.jump_buffer_timer - dt).max(0.0)
            };

            if state.jump_timer > 0.0 {
                if input.jump.down {
                    velocity.linvel.y += controller.jump_down_speed;
                    state.jump_timer = (state.jump_timer - dt).max(0.0);
                } else {
                    state.jump_timer = 0.0;
                    velocity.linvel.y = -controller.jump_stop_force;
                }
            }

            if state.fall_time < 1.0 {
                state.fall_time += dt * 5.0; // TODO: wtf? 5?
                if state.fall_time > 1.0 {
                    state.falling = true;
                }
            } else if velocity.linvel.y < -2.0 {
                state.fall_speed = velocity.linvel.y;
            }
        }

        let jump_requested = input.jump.pressed || state.jump_buffer_timer > 0.0;

        // clamp max fall velocity
        if velocity.linvel.y < controller.max_fall_velocity {
            velocity.linvel.y = controller.max_fall_velocity;
        }

        // falling and hit ground this frame
        if on_ground && state.falling && state.jump_cooldown.is_complete() {
            state.falling = false;
            state.slam_storage = false;

            if state.fall_speed <= -50.0 {
                shake.trauma = 0.5;
            }

            state.fall_speed = 0.0;
            state.heavy_fall = false;
        }

        let near_ground_check = physics_context.cast_ray(transform.translation, Vec3::NEG_Y, 2.0, false, filter);

        if !on_ground && input.slide.pressed {
            // TODO: if state.sliding { stop_slide() }
            state.sliding = false;
            state.slide_ending_this_frame = true;
            state.slide_length = 0.0;

            if state.boost {
                state.boost = false;
                state.boost_left = 0.0;
            }

            if state.fall_time > 0.5 && near_ground_check.is_none() && !state.heavy_fall {
                velocity.linvel = Vec3::new(0.0, -controller.ground_slam_speed, 0.0);
                state.falling = true;
                state.fall_speed = -controller.ground_slam_speed;
                state.heavy_fall = true;
                state.slam_force = 1.0;
            }
        }

        if state.heavy_fall {
            if !state.slam_storage {
                velocity.linvel = Vec3::new(0.0, -controller.ground_slam_speed, 0.0);
            }
            state.slam_force += dt * 5.0;
        }

        // if jump_requested && !state.falling && on_ground && state.jump_cooldown.is_complete() {
        let coyote_jump = jump_requested && !on_ground && state.coyote_timer > 0.0 && !on_wall;
        let normal_jump = jump_requested && !state.falling && on_ground;
        if (coyote_jump || normal_jump) && state.jump_cooldown.is_complete() {
            state.jump_timer = controller.jump_time;
            state.jump_buffer_timer = 0.0;
            state.current_wall_jumps = 0;
            state.cling_fade = 0.0;
            state.jumping = true;
            state.falling = true;
            state.not_jumping_cooldown.reset();

            velocity.linvel.y = 0.0;
            if state.sliding {
                // TODO: stop_slide()
                state.sliding = false;
                state.slide_ending_this_frame = true;
                state.slide_length = 0.0;
                velocity.linvel.y = controller.slide_jump_speed;
            } else if state.boost {
                if state.boost_charge > 100.0 {
                    state.boost_charge -= 100.0;
                    velocity.linvel.y = controller.dash_jump_speed;
                } else {
                    velocity.linvel = input.movement_dir * controller.walk_speed * dt;
                    velocity.linvel.y = 0.0;
                    shake.trauma = 0.6; // play stamina-failed sound instead
                }
            } else if state.super_jump_chance > 0.0 && state.extra_jump_chance > 0.0 {
                let jump_multiplier = if state.slam_force < 5.5 { 0.5 + state.slam_force } else { 10.0 };
                println!(
                    "--- Super Jump: slam_force: {}, jump_multiplier: {}",
                    state.slam_force, jump_multiplier
                );
                velocity.linvel.y = controller.jump_speed * jump_multiplier;
                state.slam_force = 0.0;
            } else {
                velocity.linvel.y = controller.jump_speed;
            }

            state.jump_cooldown.reset_with_duration(0.25);
            state.boost = false;
        }

        if !on_ground && on_wall {
            // check if movement direction is in the direction of the wall we are on
            if !state.heavy_fall
                && physics_context
                    .cast_ray(transform.translation, input.movement_dir, 1.0, false, filter)
                    .is_some()
            {
                if velocity.linvel.y < -1.0 {
                    velocity.linvel.x = velocity.linvel.x.clamp(-1.0, 1.0);
                    velocity.linvel.y = -2.0 * state.cling_fade;
                    velocity.linvel.z = velocity.linvel.z.clamp(-1.0, 1.0);
                    state.cling_fade = move_towards(state.cling_fade, 50.0, dt * 4.0);

                    shake.trauma = 0.25; // replace with sound
                }
            }

            if jump_requested && state.jump_cooldown.is_complete() && state.current_wall_jumps < 3 {
                state.jump_timer = controller.jump_time;
                state.jump_buffer_timer = 0.0;
                state.jumping = true;
                state.not_jumping_cooldown.reset();
                state.jump_cooldown.reset_with_duration(0.1);
                state.current_wall_jumps += 1;

                if state.heavy_fall {
                    state.slam_storage = true;
                }

                let jump_direction = (transform.translation - Vec3::NEG_Y - closest_pt).normalize();

                velocity.linvel.y = 0.0;
                velocity.linvel += Vec3::new(jump_direction.x, 1.0, jump_direction.z) * controller.wall_jump_speed;

                state.boost = false;
            }
        }

        if input.slide.pressed && on_ground && !state.sliding {
            // TODO: start_slide()
            state.sliding = true;
            state.boost = true;
            state.slide_safety_timer = 1.0;
            // TODO: move to crouch
        }

        // skip the ground slam if slide is pressed and we are near the ground
        if !on_ground && !state.sliding && !state.jumping && input.slide.pressed && near_ground_check.is_some() {
            // TODO: start_slide()
            state.sliding = true;
            state.boost = true;
            state.slide_safety_timer = 1.0;
            // TODO: move to crouch
        }

        if input.slide.released {
            // TODO: stop_slide()
            state.sliding = false;
            state.slide_ending_this_frame = true;
            state.slide_length = 0.0;
        }

        if state.sliding {
            state.slide_length += dt;

            // TODO: adjust crouching

            if state.slide_safety_timer > 0.0 {
                state.slide_safety_timer -= dt * 5.0;
            }

            if on_ground {
                shake.trauma = 0.2;
            }
        } else {
            // handle lerping from crouch to standing
        }

        if input.dash.pressed {
            if state.boost_charge > 100.0 {
                // TODO: stop_slide()
                state.sliding = false;
                state.slide_ending_this_frame = true;
                state.slide_length = 0.0;

                state.boost_left = state.boost_duration;
                state.dash_storage = 1.0;
                state.boost = true;
                state.boost_charge -= 100.0;

                if state.heavy_fall {
                    state.fall_speed = 0.0;
                    state.heavy_fall = false;
                }
            } else {
                // TODO: play sound, dont shake
                shake.trauma = 0.5;
            }
        }

        if state.boost_charge < 300.0 && !state.sliding {
            state.boost_charge = move_towards(state.boost_charge, 300.0, 70.0 * dt);
        }

        // FixedUpdate()
        if state.sliding && state.slide_safety_timer <= 0.0 {
            let ground_velocity = velocity.linvel.xz();
            if ground_velocity.length() < 10.0 {
                state.slide_safety_timer = move_towards(state.slide_safety_timer, -0.1, dt);
                if state.slide_safety_timer <= -0.1 {
                    // TODO: stop_slide()
                    state.sliding = false;
                    state.slide_ending_this_frame = true;
                    state.slide_length = 0.0;
                }
            } else {
                state.slide_safety_timer = 0.0;
            }
        }

        if !state.sliding {
            if state.heavy_fall {
                state.pre_slide_delay = 0.2;
                state.pre_slide_speed = state.slam_force;

                if let Some((_, toi)) = physics_context.cast_shape(
                    transform.translation,
                    transform.rotation,
                    velocity.linvel * Vec3::Y,
                    &cast_capsule, // smaller radius so we dont hit any walls
                    dt,
                    filter,
                ) {
                    transform.translation.y += velocity.linvel.y * toi.toi;
                    velocity.linvel = Vec3::ZERO;
                    state.super_jump_chance = 0.085;
                }
            } else if !state.boost && state.falling && velocity.linvel.length() / 24.0 > state.pre_slide_speed {
                state.pre_slide_delay = 0.2;
                state.pre_slide_speed = velocity.linvel.length() / 24.0;
            } else {
                state.pre_slide_delay = move_towards(state.pre_slide_delay, 0.0, dt);
                if state.pre_slide_delay <= 0.0 {
                    state.pre_slide_delay = 0.2;
                    state.pre_slide_speed = velocity.linvel.length() / 24.0;
                }
            }
        }

        // Move()
        if !state.boost {
            if on_ground && !state.jumping {
                state.current_wall_jumps = 0;

                let mut new_velocity = input.movement_dir * controller.walk_speed * dt;
                new_velocity.y = velocity.linvel.y - controller.gravity * dt;
                velocity.linvel = velocity.linvel.lerp(new_velocity, 0.25);
            } else {
                let wish_velocity = input.movement_dir * controller.walk_speed * dt;

                let mut air_dir = Vec3::ZERO;
                if (wish_velocity.x > 0.0 && velocity.linvel.x < wish_velocity.x)
                    || (wish_velocity.x < 0.0 && velocity.linvel.x > wish_velocity.x)
                {
                    air_dir.x = wish_velocity.x;
                }

                if (wish_velocity.z > 0.0 && velocity.linvel.z < wish_velocity.z)
                    || (wish_velocity.z < 0.0 && velocity.linvel.z > wish_velocity.z)
                {
                    air_dir.z = wish_velocity.z;
                }

                // TODO: this can maybe use acceleration method with quake with_vel system?
                let vel_y = velocity.linvel.y - controller.gravity * dt;
                velocity.linvel += air_dir.normalize_or_zero() * controller.air_acceleration * dt;
                velocity.linvel.y = vel_y;
            }
            return;
        }

        // Dodge()
        if state.sliding {
            let mut slide_multiplier = 1.0;
            if state.pre_slide_speed > 1.0 {
                state.pre_slide_speed = state.pre_slide_speed.min(3.0);
                slide_multiplier = state.pre_slide_speed;
                if on_ground {
                    state.pre_slide_speed -= state.pre_slide_speed * dt;
                }
                state.pre_slide_delay = 0.0;
            }

            if state.boost_left > 0.0 {
                state.dash_storage = move_towards(state.dash_storage, 0.0, dt);
                if state.dash_storage <= 0.0 {
                    state.boost_left = 0.0;
                }
            }

            // limit horizontal movement while sliding
            let mut new_velocity = input.dash_slide_dir * controller.slide_speed * slide_multiplier * dt;
            new_velocity.y = velocity.linvel.y;
            new_velocity += (input.movement.x * transform.right()).clamp_length_max(1.0) * 5.0;
            velocity.linvel = new_velocity + input.movement_dir;
        } else {
            let mut new_velocity = input.dash_slide_dir * controller.dash_speed * dt;
            new_velocity.y = if state.slide_ending_this_frame { velocity.linvel.y } else { 0.0 };

            // TODO: this results in the last frame of a slide getting a boost
            if !state.slide_ending_this_frame || (on_ground && !state.jumping) {
                velocity.linvel = new_velocity;
            }

            state.boost_left -= dt;
            if state.boost_left <= 0.0 {
                state.boost = false;
                // in the air and ran out of boost so reduce speed immediately
                if !on_ground && !state.slide_ending_this_frame {
                    new_velocity = input.dash_slide_dir * controller.walk_speed * dt;
                    velocity.linvel = new_velocity;
                }
            }
            state.slide_ending_this_frame = false;
        }

        if true {
            return;
        }

        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // ***** ***** ***** *****
        // old way
        // ***** ***** ***** *****
        let mut wish_speed = if input.dash.pressed {
            // TODO: make a fov_target var and always move towards the value. decrease fov for forward
            // perhaps it should be Target { default: T, current: T } with reset() and move_toward(value) -> T
            controller.dash_speed
        } else if state.sliding {
            controller.slide_speed
        } else {
            controller.walk_speed
        };

        if let Some((_, toi)) = ground_cast {
            let has_traction = Vec3::dot(toi.normal1, Vec3::Y) > controller.traction_normal_cutoff;

            // Only apply friction after at least one tick, allows b-hopping without losing speed
            if has_traction {
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
            }

            let mut add = acceleration(
                input.movement_dir,
                wish_speed,
                controller.acceleration,
                velocity.linvel,
                dt,
            );
            if !has_traction {
                add.y -= controller.gravity * dt;
            }
            velocity.linvel += add;

            if has_traction {
                let linvel = velocity.linvel;
                velocity.linvel -= Vec3::dot(linvel, toi.normal1) * toi.normal1;

                // if input.jump_was_pressed {
                //     velocity.linvel.y = controller.jump_speed;
                // }
            }
        } else {
            wish_speed = f32::min(wish_speed, controller.air_speed_cap);

            let mut add = acceleration(
                input.movement_dir,
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

        let crouch_speed = if input.dash.down { -controller.crouch_speed } else { controller.uncrouch_speed };
        controller.height += dt * crouch_speed;
        controller.height = controller.height.clamp(crouch_height, upright_height);

        if let Some(mut capsule) = collider.as_capsule_mut() {
            // capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5);
            capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5 * (controller.height - 1.0));
        }

        // Step offset
        if controller.step_offset > f32::EPSILON {
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

fn acceleration(wish_direction: Vec3, wish_speed: f32, acceleration: f32, velocity: Vec3, dt: f32) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
}

fn debug_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
        .single(world)
        .clone();

    // let entity = world.query_filtered::<Entity, With<FpsController>>().single(world);
    // egui::Window::new("FPS Player").show(egui_context.get_mut(), |ui| {
    //     egui::ScrollArea::vertical().show(ui, |ui| {
    //         bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
    //         ui.allocate_space(ui.available_size());
    //     });
    // });

    let mut state = world.query::<&mut FpsControllerState>().single_mut(world);
    egui::Window::new("State")
        .interactable(false)
        .title_bar(false)
        .fixed_pos(Pos2::ZERO)
        .auto_sized()
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.checkbox(&mut state.jumping, "jumping");
                ui.checkbox(&mut state.sliding, "sliding");
                ui.checkbox(&mut state.heavy_fall, "heavy_fall");
                ui.checkbox(&mut state.falling, "falling");
                ui.checkbox(&mut state.boost, "boost");
                ui.spacing();
                fn float_ui(ui: &mut egui::Ui, value: &mut f32, label: &str) {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        ui.add(DragValue::new(value));
                    });
                }
                float_ui(ui, &mut state.boost_charge, "boost_charge");
                float_ui(ui, &mut state.fall_time, "fall_time");
                float_ui(ui, &mut state.fall_speed, "fall_speed");
                float_ui(ui, &mut state.slam_force, "slam_force");
                ui.checkbox(&mut state.slam_storage, "slam_storage");
                float_ui(ui, &mut state.super_jump_chance, "super_jump_chance");
                float_ui(ui, &mut state.extra_jump_chance, "extra_jump_chance");
                ui.spacing();
                ui.label("Slide");
                float_ui(ui, &mut state.pre_slide_delay, "pre_slide_delay");
                float_ui(ui, &mut state.pre_slide_speed, "pre_slide_speed");
                float_ui(ui, &mut state.slide_safety_timer, "slide_safety_timer");
                float_ui(ui, &mut state.slide_length, "slide_length");
                ui.checkbox(&mut state.standing, "standing");
                ui.spacing();
                ui.label("Jump");
                float_ui(ui, &mut state.jump_cooldown.elapsed, "jump_cooldown.elapsed");
                ui.checkbox(&mut state.jump_cooldown.finished, "jump_cooldown.finished");
                ui.checkbox(
                    &mut state.not_jumping_cooldown.finished,
                    "not_jumping_cooldown.finished",
                );
                float_ui(ui, &mut state.jump_buffer_timer, "jump_buffer_timer");
                float_ui(ui, &mut state.coyote_timer, "coyote_timer");
                let mut tmp_wall_jumps = state.current_wall_jumps as f32;
                float_ui(ui, &mut tmp_wall_jumps, "current_wall_jumps");
                float_ui(ui, &mut state.cling_fade, "cling_fade");
            });
        });
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if f32::abs(target - current) <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}

/// projectile motion, get velocity required to launch an object from start to end. has issues...doesnt always reach the target.
/// revisit later for grapple hook thing or just fast teleport
#[allow(dead_code)]
fn calc_jump_velocity(start: Vec3, end: Vec3, gravity: f32) -> Vec3 {
    let mut trajectory_height = end.y - start.y - 0.1;
    if trajectory_height < 0.0 {
        trajectory_height = 2.0
    };
    let displacement_y = end.y - start.y;
    let displacement_xz = Vec3::new(end.x - start.x, 0.0, end.z - start.z);
    let velocity = Vec3::Y * f32::sqrt(2.0 * gravity * trajectory_height);

    let velocity_xz = displacement_xz / f32::sqrt(2.0 * trajectory_height / gravity)
        + f32::sqrt(2.0 * (displacement_y - trajectory_height) / gravity);
    velocity_xz + velocity
}
