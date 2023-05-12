use bevy::prelude::*;
use dolly::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DollyPlugin)
        .add_startup_system(setup)
        .add_system(tick)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let camera_position = Vec3::new(4., 3., 8.);
    let player_position = Vec3::new(2., 0.25, 2.);

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

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(StandardMaterial::from(Color::YELLOW)),
            transform: Transform::from_translation(player_position),
            ..default()
        },
        Player,
    ));

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
        material: materials.add(StandardMaterial::from(Color::ALICE_BLUE)),
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
        // align the camera to our player
        .with(Position::new(player_position))
        .with(Rotation::new(Quat::IDENTITY))
        // Predict camera movement to make the subsequent smoothing reactive
        .with(Smooth::new_position(1.25).predictive(true))
        .with(Arm::new(Vec3::new(0.0, 1.5, -5.5)))
        // Smooth the predicted movement
        .with(Smooth::new_position(2.5))
        .with(
            LookAt::new(player_position + Vec3::Y)
                .tracking_smoothness(1.25)
                .tracking_predictive(true),
        )
        .build();

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_position),
            ..default()
        },
        camera_rig,
    ));
}

fn tick(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut rig_q: Query<(&mut CameraRig, &mut Transform), Without<Player>>,
) {
    fn get_move_input(keys: Res<Input<KeyCode>>) -> Vec3 {
        const SPEED: f32 = 0.08;
        const ROT_SPEED: f32 = 0.02;
        let mut pos = Vec3::ZERO;

        if keys.pressed(KeyCode::W) {
            pos.z -= SPEED;
        }

        if keys.pressed(KeyCode::S) {
            pos.z += SPEED;
        }

        if keys.pressed(KeyCode::A) {
            pos.x += ROT_SPEED;
        }

        if keys.pressed(KeyCode::D) {
            pos.x -= ROT_SPEED;
        }

        pos
    }

    let delta_move = get_move_input(keys);
    let mut player_transform = player_q.single_mut();
    player_transform.rotate_y(delta_move.x);
    player_transform.translation = player_transform.translation + player_transform.forward() * delta_move.z;

    let (mut rig, mut camera_transform) = rig_q.single_mut();
    rig.driver_mut::<Position>().position = player_transform.translation;
    rig.driver_mut::<Rotation>().rotation = player_transform.rotation;
    rig.driver_mut::<LookAt>().target = player_transform.translation + Vec3::Y;
    rig.update_into(time.delta_seconds(), camera_transform.as_mut());
}
