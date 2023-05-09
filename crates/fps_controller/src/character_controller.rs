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
    mut query: Query<(&mut KinematicCharacterController, &FpsControllerInput, &Velocity)>,
    render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
) {
    for (mut controller, input, _) in query.iter_mut() {
        for tf in render_query.iter() {
            let tf_yaw = tf.rotation.to_euler(EulerRot::YXZ).0;
            let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, tf_yaw - input.yaw);
            move_to_world.z_axis *= -1.0; // Forward is -Z

            let mut wish_direction = move_to_world * (input.movement * Vec3::new(30.0, 0.0, 30.0));
            let mut wish_speed = wish_direction.length();
            if wish_speed > f32::EPSILON {
                // Avoid division by zero
                wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
            }

            // wish_direction = wish_direction.clamp_length_max(1.0);

            // config these
            let crouched_speed = 5.0;
            let walk_speed = 9.0;
            let run_speed = 14.0;
            let air_speed_cap = 2.0;
            let gravity = 20.0;
            let jump_speed = 8.0;

            let max_speed = if input.crouch {
                crouched_speed
            } else if input.sprint {
                run_speed
            } else {
                walk_speed
            };
            wish_speed = f32::min(wish_speed, max_speed);
            wish_speed = f32::min(wish_speed, air_speed_cap);
            wish_direction.y = gravity * time.delta_seconds();

            if input.jump {
                wish_direction.y = jump_speed;
            }

            controller.translation = Some(-wish_direction * 0.1);
        }
    }
}

fn read_result_system(controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    for (entity, output) in controllers.iter() {
        println!(
            "Entity {:?} moved by {:?} and touches the ground: {:?}",
            entity, output.effective_translation, output.grounded
        );
    }
}
