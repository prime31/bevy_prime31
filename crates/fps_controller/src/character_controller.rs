use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::input::{FpsControllerInput, FpsControllerStages, FpsPlayer, RenderPlayer};

// https://github.com/IsaiahKelly/quake3-movement-for-unity/blob/master/Quake3Movement/Scripts/Q3PlayerController.cs

#[derive(Default)]
pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update.in_set(FpsControllerStages::Logic))
            .add_system(read_result_system);
    }
}

pub fn update(
    time: Res<Time>,
    mut rapier: ResMut<RapierContext>,
    mut query: Query<(Entity, &mut Transform, &mut KinematicCharacterController, &mut KinematicCharacterControllerOutput, &mut FpsControllerInput, &Collider), With<FpsPlayer>>,
    render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    for (entity, mut logical_tf, mut controller, mut controller_out, mut input, collider) in query.iter_mut() {
        let grounded = controller_out.grounded;

        for tf in render_query.iter() {
            // friction
            {
                let mut vel = input.vel;
                vel.y = 0.0;
                let speed = vel.length();
                let mut drop = 0.0;

                // only if grounded
                if grounded {
                    let ground_deceleration = 10.0;
                    let friction = 6.0;
                    let control = if speed < ground_deceleration { ground_deceleration } else { speed };
                    drop = control * friction * time.delta_seconds();
                }

                let mut new_speed = speed - drop;
                if new_speed < 0.0 {
                    new_speed = 0.0;
                }

                if speed > 0.0 {
                    new_speed /= speed;
                };
                input.vel.x *= new_speed;
                input.vel.z *= new_speed;
            }

            let (tf_yaw, _, _) = tf.rotation.to_euler(EulerRot::YXZ);
            let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, tf_yaw - input.yaw);
            move_to_world.z_axis *= -1.0; // Forward is -Z
            // move_to_world.x_axis *= -1.0;

            let mut wish_direction = (move_to_world * input.movement).normalize_or_zero();
            wish_direction.y = 0.0;
            let mut wish_speed = wish_direction.length();

            // config these
            let crouched_speed = 5.0;
            let walk_speed = 9.0;
            let run_speed = 14.0;
            let _air_speed_cap = 2.0;
            let gravity = 20.0;
            let jump_speed = 10.0;
            let ground_accel = 10.0;
            let _air_accel = 20.0;

            let target_speed = if input.crouch {
                crouched_speed
            } else if input.sprint {
                run_speed
            } else {
                walk_speed
            };

            wish_speed *= target_speed;

            if grounded {
                let add_speed = acceleration(
                    wish_direction,
                    wish_speed,
                    ground_accel,
                    input.vel,
                    time.delta_seconds(),
                );
                input.vel += add_speed;

                // reset gravity rather than accrue it
                input.vel.y = -gravity * time.delta_seconds();

                if input.jump {
                    input.vel.y = jump_speed;
                }
            } else {
                input.vel.y -= gravity * time.delta_seconds();
            }

            controller.filter_flags = QueryFilterFlags::EXCLUDE_SENSORS;
            // controller.translation = Some(input.vel * time.delta_seconds());

            let mut move_opts = MoveShapeOptions::default();
            move_opts.max_slope_climb_angle = 25.0_f32.to_radians();
            move_opts.min_slope_slide_angle = 10.0_f32.to_radians();

            let move_res = rapier.move_shape(
                input.vel * time.delta_seconds(),
                collider,
                logical_tf.translation,
                Quat::IDENTITY,
                1.0,
                &move_opts,
                QueryFilter::default().exclude_collider(entity).exclude_sensors(),
                |_col| {
                    // println!("col: {:?}", col);
                },
            );

            logical_tf.translation += move_res.effective_translation;

            controller_out.grounded = move_res.grounded;
        }
    }
}

fn acceleration(wish_direction: Vec3, wish_speed: f32, acceleration: f32, velocity: Vec3, dt: f32) -> Vec3 {
    let current_speed = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - current_speed;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
}

fn read_result_system(_controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    // for (entity, output) in _controllers.iter() {
    //     println!(
    //         "Entity {:?} moved by {:?} and touches the ground: {:?}",
    //         entity, output.effective_translation, output.grounded
    //     );
    // }
}
