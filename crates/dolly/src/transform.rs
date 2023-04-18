use core::fmt::Debug;

use bevy::prelude::{Vec3, Quat};

/// A thin wrapper over a `Vec3` and a `Quat`
#[derive(Clone, Copy, Debug)]
pub struct CameraTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

impl CameraTransform {
    ///
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
        }
    }

    ///
    pub fn into_position_rotation(self) -> (Vec3, Quat) {
        (self.position, self.rotation)
    }

    /// +X
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// +Y
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// +/-Z
    pub fn forward(&self) -> Vec3 {
        self.rotation * bevy::math::vec3(0.0, 0.0, -1.0)
    }

    ///
    pub const IDENTITY: CameraTransform = CameraTransform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };
}
