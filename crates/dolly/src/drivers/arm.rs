use bevy::prelude::Vec3;

use crate::{
    driver::RigDriver, rig::RigUpdateParams, transform::CameraTransform,
};

/// Offsets the camera along a vector, in the coordinate space of the parent.
#[derive(Debug)]
pub struct Arm {
    ///
    pub offset: Vec3,
}

impl Arm {
    ///
    pub fn new(offset: Vec3) -> Self {
        Self { offset }
    }
}

impl RigDriver for Arm {
    fn update(&mut self, params: RigUpdateParams) -> CameraTransform {
        CameraTransform {
            rotation: params.parent.rotation,
            position: params.parent.position + params.parent.rotation * self.offset,
        }
    }
}
