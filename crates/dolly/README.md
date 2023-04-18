# bevy_dolly

Fork of [dolly](https://github.com/h3r2tic/dolly) made for bevy.
- removed dependency on glam in favor of using bevy's default glam crate.
- removed engine-agnostic aspects (no more left handed support)
- added bevy specific helpers




## Example (see examples folder for more)

```rust
// create a simple flycam CameraRig component and stick it on an Entity
let camera_rig: CameraRig = CameraRig::builder()
    .with(Position::new(Vec3::Y))
    .with(YawPitch::new())
    .with(Smooth::new_position_rotation(1.0, 1.0))
    .build();

// ... in your system

// gather input in your system
let move_vec = camera_transform.rotation
    * Vec3::new(right, up, -forward).clamp_length_max(1.0)
    * 10.0f32.powf(boost);

// update the drivers we added previously
rig.driver_mut::<YawPitch>().rotate_yaw_pitch(-0.3 * mouse_delta_x, -0.3 * mouse_delta_y);
rig.driver_mut::<Position>().translate(move_vec * time.delta_seconds() * 10.0);

// calculate the final transform and copy it to the camera transform
rig.update_into(time.delta_seconds(), camera_transform.as_mut());

```
