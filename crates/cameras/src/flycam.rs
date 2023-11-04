use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub struct FlycamPlugin;

impl Plugin for FlycamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_camera)
            .add_systems(Update, camera_movement)
            .add_systems(Update, camera_look)
            .add_systems(Update, toggle_cursor);
    }
}

#[derive(Component)]
pub struct FlycamControls {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub enable_movement: bool,
    pub enable_look: bool,

    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_boost: KeyCode,
}

impl Default for FlycamControls {
    fn default() -> Self {
        Self {
            yaw: Default::default(),
            pitch: Default::default(),
            sensitivity: 25.0,
            enable_movement: true,
            enable_look: true,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_boost: KeyCode::ShiftLeft,
        }
    }
}

fn spawn_camera(mut commands: Commands, query: Query<(Entity, &Camera)>) {
    for (entity, camera) in query.iter() {
        if camera.order != 0 {
            continue;
        }

        commands.entity(entity).insert(FlycamControls::default());
    }
}

fn camera_movement(
    mut cam: Query<(&FlycamControls, &mut Transform)>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (flycam, mut cam_transform) = cam.single_mut();
    if !flycam.enable_movement {
        return;
    }

    let if_then_1 = |b| if b { 1.0 } else { 0.0 };
    let forward =
        if_then_1(keyboard_input.pressed(flycam.key_forward)) - if_then_1(keyboard_input.pressed(flycam.key_back));
    let sideways =
        if_then_1(keyboard_input.pressed(flycam.key_right)) - if_then_1(keyboard_input.pressed(flycam.key_left));
    let up = if_then_1(keyboard_input.pressed(flycam.key_up)) - if_then_1(keyboard_input.pressed(flycam.key_down));

    if forward == 0.0 && sideways == 0.0 && up == 0.0 {
        return;
    }

    let speed = if keyboard_input.pressed(flycam.key_boost) { 20.0 } else { 5.0 };

    let movement = Vec3::new(sideways, forward, up).normalize_or_zero() * speed * time.delta_seconds();

    let diff =
        cam_transform.forward() * movement.y + cam_transform.right() * movement.x + cam_transform.up() * movement.z;
    cam_transform.translation += diff;
}

fn camera_look(
    time: Res<Time>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut query: Query<(&mut FlycamControls, &mut Transform)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut flycam, mut transform) = query.single_mut();
    let window = window_q.get_single().unwrap();

    if !mouse_input.pressed(MouseButton::Left) && window.cursor.grab_mode != CursorGrabMode::Confined {
        return;
    }
    if !flycam.enable_look {
        return;
    }

    // on mouse down copy the data to the flycam from the camera rotation
    if mouse_input.just_pressed(MouseButton::Left) {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        flycam.yaw = yaw.to_degrees();
        flycam.pitch = pitch.to_degrees();
    }

    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event_reader.read() {
        delta += event.delta;
    }
    if delta.is_nan() || delta.abs_diff_eq(Vec2::ZERO, f32::EPSILON) {
        return;
    }

    flycam.yaw -= delta.x * flycam.sensitivity * time.delta_seconds();
    flycam.pitch -= delta.y * flycam.sensitivity * time.delta_seconds();

    flycam.pitch = flycam.pitch.clamp(-89.0, 89.9);
    // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

    let yaw_radians = flycam.yaw.to_radians();
    let pitch_radians = flycam.pitch.to_radians();

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw_radians, pitch_radians, 0.0);
}

fn toggle_cursor(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut q: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = q.get_single_mut().unwrap();

    let mut set_focus = |focused: bool| {
        let grab_mode = match focused {
            true => CursorGrabMode::Confined,
            false => CursorGrabMode::None,
        };
        window.cursor.grab_mode = grab_mode;
        window.cursor.visible = !focused;
    };

    if keys.just_pressed(KeyCode::Escape) {
        set_focus(false);
    } else if buttons.just_pressed(MouseButton::Right) {
        set_focus(true);
    }
}
