use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::input::{FpsControllerInput, FpsPlayer};

#[derive(Default)]
pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update).add_system(read_result_system);
    }
}

pub fn update(
    q: Query<&FpsControllerInput, With<FpsPlayer>>,
    mut controllers: Query<&mut KinematicCharacterController>,
) {
    let input = q.get_single().unwrap();

    for mut controller in controllers.iter_mut() {
        controller.translation = Some(input.movement * 0.1);
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
