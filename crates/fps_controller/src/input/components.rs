use bevy::prelude::*;
use leafwing_input_manager::{Actionlike, prelude::ActionState};

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsPlayer;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum InputAction {
    Move,
    MouseLook,
    ControllerLook,
    Jump,
    Slide,
    Dash,
    Shoot,
}

pub type InputActions = ActionState<InputAction>;

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
    pub shoot: InputState,
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
    pub movement_dir: Vec3,
    pub dash_slide_dir: Vec3,
    // move these to some state struct
    pub vel: Vec3,
}

#[derive(Component, Reflect)]
pub struct FpsControllerInputConfig {
    pub mouse_sensitivity: f32,
    pub gamepad_sensitivity: f32,
}

impl Default for FpsControllerInputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.7,
            gamepad_sensitivity: 3.0,
        }
    }
}
