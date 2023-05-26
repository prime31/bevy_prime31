use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    input::mouse::MouseWheel,
    math::Vec3Swizzles,
    prelude::{
        default, AmbientLight, App, AssetPlugin, AssetServer, Camera3dBundle, Color, Commands, PluginGroup, Res,
        Transform, *,
    },
    render::{camera::Viewport, view::RenderLayers},
    window::CursorGrabMode,
    DefaultPlugins,
};

use bevy_rapier3d::prelude::*;

use debug_text::DebugTextPlugin;
use egui_helper::EguiHelperPlugin;
use fps_controller::{
    camera_shake::*,
    input::{FpsInputPlugin, FpsPlayer, RenderPlayer},
    time_controller::TimeManagerPlugin,
    ultrakill::{FpsController, FpsControllerState, UltrakillControllerPlugin}, math::map,
};
use valve_maps::bevy::{ValveMapBundle, ValveMapPlugin};

#[derive(Component)]
struct TextMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_plugin(ValveMapPlugin)
        .add_plugin(EguiHelperPlugin)
        .add_plugin(DebugTextPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsInputPlugin)
        .add_plugin(UltrakillControllerPlugin)
        .add_plugin(CameraShakePlugin)
        .add_plugin(TimeManagerPlugin)
        .add_startup_system(setup_scene)
        .add_systems((print_collision_events, display_text, manage_cursor, zoom_2nd_camera))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("Plane"),
        Collider::cuboid(140.0, 0.1, 140.0),
        Restitution::coefficient(1.0),
        TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
    ));

    commands
        .spawn((
            (
                Name::new("Player"),
                FpsPlayer,
                FpsController::default(),
                FpsControllerState::new(),
                valve_maps::bevy::ValveMapPlayer,
                RenderLayers::layer(1),
            ),
            PbrBundle {
                mesh: meshes.add(shape::Capsule::default().into()),
                material: materials.add(Color::rgb(0.8, 0.1, 0.9).into()),
                ..Default::default()
            },
            Collider::capsule_y(0.5, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
        ))
        .with_children(|builder| {
            // example of a wall check sensor collider. not sure if this is better than just doing a shape_cast yet.
            // let id = builder
            //     .spawn((
            //         Collider::cylinder(0.4, 0.6),
            //         Sensor,
            //         ActiveEvents::COLLISION_EVENTS,
            //         Transform::from_xyz(0.0, 0.0, 0.0),
            //     ))
            //     .id();
            // print!("---- cylinder: {:?}", id);

            builder
                .spawn((Shake3d::default(), SpatialBundle::default()))
                .with_children(|builder| {
                    builder
                        .spawn((
                            RenderPlayer,
                            Camera3dBundle {
                                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                                projection: Projection::Perspective(PerspectiveProjection {
                                    fov: 100.0_f32.to_radians(),
                                    ..default()
                                }),
                                ..default()
                            },
                            RenderLayers::default().without(1), // all but our LogicalPlayer
                        ))
                        .with_children(|builder| {
                            // Right Camera for 3rd person view trailing a bit and slightly above the player
                            let win_w = 1280;
                            let frame_w = 256;
                            let frame_h = 256 / (1280 / 720);
                            builder.spawn((
                                Name::new("Camera Two"),
                                Camera3dBundle {
                                    transform: Transform::from_xyz(0.0, 0.0, 15.0),
                                    camera: Camera {
                                        order: 1, // after other camera
                                        viewport: Some(Viewport {
                                            physical_position: UVec2::new(win_w * 2 - frame_w * 2, 0),
                                            physical_size: UVec2::new(frame_w * 2, frame_h * 2),
                                            ..default()
                                        }),
                                        ..default()
                                    },
                                    camera_3d: Camera3d {
                                        clear_color: ClearColorConfig::None,
                                        ..default()
                                    },
                                    ..default()
                                },
                                UiCameraConfig { show_ui: false },
                                RenderLayers::default().with(1),
                            ));
                        });
                });
        });

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 14.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        TextMarker
    ));

    commands.spawn(ValveMapBundle {
        map: asset_server.load("playground.map"),
        ..Default::default()
    });
}

fn print_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for _collision_event in collision_events.iter() {
        // println!("Received collision event: {:?}", _collision_event);
    }

    for _contact_force_event in contact_force_events.iter() {
        // println!("Received contact force event: {:?}", _contact_force_event);
    }
}

fn manage_cursor(
    key: Res<Input<KeyCode>>,
    btn: Res<Input<MouseButton>>,
    egui_state: Res<egui_helper::EguiHelperState>,
    mut window_query: Query<&mut Window>,
) {
    let mut window = window_query.single_mut();

    if !egui_state.wants_input && !egui_state.enabled {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        }
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn display_text(mut controller_query: Query<&Velocity>, mut text_query: Query<&mut Text, With<TextMarker>>) {
    for velocity in &mut controller_query {
        for mut text in &mut text_query {
            text.sections[0].value = format!(
                "vel: {:.2}, {:.2}, {:.2}\nspeed: {:.2}\nxz speed: {:.2}",
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.linvel.z,
                velocity.linvel.length(),
                velocity.linvel.xz().length()
            );
        }
    }
}

fn zoom_2nd_camera(
    egui_state: Res<egui_helper::EguiHelperState>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut q: Query<(&mut Projection, &mut Transform), Without<RenderPlayer>>,
) {
    if egui_state.wants_input {
        return;
    }
    let scroll = ev_scroll.iter().fold(0.0, |val, evt| val + evt.y);
    if scroll == 0.0 {
        return;
    }

    let Ok((mut proj, mut tf)) = q.get_single_mut() else { return };
    if let Projection::Perspective(proj) = proj.as_mut() {
        proj.fov = (proj.fov + scroll * 0.02).clamp(10.0_f32.to_radians(), 100.0_f32.to_radians());

        // map the lower range of fov to camera height so it tends towards -1.0 (ground) when zoomed in
        let desired_y = map(proj.fov, 0.17, 0.4, -1.0, 0.0).clamp(-1.0, 0.0);
        tf.translation.y = desired_y;
    }
}
