use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::input::{FpsControllerInput, FpsControllerStages, FpsPlayer};

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
    mut query: Query<
        (
            &Transform,
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
            &mut FpsControllerInput,
        ),
        With<FpsPlayer>,
    >,
) {
    for (tf, mut controller, controller_out, mut input) in query.iter_mut() {
        // friction
        {
            let speed = input.vel.length();
            let mut drop = 0.0;

            // only if grounded
            if controller_out.grounded {
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

        let wish_direction = tf.forward() * input.movement.z + tf.right() * input.movement.x;
        let mut wish_speed = wish_direction.length();

        // config these
        let walk_speed = 9.0;
        let run_speed = 14.0;
        let gravity = 20.0;
        let jump_speed = 10.0;
        let ground_accel = 10.0;
        let air_accel = 7.0;

        let target_speed = if input.sprint { run_speed } else { walk_speed };
        wish_speed *= target_speed;

        if input.dash_pressed {
            wish_speed *= 50.0;
        }

        if controller_out.grounded {
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

            if input.jump_pressed {
                input.vel.y = jump_speed;
            }
        } else {
            let mut add_speed = acceleration(wish_direction, wish_speed, air_accel, input.vel, time.delta_seconds());
            add_speed.y = -gravity * time.delta_seconds();
            input.vel += add_speed;
        }

        controller.filter_flags = QueryFilterFlags::EXCLUDE_SENSORS;
        controller.translation = Some(input.vel * time.delta_seconds());
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
