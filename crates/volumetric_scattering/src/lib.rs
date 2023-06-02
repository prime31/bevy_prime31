use bevy::prelude::*;

// https://medium.com/@andrew_b_berg/volumetric-light-scattering-in-three-js-6e1850680a41

#[derive(Default)]
pub struct VolumetricScatteringPlugin;

impl Plugin for VolumetricScatteringPlugin {
    fn build(&self, _app: &mut App) {}
}
