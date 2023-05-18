use bevy::prelude::*;

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsPlayer;

#[derive(Default, Reflect)]
pub struct InputState {
    pub pressed: bool,
    pub down: bool,
    pub released: bool,
}

#[derive(Component, Default, Reflect)]
pub struct FpsControllerInput {
    pub jump: InputState,
    pub slide: InputState,
    pub dash: InputState,
    pub pitch: f32,
    pub yaw: f32,
    pub tilt: f32,
    pub movement: Vec3,
    pub movement_dir: Vec3,
    pub dodge_slide_dir: Vec3,
    // move these to some state struct
    pub vel: Vec3,
}

#[derive(Component, Reflect)]
pub struct FpsControllerInputConfig {
    pub enable_input: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_slide: KeyCode,
    pub key_dash: KeyCode,
    pub key_jump: KeyCode,
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
            key_slide: KeyCode::LControl,
            key_dash: KeyCode::LShift,
            key_jump: KeyCode::Space,
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct FpsControllerState {}