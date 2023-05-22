use std::f32::consts::FRAC_PI_2;

use bevy::prelude::{Vec3, ResMut, Color};
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

pub fn get_circle_xz_pts(pos: Vec3, radius: f32, resolution: u32) -> Vec<Vec3> {
    let mut pts = Vec::new();

    let angle_to_vec: fn(f32, f32) -> Vec3 =
        |angle_rads, len| Vec3::new(f32::cos(angle_rads) * len, 0.0, f32::sin(angle_rads) * len);

    let resolution = resolution as f32;
    let mut last = Vec3::X * radius;

    for i in 1..resolution as u32 * 4 + 2 {
        let at = angle_to_vec(i as f32 * FRAC_PI_2 / resolution, radius);
        pts.push(pos + last);
        pts.push(pos + at);
        // lines.line(pos + last, pos + at, 10.0);
        last = at;
    }

    pts
}

pub fn thick_line(dl: &mut ResMut<DebugLines>, start: Vec3, end: Vec3, duration: f32) {
    thick_colored_line(dl, start, end, duration, Color::WHITE);
}

pub fn thick_colored_line(dl: &mut ResMut<DebugLines>, start: Vec3, end: Vec3, duration: f32, color: Color) {
    let jitter = 0.0025;
    let a = Vec3::new(-1.0, 0.0, 1.0) * jitter;
    let b = Vec3::new(1.0, 0.0, 1.0) * jitter;
    let c = Vec3::new(-1.0, 0.0, -1.0) * jitter;
    let d = Vec3::new(-1.0, 0.0, -1.0) * jitter;

    dl.line_colored(start + a, end + a, duration, color);
    dl.line_colored(start + b, end + b, duration, color);
    dl.line_colored(start + c, end + c, duration, color);
    dl.line_colored(start + d, end + d, duration, color);
    dl.line_colored(start, end, duration, color);
}
