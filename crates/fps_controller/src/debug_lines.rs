use std::f32::consts::FRAC_PI_2;

use bevy::prelude::Vec3;
use bevy_prototype_debug_lines::DebugLines;

pub fn draw_circle_xz(lines: &mut DebugLines) {
    let pos = Vec3::ZERO;
    let radius = 3.0;
    let resolution = 3;

    let angle_to_vec: fn(f32, f32) -> Vec3 =
        |angle_rads, len| Vec3::new(f32::cos(angle_rads) * len, 0.0, f32::sin(angle_rads) * len);

    let resolution = resolution as f32;
    let mut last = Vec3::X * radius;

    for i in 1..resolution as u32 * 4 + 2 {
        let at = angle_to_vec(i as f32 * FRAC_PI_2 / resolution, radius);
        lines.line(pos + last, pos + at, 10.0);
        last = at;
    }
}
