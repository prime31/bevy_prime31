use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    math::Vec3Swizzles,
    prelude::{
        default, AmbientLight, App, AssetPlugin, AssetServer, Camera3dBundle, Color, Commands, PluginGroup, Res,
        Transform, Vec3, *,
    },
    render::{camera::Viewport, view::RenderLayers},
    window::CursorGrabMode,
    DefaultPlugins,
};

use bevy_rapier3d::prelude::*;

use egui_helper::EguiHelperPlugin;
use fps_controller::{
    input::{FpsInputPlugin, FpsPlayer, RenderPlayer},
    ultrakill::{FpsController, UltrakillControllerPlugin},
};
use valve_maps::bevy::{ValveMapBundle, ValveMapPlugin};

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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsInputPlugin)
        .add_plugin(UltrakillControllerPlugin)
        .add_startup_system(setup_scene)
        .add_systems((print_collision_events, display_text, manage_cursor))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands
        .spawn(Collider::cuboid(140.0, 0.1, 140.0))
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    commands
        .spawn((
            (FpsPlayer, valve_maps::bevy::ValveMapPlayer, RenderLayers::layer(1)),
            PbrBundle {
                mesh: meshes.add(shape::Capsule::default().into()),
                material: materials.add(Color::rgb(0.8, 0.1, 0.9).into()),
                ..Default::default()
            },
            Collider::capsule(Vec3::Y * -0.5, Vec3::Y * 0.5, 0.5),
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
            FpsController { ..default() },
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    RenderPlayer,
                    Camera3dBundle {
                        transform: Transform::from_xyz(0.0, 1.0, 0.0),
                        projection: Projection::Perspective(PerspectiveProjection { fov: 100.0_f32.to_radians(), ..default() }),
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
                        Camera3dBundle {
                            transform: Transform::from_xyz(0., 1.5, 15.0),
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

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 24.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );

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
    mut controller_query: Query<&mut FpsController>,
) {
    let mut window = window_query.single_mut();

    if !egui_state.wants_input && !egui_state.enabled {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
            for mut controller in &mut controller_query {
                controller.enable_input = true;
            }
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

fn display_text(mut controller_query: Query<(&Transform, &Velocity)>, mut text_query: Query<&mut Text>) {
    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.sections[0].value = format!(
                "vel: {:.2}, {:.2}, {:.2}\nspeed: {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}",
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.linvel.z,
                velocity.linvel.length(),
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.linvel.xz().length()
            );
        }
    }
}
