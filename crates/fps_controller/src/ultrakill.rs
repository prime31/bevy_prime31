use crate::input::{FpsControllerInput, FpsControllerStages};
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier3d::prelude::*;
use egui_helper::bevy_inspector_egui::{self, bevy_egui::EguiContext, egui};

#[derive(Default)]
pub struct UltrakillControllerPlugin;

impl Plugin for UltrakillControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(controller_move.in_set(FpsControllerStages::Logic))
            .add_system(debug_ui.run_if(egui_helper::run_if_egui_enabled));
    }
}

#[derive(Component)]
pub struct RenderPlayer;

#[derive(Component)]
pub struct FpsController {
    pub radius: f32,
    pub gravity: f32,
    pub walk_speed: f32,
    pub slide_speed: f32,
    pub dash_speed: f32,
    pub forward_speed: f32,
    pub side_speed: f32,
    pub air_speed_cap: f32,
    pub air_acceleration: f32,
    pub max_air_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    /// If the dot product (alignment) of the normal of the surface and the upward vector,
    /// which is a value from [-1, 1], is greater than this value, ground movement is applied
    pub traction_normal_cutoff: f32,
    pub friction_speed_cutoff: f32,
    pub jump_speed: f32,
    pub fly_speed: f32,
    pub crouch_speed: f32,
    pub uncrouch_speed: f32,
    pub height: f32,
    pub upright_height: f32,
    pub crouch_height: f32,
    pub fast_fly_speed: f32,
    pub fly_friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub ground_tick: u8,
    pub stop_speed: f32,
    pub sensitivity: f32,
    pub enable_input: bool,
    pub step_offset: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_sprint: KeyCode,
    pub key_jump: KeyCode,
    pub key_fly: KeyCode,
    pub key_crouch: KeyCode,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            radius: 0.5,
            fly_speed: 10.0,
            fast_fly_speed: 30.0,
            gravity: 23.0,
            walk_speed: 9.0,
            slide_speed: 30.0,
            dash_speed: 1000.0,
            forward_speed: 30.0,
            side_speed: 50.0,
            air_speed_cap: 2.0,
            air_acceleration: 20.0,
            max_air_speed: 15.0,
            crouch_speed: 50.0,
            uncrouch_speed: 8.0,
            height: 1.0,
            upright_height: 2.0,
            crouch_height: 1.0,
            acceleration: 10.0,
            friction: 10.0,
            traction_normal_cutoff: 0.7,
            friction_speed_cutoff: 0.1,
            fly_friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            ground_tick: 0,
            stop_speed: 1.0,
            jump_speed: 10.5,
            step_offset: 0.0,
            enable_input: true,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_sprint: KeyCode::LShift,
            key_jump: KeyCode::Space,
            key_fly: KeyCode::F,
            key_crouch: KeyCode::C,
            sensitivity: 0.005,
        }
    }
}

pub fn controller_move(
    time: Res<Time>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &FpsControllerInput,
        &mut FpsController,
        &mut Collider,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    let dt = time.delta_seconds();

    for (entity, input, mut controller, mut collider, mut transform, mut velocity) in query.iter_mut() {
        if let Some(capsule) = collider.as_capsule() {
            // Capsule cast downwards to find ground
            // Better than a ray cast as it handles when you are near the edge of a surface
            let capsule = capsule.raw;
            let cast_capsule =
                Collider::capsule(capsule.segment.a.into(), capsule.segment.b.into(), capsule.radius * 0.9);
            // Avoid self collisions
            let filter = QueryFilter::default().exclude_rigid_body(entity).exclude_sensors();
            let ground_cast = physics_context.cast_shape(
                transform.translation,
                transform.rotation,
                -Vec3::Y,
                &cast_capsule,
                0.125,
                filter,
            );

            let wish_direction = input.movement_dir;

            let mut wish_speed = if input.dash.pressed {
                controller.dash_speed
            } else if input.slide.down {
                controller.slide_speed
            } else {
                controller.walk_speed
            };

            if let Some((_, toi)) = ground_cast {
                let has_traction = Vec3::dot(toi.normal1, Vec3::Y) > controller.traction_normal_cutoff;

                // Only apply friction after at least one tick, allows b-hopping without losing speed
                if controller.ground_tick >= 1 && has_traction {
                    let lateral_speed = velocity.linvel.xz().length();
                    if lateral_speed > controller.friction_speed_cutoff {
                        let control = f32::max(lateral_speed, controller.stop_speed);
                        let drop = control * controller.friction * dt;
                        let new_speed = f32::max((lateral_speed - drop) / lateral_speed, 0.0);
                        velocity.linvel.x *= new_speed;
                        velocity.linvel.z *= new_speed;
                    } else {
                        velocity.linvel = Vec3::ZERO;
                    }
                    if controller.ground_tick == 1 {
                        velocity.linvel.y = -toi.toi;
                    }
                }

                let mut add = acceleration(wish_direction, wish_speed, controller.acceleration, velocity.linvel, dt);
                if !has_traction {
                    add.y -= controller.gravity * dt;
                }
                velocity.linvel += add;

                if has_traction {
                    let linvel = velocity.linvel;
                    velocity.linvel -= Vec3::dot(linvel, toi.normal1) * toi.normal1;

                    if input.jump.pressed {
                        velocity.linvel.y = controller.jump_speed;
                    }
                }

                // Increment ground tick but cap at max value
                controller.ground_tick = controller.ground_tick.saturating_add(1);
            } else {
                controller.ground_tick = 0;
                wish_speed = f32::min(wish_speed, controller.air_speed_cap);

                let mut add = acceleration(
                    wish_direction,
                    wish_speed,
                    controller.air_acceleration,
                    velocity.linvel,
                    dt,
                );
                add.y = -controller.gravity * dt;
                velocity.linvel += add;

                let air_speed = velocity.linvel.xz().length();
                if air_speed > controller.max_air_speed {
                    let ratio = controller.max_air_speed / air_speed;
                    velocity.linvel.x *= ratio;
                    velocity.linvel.z *= ratio;
                }
            }

            // Crouching
            let crouch_height = controller.crouch_height;
            let upright_height = controller.upright_height;

            let crouch_speed = if input.dash.down { -controller.crouch_speed } else { controller.uncrouch_speed };
            controller.height += dt * crouch_speed;
            controller.height = controller.height.clamp(crouch_height, upright_height);

            if let Some(mut capsule) = collider.as_capsule_mut() {
                // capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5);
                capsule.set_segment(Vec3::Y * -0.5, Vec3::Y * 0.5 * (controller.height - 1.0));
            }

            // Step offset
            if controller.step_offset > f32::EPSILON && controller.ground_tick >= 1 {
                let cast_offset = velocity.linvel.normalize_or_zero() * controller.radius * 1.0625;
                let cast = physics_context.cast_ray_and_get_normal(
                    transform.translation + cast_offset + Vec3::Y * controller.step_offset * 1.0625,
                    -Vec3::Y,
                    controller.step_offset * 0.9375,
                    false,
                    filter,
                );

                if let Some((_, hit)) = cast {
                    transform.translation.y += controller.step_offset * 1.0625 - hit.toi;
                    transform.translation += cast_offset;
                }
            }
        }
    }
}

fn acceleration(wish_direction: Vec3, wish_speed: f32, acceleration: f32, velocity: Vec3, dt: f32) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
}

fn debug_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
        .single(world)
        .clone();

    let entity = world.query_filtered::<Entity, With<FpsController>>().single(world);

    egui::Window::new("Hello").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
            ui.allocate_space(ui.available_size());
        });
    });
}
