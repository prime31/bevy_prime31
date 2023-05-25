use std::f32::consts::FRAC_PI_2;

use bevy::prelude::{Color, Vec3};
use bevy_prototype_debug_lines::DebugLines;

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

impl DebugLinesExt for DebugLines {
    fn get(&mut self) -> &mut DebugLines {
        self
    }
}

pub trait DebugLinesExt {
    fn get(&mut self) -> &mut DebugLines;

    fn thick_line(&mut self, start: Vec3, end: Vec3, duration: f32) {
        self.thick_colored_line(start, end, duration, Color::WHITE);
    }

    fn thick_colored_line(&mut self, start: Vec3, end: Vec3, duration: f32, color: Color) {
        let dl = self.get();
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

    fn draw_circle_xz(&mut self, pos: Vec3, radius: f32, resolution: u32) {
        let lines = self.get();

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
}
