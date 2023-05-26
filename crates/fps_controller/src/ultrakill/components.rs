use bevy::prelude::*;

use crate::math::move_towards;

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