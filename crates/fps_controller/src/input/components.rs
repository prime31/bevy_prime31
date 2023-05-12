use bevy::prelude::*;

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
