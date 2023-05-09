use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use egui_helper::EguiHelperState;

#[derive(Default)]
pub struct FpsInputPlugin;

impl Plugin for FpsInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FpsControllerInput>()
            .register_type::<FpsControllerInputConfig>()
            .add_system(setup.on_startup().in_base_set(StartupSet::PostStartup))
            .add_system(controller_input)
            .add_system(calculate_movement)
            .add_system(sync_render_player);
    }
}

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsPlayer;

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
            sensitivity: 0.7,
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

fn setup(mut commands: Commands, q: Query<Entity, With<FpsPlayer>>) {
    for entity in q.iter() {
        commands
            .entity(entity)
            .insert((FpsControllerInput::default(), FpsControllerInputConfig::default()));
    }
}

fn controller_input(
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

        input.movement = Vec3::new(
            get_axis(&key_input, controller.key_right, controller.key_left),
            get_axis(&key_input, controller.key_up, controller.key_down),
            get_axis(&key_input, controller.key_forward, controller.key_back),
        );
    }
}

fn calculate_movement(
    time: Res<Time>,
    query: Query<&FpsControllerInput>,
    render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    // TODO: should this handle doing basic integration of input + frictions/accelerations?
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
    egui_state: Res<EguiHelperState>,
    logical_query: Query<&FpsControllerInput, With<FpsPlayer>>,
    mut render_query: Query<&mut Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    if egui_state.wants_input {
        return;
    };

    for controller in logical_query.iter() {
        for mut tf in render_query.iter_mut() {
            let euler = tf.rotation.to_euler(EulerRot::YXZ);

            let mut yaw = euler.0 - controller.yaw;
            let pitch = (euler.1 - controller.pitch).clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
            if yaw.abs() > PI {
                yaw = yaw.rem_euclid(TAU);
            }

            tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        }
    }
}
