use crate::input::{FpsControllerInput, FpsControllerStages};
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;
use egui_helper::bevy_inspector_egui::{self, bevy_egui::EguiContext, egui};

#[derive(Default)]
pub struct UltrakillControllerPlugin;

impl Plugin for UltrakillControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(controller_move.in_set(FpsControllerStages::Logic))
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
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub slide_acceleration: f32,
    pub ground_slam_speed: f32,
    pub max_fall_velocity: f32,
    pub friction: f32,
    /// If the dot product (alignment) of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, ground movement is applied
    pub traction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub jump_speed: f32,
    pub wall_jump_speed: f32,
    pub fly_speed: f32,
    pub crouch_speed: f32,
    pub uncrouch_speed: f32,
    pub height: f32,
    pub upright_height: f32,
    pub crouch_height: f32,
    pub fast_fly_speed: f32,
    pub fly_friction: f32,
    pub ground_tick: u8,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub enable_input: bool,
    pub step_offset: f32,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            radius: 0.5,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 9.0,
            slide_speed: 25.0,
            dash_speed: 144.0,
            forward_speed: 30.0,
            side_speed: 50.0,
            air_speed_cap: 2.0,
            air_acceleration: 20.0,
            ground_slam_speed: -100.0,
            max_fall_velocity: -100.0,
            max_air_speed: 15.0,
            crouch_speed: 50.0,
            uncrouch_speed: 8.0,
            height: 1.0,
            upright_height: 2.0,
            crouch_height: 1.0,
            acceleration: 10.0,
            slide_acceleration: 1.0,
            friction: 10.0,
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            fly_friction: 0.5,
            ground_tick: 0,
            stop_speed: 1.0,
            jump_speed: 10.5,
            wall_jump_speed: 10.0,
            step_offset: 0.0,
            enable_input: true,
            sensitivity: 0.005,
        }
    }
}

#[derive(Default)]
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

#[derive(Component, Default)]
pub struct FpsControllerState {
    pub jumping: bool,
    pub sliding: bool,
    pub dashing: bool,
    pub ground_slamming: bool,
    pub heavy_fall: bool,
    pub falling: bool,
    pub fall_time: f32,
    pub fall_speed: f32,
    // slide
    pub slide_safety_timer: f32,
    pub slide_length: f32,
    pub standing: bool,
    // jump/wall jump
    pub jump_cooldown: CooldownTimer,
    pub not_jumping_cooldown: CooldownTimer,
    pub current_wall_jumps: u8,
    pub cling_fade: f32,
}

impl FpsControllerState {
    pub fn new() -> Self {
        Self {
            jump_cooldown: CooldownTimer::new(0.2),
            not_jumping_cooldown: CooldownTimer::new(0.25),
            ..Default::default()
        }
    }

    pub fn tick_timers(&mut self, dt: f32) {
        self.jump_cooldown.tick(dt);
        self.not_jumping_cooldown.tick(dt);

        if self.not_jumping_cooldown.finished_this_tick {
            self.jumping = false;
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
    collider_q: Query<&Collider, Without<FpsController>>,
    transform_q: Query<&Transform, Without<FpsController>>,
) {
    let dt = time.delta_seconds();

    for (entity, input, mut state, mut controller, mut collider, mut transform, mut velocity) in query.iter_mut() {
        let Some(capsule) = collider.as_capsule() else { return };

        state.tick_timers(dt);

        // Capsule cast downwards to find ground
        let capsule = capsule.raw;
        let cast_capsule = Collider::capsule(capsule.segment.a.into(), capsule.segment.b.into(), capsule.radius * 0.9);

        let filter = QueryFilter::default().exclude_rigid_body(entity).exclude_sensors();
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
        let filter = QueryFilter::default().exclude_rigid_body(entity).exclude_sensors();
        let mut on_wall = false;
        let mut closest_pt = Vec3::splat(f32::MAX);
        let mut closest_dist = f32::MAX;
        physics_context.intersections_with_shape(
            transform.translation,
            transform.rotation,
            &cast_cylinder,
            filter,
            |entity| {
                let collider = collider_q
                    .get(entity)
                    .expect("Collider not found for intersected entity");
                let collider = collider
                    .as_convex_polyhedron()
                    .expect("Collider is not a ConvexPolyhedron");

                let player_pos = transform.translation;
                let collider_pos = transform_q.get(entity).unwrap().translation;
                collider.points().for_each(|pt| {
                    let dist = player_pos.distance_squared(collider_pos + pt);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_pt = collider_pos + pt;
                    }
                });

                on_wall = true;
                true
            },
        );

        if on_ground {
            state.fall_time = 0.0;
            state.cling_fade = 0.0;
        } else {
            if state.fall_time < 1.0 {
                state.fall_time += dt * 5.0; // TODO: wtf? 5?
                if state.fall_time > 1.0 {
                    state.falling = true;
                }
            } else if velocity.linvel.y < -2.0 {
                state.fall_speed = velocity.linvel.y;
            }
        }

        // clamp max velocity
        if velocity.linvel.y < controller.max_fall_velocity {
            velocity.linvel.y = controller.max_fall_velocity;
        }

        // falling and hit ground this frame
        if on_ground && state.falling && state.jump_cooldown.is_complete() {
            state.falling = false;
            state.fall_speed = 0.0;
            state.heavy_fall = false;
        }

        if !on_ground && input.slide.pressed {
            // if state.sliding { stop_slide() }

            // if (fallTime > 0.5f && !Physics.Raycast(gc.transform.position + base.transform.up, base.transform.up * -1f, out var _, 3f, lmask) && !gc.heavyFall)
            if state.fall_time > 0.5 && !state.heavy_fall {
                velocity.linvel = Vec3::new(0.0, controller.ground_slam_speed, 0.0);
                state.falling = true;
                state.fall_speed = controller.ground_slam_speed;
                state.heavy_fall = true;
            }
        }

        if state.heavy_fall {
            velocity.linvel = Vec3::new(0.0, controller.ground_slam_speed, 0.0);
        }

        if input.jump.pressed && !state.falling && on_ground && state.jump_cooldown.is_complete() {
            state.current_wall_jumps = 0;
            state.cling_fade = 0.0;
            state.jumping = true;
            state.falling = true;
            state.not_jumping_cooldown.reset();

            velocity.linvel.y = 0.0;
            if state.sliding {
                // if state.sliding { stop_slide() }
                state.sliding = false;
                velocity.linvel.y = controller.jump_speed * 0.8;
            } else {
                velocity.linvel.y = controller.jump_speed;
            }

            state.jump_cooldown.reset_with_duration(0.25);
        }

        if !on_ground && on_wall {
            if physics_context
                .cast_ray(transform.translation, input.movement_dir, 1.0, false, filter)
                .is_some()
            {
                if velocity.linvel.y < -1.0 && !state.heavy_fall {
                    if velocity.linvel.y < -1.0 && !state.heavy_fall {
                        velocity.linvel.x = velocity.linvel.x.clamp(-1.0, 1.0);
                        velocity.linvel.y = -2.0 * state.cling_fade;
                        velocity.linvel.z = velocity.linvel.z.clamp(-1.0, 1.0);
                        state.cling_fade = move_towards(state.cling_fade, 50.0, dt * 4.0);
                    }
                }
            }

            if input.jump.pressed && state.jump_cooldown.is_complete() && state.current_wall_jumps < 3 {
                state.jumping = true;
                state.not_jumping_cooldown.reset();
                state.jump_cooldown.reset_with_duration(0.1);
                state.current_wall_jumps += 1;

                velocity.linvel = Vec3::ZERO;
                let jump_pos = (transform.translation - closest_pt).normalize();
                velocity.linvel = Vec3::new(jump_pos.x, 1.0, jump_pos.z) * controller.wall_jump_speed;
            }
        }

        //if (gc.onGround && activated &&  && !sliding)
        if input.slide.pressed && on_ground && !state.sliding {
            state.sliding = true;
        }

        if !on_ground
            && !state.sliding
            && !state.jumping
            && physics_context
                .cast_ray(transform.translation, Vec3::NEG_Y, 2.0, false, filter)
                .is_some()
        {
            state.sliding = true;
        }

        if input.slide.released {
            state.sliding = false;
        }

        if state.sliding {
            state.slide_length += dt;

            if state.slide_safety_timer > 0.0 {
                state.slide_safety_timer -= dt * 5.0;
            }

            if on_ground {
                // camera shake
            }
        } else {
            // handle lerping from crouch to standin
        }

        // ***** ***** ***** *****
        // old way
        let mut wish_speed = if input.dash.pressed {
            controller.dash_speed
        } else if input.slide.down {
            controller.slide_speed
        } else {
            controller.walk_speed
        };

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

            // start slide
            if input.slide.pressed {
                state.sliding = true;
                state.slide_safety_timer = 1.0;
                // TODO: `if !crouching` set collider size
            }

            if input.slide.down {
                let mut base_dir = input.dash_slide_dir * controller.slide_speed;
                base_dir.y = velocity.linvel.y;
                base_dir += (input.movement.x * transform.right()).clamp_length_max(1.0) * 5.0;
                velocity.linvel = base_dir;
                return;
                // TODO: still handle jumping
            }

            // stop slide
            if input.slide.released {
                state.sliding = false;
                state.slide_length = 0.0;
            }

            if input.dash.pressed {
                state.dashing = true;
                let base_dir = input.dash_slide_dir * controller.dash_speed;
                // if slide_ending { base_dir.y = velocity.linvel.y; }
                velocity.linvel = base_dir;
                return;
                // TODO: still handle jumping
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

                if input.jump.pressed {
                    velocity.linvel.y = controller.jump_speed;
                }
            }

            // Increment ground tick but cap at max value
            controller.ground_tick = controller.ground_tick.saturating_add(1);
        } else {
            // ground slam
            if input.slide.pressed {
                state.ground_slamming = true;
                velocity.linvel = Vec3::new(0.0, controller.ground_slam_speed, 0.0);
                return;
            }

            controller.ground_tick = 0;
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

    let entity = world.query_filtered::<Entity, With<FpsController>>().single(world);

    egui::Window::new("Hello").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
            ui.allocate_space(ui.available_size());
        });
    });
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}
