//! # bevy_spectator
//!
//! A spectator camera plugin for the [Bevy game engine](https://bevyengine.org/).
//!
//! ## Controls
//!
//! |Action|Key|
//! |-|-|
//! |Forward|`W`|
//! |Left|`A`|
//! |Backward|`S`|
//! |Right|`D`|
//! |Up|`Space`|
//! |Down|`LControl`|
//! |Alt. Speed|`LShift`|
//! |Release Cursor|`Escape`|
//!
//! Movement is constrained to the appropriate axes. (`WASD` to X & Z axes, `Space` & `LShift` to the Y axis)
//!
//! ## `basic` Example
//! ```
//! use bevy::prelude::*;
//! use bevy_spectator::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(SpectatorPlugin)
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         Camera3dBundle::default(), Spectator
//!     ));
//! }
//! ```

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

/// A marker `Component` for spectating cameras.
///
/// ## Usage
/// With the `init` feature:
/// - Add it to a single entity to mark it as a spectator.
/// - `init` will then find that entity and mark it as the active spectator in [`SpectatorSettings`].
///
/// (If there isn't a single [`Spectator`] (none or multiple, instead of one), there won't be an active spectator selected by the `init` feature.)
///
/// Without the `init` feature:
/// - Add it to entities to mark spectators.
/// - Manually alter [`SpectatorSettings`] to set the active spectator.
#[derive(Component)]
pub struct Spectator;

/// A `Plugin` for spectating your scene.
pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpectatorSettings>()
            .add_startup_system(setup.in_base_set(StartupSet::PostStartup))
            .add_system(spectator_update);
    }
}

fn setup(mut commands: Commands, query: Query<Entity, With<Camera>>) {
    let entity = query.get_single().unwrap();

    commands.insert_resource(SpectatorSettings {
        active_spectator: Some(entity),
        ..Default::default()
    });

    commands.entity(entity).insert(Spectator);
}

fn spectator_update(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut settings: ResMut<SpectatorSettings>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    mut camera_transforms: Query<&mut Transform, With<Spectator>>,
    added: Query<Entity, Added<Spectator>>,
    mut focus: Local<bool>,
) {
    for entity in added.iter() {
        settings.active_spectator = Some(entity);
    }

    let mut window = q_windows.get_single_mut().unwrap();

    let Some(camera_id) = settings.active_spectator else {
        motion.clear();
        return;
    };

    let Ok(mut camera_transform) = camera_transforms.get_mut(camera_id) else {
        error!("Failed to find camera for active camera entity ({camera_id:?})");
        settings.active_spectator = None;
        motion.clear();
        return;
    };

    let mut set_focus = |focused: bool| {
        *focus = focused;
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

    if *focus {
        // rotation
        {
            let mouse_delta = {
                let mut total = Vec2::ZERO;
                for d in motion.iter() {
                    total += d.delta;
                }
                total
            };

            let mouse_x = -mouse_delta.x * time.delta_seconds() * settings.sensitivity;
            let mouse_y = -mouse_delta.y * time.delta_seconds() * settings.sensitivity;

            let mut dof: Vec3 = camera_transform.rotation.to_euler(EulerRot::YXZ).into();

            dof.x += mouse_x;
            // At 90 degrees, yaw gets misinterpeted as roll. Making 89 the limit fixes that.
            dof.y = (dof.y + mouse_y).clamp(-89f32.to_radians(), 89f32.to_radians());
            dof.z = 0f32;

            camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, dof.x, dof.y, dof.z);
        }

        // translation
        {
            let forward = if keys.pressed(KeyCode::W) { 1f32 } else { 0f32 };
            let backward = if keys.pressed(KeyCode::S) { 1f32 } else { 0f32 };
            let right = if keys.pressed(KeyCode::D) { 1f32 } else { 0f32 };
            let left = if keys.pressed(KeyCode::A) { 1f32 } else { 0f32 };
            let up = if keys.pressed(KeyCode::E) { 1f32 } else { 0f32 };
            let down = if keys.pressed(KeyCode::Q) { 1f32 } else { 0f32 };

            let speed = if keys.pressed(KeyCode::LShift) {
                settings.alt_speed
            } else {
                settings.base_speed
            };

            let delta_axial = (forward - backward) * speed;
            let delta_lateral = (right - left) * speed;
            let delta_vertical = (up - down) * speed;

            let mut forward = camera_transform.forward();
            forward.y = 0f32;
            let mut right = camera_transform.right();
            right.y = 0f32; // more of a sanity check
            let up = Vec3::Y;

            camera_transform.translation +=
                forward * delta_axial + right * delta_lateral + up * delta_vertical;
        }
    }

    motion.clear();
}

/// A `Resource` for controlling [`Spectator`]s.
#[derive(Resource)]
pub struct SpectatorSettings {
    /// The `Entity` of the active [`Spectator`]. (Default: `None`)
    ///
    /// Use this to control which [`Spectator`] you are using.
    pub active_spectator: Option<Entity>,
    /// The `WindowID` of the active `Window`. (Default: `Some(WindowId::primary())`)
    ///
    /// Use this to control which `Window` will grab your mouse/hide the cursor.
    // pub active_window: Option<WindowId>,
    /// The base speed of the active [`Spectator`]. (Default: `0.1`)
    ///
    /// Use this to control how fast the [`Spectator`] normally moves.
    pub base_speed: f32,
    /// The alternate speed of the active [`Spectator`]. (Default: `0.5`)
    ///
    /// Use this to control how fast the [`Spectator`] moves when you hold `Sprint`.
    pub alt_speed: f32,
    /// The camera sensitivity of the active [`Spectator`]. (Default: `0.1`)
    ///
    /// Use this to control how fast the [`Spectator`] turns when you move the mouse.
    pub sensitivity: f32,
}

impl Default for SpectatorSettings {
    fn default() -> Self {
        Self {
            active_spectator: None,
            base_speed: 0.1,
            alt_speed: 0.5,
            sensitivity: 0.16,
        }
    }
}
