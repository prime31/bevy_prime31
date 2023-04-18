use bevy::prelude::*;
use dolly::prelude::*;

#[derive(Component)]
struct ActiveMoveTransform;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DollyPlugin)
        .add_startup_system(setup)
        .add_system(check_swap_active)
        .add_system(tick)
        .add_system(sync_rig_to_camera.after(tick))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_position = Vec3::new(4., 3., 8.);
    let player_position = Vec3::new(2., 0.5, 2.);

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
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial::from(Color::YELLOW)),
            transform: Transform::from_translation(player_position),
            ..default()
        },
        ActiveMoveTransform,
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
        // Allow moving the camera
        .with(Position::new(camera_position))
        // Predict camera movement to make the subsequent smoothing reactive
        .with(Smooth::new_position(1.25).predictive(true))
        // Smooth the predicted movement
        .with(Smooth::new_position(2.5))
        .with(LookAt::new(player_position + Vec3::Y).tracking_smoothness(1.25))
        .build();

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(camera_position),
        ..default()
    });

    // camera proxy with the CameraRig. This is the transform that should be modified
    commands.spawn((Transform::from_translation(camera_position), camera_rig));
}

fn check_swap_active(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    query: Query<(Entity, Option<&ActiveMoveTransform>), Or<(With<Player>, With<CameraRig>)>>,
) {
    if !keys.just_pressed(KeyCode::C) {
        return;
    };

    for (entity, maybe_move_transform) in query.iter() {
        if let Some(_) = maybe_move_transform {
            commands.entity(entity).remove::<ActiveMoveTransform>();
        } else {
            commands.entity(entity).insert(ActiveMoveTransform);
        }
    }
}

fn sync_rig_to_camera(
    time: Res<Time>,
    mut rig_q: Query<(&mut CameraRig, &Transform), Without<Camera>>,
    player_q: Query<&Transform, (With<Player>, Without<CameraRig>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let (mut rig, rig_transform) = rig_q.single_mut();
    let player_transform = player_q.single();
    let mut camera_transform = camera_q.single_mut();

    rig.driver_mut::<Position>().position = rig_transform.translation;
    rig.driver_mut::<LookAt>().target = player_transform.translation;
    rig.update_into(time.delta_seconds(), camera_transform.as_mut());
}

fn tick(keys: Res<Input<KeyCode>>, mut q: Query<&mut Transform, With<ActiveMoveTransform>>) {
    fn get_move_input(keys: Res<Input<KeyCode>>) -> Vec3 {
        const SPEED: f32 = 0.05;
        let mut pos = Vec3::ZERO;

        if keys.pressed(KeyCode::W) {
            pos.z -= SPEED;
        }

        if keys.pressed(KeyCode::A) {
            pos.x -= SPEED;
        }

        if keys.pressed(KeyCode::S) {
            pos.z += SPEED;
        }

        if keys.pressed(KeyCode::D) {
            pos.x += SPEED;
        }

        if keys.pressed(KeyCode::Q) {
            pos.y -= SPEED;
        }

        if keys.pressed(KeyCode::E) {
            pos.y += SPEED;
        }

        pos
    }

    let delta_move = get_move_input(keys);
    q.single_mut().translation += delta_move;
}
