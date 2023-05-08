use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_rapier3d::prelude::Collider;

#[derive(Default)]
pub struct FpsInputPlugin;

impl Plugin for FpsInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FpsControllerInput>()
            .register_type::<FpsControllerInputConfig>()
            .add_system(setup.on_startup().in_base_set(StartupSet::PostStartup))
            .add_system(controller_input)
            .add_system(sync_render_player);
    }
}

#[derive(Component)]
pub struct LogicalPlayer;

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component, Default, Reflect)]
pub struct FpsControllerInput {
    pub fly: bool,
    pub sprint: bool,
    pub jump: bool,
    pub crouch: bool,
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
}

#[derive(Component, Reflect)]
pub struct FpsControllerInputConfig {
    pub enable_input: bool,
    pub sensitivity: f32,
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

impl Default for FpsControllerInputConfig {
    fn default() -> Self {
        Self {
            enable_input: true,
            sensitivity: 0.005,
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
        }
    }
}

const ANGLE_EPSILON: f32 = 0.001953125;

fn setup(mut commands: Commands, q: Query<Entity, With<LogicalPlayer>>) {
    for entity in q.iter() {
        commands
            .entity(entity)
            .insert((FpsControllerInput::default(), FpsControllerInputConfig::default()));
    }
}

fn controller_input(
    key_input: Res<Input<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&FpsControllerInputConfig, &mut FpsControllerInput)>,
) {
    for (controller, mut input) in query.iter_mut() {
        if !controller.enable_input {
            continue;
        }

        let mut mouse_delta: Vec2 = mouse_events
            .iter()
            .fold(Vec2::ZERO, |collector, evt| collector + evt.delta);
        mouse_delta *= controller.sensitivity;

        input.pitch = (input.pitch - mouse_delta.y).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
        input.yaw -= mouse_delta.x;
        if input.yaw.abs() > PI {
            input.yaw = input.yaw.rem_euclid(TAU);
        }

        input.movement = Vec3::new(
            get_axis(&key_input, controller.key_right, controller.key_left),
            get_axis(&key_input, controller.key_up, controller.key_down),
            get_axis(&key_input, controller.key_forward, controller.key_back),
        );
        input.sprint = key_input.pressed(controller.key_sprint);
        input.jump = key_input.just_pressed(controller.key_jump);
        input.fly = key_input.just_pressed(controller.key_fly);
        input.crouch = key_input.pressed(controller.key_crouch);
    }
}

#[allow(dead_code)]
fn manage_cursor(
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

pub fn sync_render_player(
    logical_query: Query<(&Transform, &Collider, &FpsControllerInput), With<LogicalPlayer>>,
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
