use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use dolly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DollyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_instructions)
        .add_systems(Update, tick)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 15.0,
            subdivisions: 10,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cubes
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::DARK_GREEN)),
        transform: Transform::from_xyz(0., 0.5, -6.),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::BLUE)),
        transform: Transform::from_xyz(0., 0.5, 6.),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::YELLOW)),
        transform: Transform::from_translation(Vec3::new(2., 0.5, 2.)),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::WHITE)),
        transform: Transform::from_xyz(-5., 0.5, -2.),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::WHITE)),
        transform: Transform::from_xyz(-5., 0.5, 2.),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::BLACK)),
        transform: Transform::from_xyz(2., 0.5, -2.),
        ..default()
    });

    // torus
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Torus::default())),
        material: materials.add(StandardMaterial::from(Color::VIOLET)),
        transform: Transform::from_xyz(0., 2., -9.),
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.2,
    });

    // point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera rig
    let camera_rig: CameraRig = CameraRig::builder()
        .with(Position::new(Vec3::Y))
        .with(YawPitch::new())
        .with(Smooth::new_position_rotation(1.0, 1.0))
        .build();

    // camera
    commands.spawn((Camera3dBundle::default(), camera_rig));
}

fn setup_instructions(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((TextBundle::from_section(
        "WASD + QE to move\nHold left Shift for turbo mode\nPress C to Toggle from trackpad to mouse controls",
        TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 15.0,
            color: Color::WHITE,
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(10.0),
        left: Val::Px(10.0),
        ..default()
    }),));
}

fn tick(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut CameraRig), With<Camera>>,
    mut use_mouse_motion: Local<bool>,
) {
    let (mut camera_transform, mut rig) = camera_query.single_mut();
    let mut right = 0.0;
    let mut up = 0.0;
    let mut forward = 0.0;

    if keys.pressed(KeyCode::A) {
        right = -1.0;
    }

    if keys.pressed(KeyCode::D) {
        right = 1.0;
    }

    if keys.pressed(KeyCode::W) {
        forward = 1.0;
    }

    if keys.pressed(KeyCode::S) {
        forward = -1.0;
    }

    if keys.pressed(KeyCode::Q) {
        up = -1.0;
    }

    if keys.pressed(KeyCode::E) {
        up = 1.0;
    }

    let boost = if keys.pressed(KeyCode::ShiftLeft) { 1.0 } else { 0.0 };

    if keys.just_pressed(KeyCode::C) {
        *use_mouse_motion = !*use_mouse_motion;
        println!(
            "Switching mouse control to {}",
            if *use_mouse_motion { "MOTION" } else { "WHEEL" }
        );
    }

    let mut mouse_delta = Vec2::ZERO;
    if *use_mouse_motion {
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
    } else {
        for mouse_wheel_event in mouse_wheel_events.iter() {
            mouse_delta += Vec2::new(mouse_wheel_event.x, mouse_wheel_event.y);
        }
    }

    let move_vec =
        camera_transform.rotation * Vec3::new(right, up, -forward).clamp_length_max(1.0) * 10.0f32.powf(boost);

    rig.driver_mut::<YawPitch>()
        .rotate_yaw_pitch(-0.3 * mouse_delta.x, -0.3 * mouse_delta.y);
    rig.driver_mut::<Position>()
        .translate(move_vec * time.delta_seconds() * 10.0);

    rig.update_into(time.delta_seconds(), camera_transform.as_mut());
}
