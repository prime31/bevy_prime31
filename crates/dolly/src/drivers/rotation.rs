use bevy::prelude::Quat;

use crate::{
    driver::RigDriver, rig::RigUpdateParams, transform::CameraTransform,
};

/// Directly sets the rotation of the camera
#[derive(Default, Debug)]
pub struct Rotation {
    pub rotation: Quat,
}

impl Rotation {
    pub fn new(rotation: Quat) -> Self {
        Self { rotation }
    }
}

impl RigDriver for Rotation {
    fn update(&mut self, params: RigUpdateParams) -> CameraTransform {
        CameraTransform {
            position: params.parent.position,
            rotation: self.rotation,
        }
    }
}
