use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::input::{FpsControllerInput, FpsPlayer, RenderPlayer};

#[derive(Default)]
pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update).add_system(read_result_system);
    }
}

pub fn update(
    time: Res<Time>,
    query: Query<&FpsControllerInput>,
    render_query: Query<&Transform, (With<RenderPlayer>, Without<FpsPlayer>)>,
    mut controllers: Query<&mut KinematicCharacterController>,
) {
    let input = query.get_single().unwrap();

    for input in query.iter() {
        for tf in render_query.iter() {
            let euler = tf.rotation.to_euler(EulerRot::YXZ);
            let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, euler.0 - input.yaw);
            move_to_world.z_axis *= -1.0; // Forward is -Z

            let mut wish_direction = move_to_world * (input.movement * Vec3::new(30.0, 0.0, 30.0));
            let mut wish_speed = wish_direction.length();
            if wish_speed > f32::EPSILON {
                // Avoid division by zero
                wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
            }

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

            for mut controller in controllers.iter_mut() {
                controller.translation = Some(-wish_direction * 0.1);
            }
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
