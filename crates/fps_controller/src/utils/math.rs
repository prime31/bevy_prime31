
pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if f32::abs(target - current) <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}

/// moves current towards target by shift amount clamping the result. start can be less than or greater than end.
/// example: start is 2, end is 10, shift is 4 results in 6
pub fn approach(current: f32, target: f32, shift: f32) -> f32 {
    if current < target {
        return f32::min(current + shift, target)
    }
    f32::max(current - shift, target)
}

/// maps value (which is in the range left_min - left_max) to a value in the range right_min - right_max
pub fn map(value: f32, left_min: f32, left_max: f32, right_min: f32, right_max: f32) -> f32 {
    let slope = (right_max - right_min) / (left_max - left_min);
    right_min + slope * (value - left_min)
}

/// Maps a value from some arbitrary range to the 0 to 1 range
pub fn map_01(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}
