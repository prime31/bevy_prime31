use bevy::prelude::*;

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsPlayer;

#[derive(Component, Default, Reflect)]
pub struct FpsControllerInput {
    pub sprint: bool,
    pub jump_pressed: bool,
    pub jump_down: bool,
    pub dash_pressed: bool,
    pub dash_down: bool,
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
    pub key_sprint: KeyCode,
    pub key_dash: KeyCode,
    pub key_jump: KeyCode,
    pub key_fly: KeyCode,
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
            key_sprint: KeyCode::LShift,
            key_dash: KeyCode::E,
            key_jump: KeyCode::Space,
            key_fly: KeyCode::F,
        }
    }
}
