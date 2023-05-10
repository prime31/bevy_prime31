use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::input::{FpsControllerInput, FpsControllerStages, FpsPlayer, RenderPlayer};

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
    mut query: Query<(&mut KinematicCharacterController, &mut FpsControllerInput)>,
    render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
    kcc_output_q: Query<&KinematicCharacterControllerOutput>,
) {
    let mut grounded = false;
    if let Ok(kcc) = kcc_output_q.get_single() {
        grounded = kcc.grounded;
    };

    for (mut controller, mut input) in query.iter_mut() {
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
            move_to_world.z_axis *= 1.0; // Forward is -Z
            move_to_world.x_axis *= -1.0;

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
            controller.translation = Some(input.vel * time.delta_seconds());
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
