use bevy::{input::mouse::MouseWheel, prelude::*};
use dolly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DollyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, tick)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            subdivisions: 10,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
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
        .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-30.0))
        .with(Smooth::new_rotation(1.5))
        .with(Arm::new(Vec3::Z * 8.0))
        .build();

    // camera
    commands.spawn((Camera3dBundle::default(), camera_rig));
}

fn tick(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut CameraRig), With<Camera>>,
) {
    let (mut camera_transform, mut rig) = camera_query.single_mut();

    if keys.just_pressed(KeyCode::A) {
        rig.driver_mut::<YawPitch>().rotate_yaw_pitch(-90.0, 0.0);
    }

    if keys.just_pressed(KeyCode::D) {
        rig.driver_mut::<YawPitch>().rotate_yaw_pitch(90.0, 0.0);
    }

    if keys.just_pressed(KeyCode::Q) {
        rig.driver_mut::<YawPitch>().rotate_yaw_pitch(0.0, 15.0);
    }

    if keys.just_pressed(KeyCode::E) {
        rig.driver_mut::<YawPitch>().rotate_yaw_pitch(0.0, -15.0);
    }

    for mouse_wheel_event in mouse_wheel_events.iter() {
        let z = &mut rig.driver_mut::<Arm>().offset.z;
        *z -= mouse_wheel_event.y * 0.01;
        *z = z.clamp(2.0, 20.0);
    }

    rig.update_into(time.delta_seconds(), camera_transform.as_mut());
}
