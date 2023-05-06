use bevy::{prelude::*, reflect::TypeUuid};
use bevy_rapier3d::prelude::{Collider, Sensor};

use self::loader::{ValveMapLoader, ValveMapEntity};

pub mod loader;


#[derive(Debug, TypeUuid)]
#[uuid = "44cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct ValveMap {
    pub entities: Vec<ValveMapEntity>,
}


#[derive(Default, Bundle)]
pub struct ValveMapBundle {
    pub map: Handle<ValveMap>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

/// any Entities with this Component will be warped to the "classname = spawn_point" from the map on map load or reload
#[derive(Component)]
pub struct ValveMapPlayer;

/// Component added to the Entity that the Handle<ValveMap> was added to after the map is loaded. Used later
/// during hot-reload to identify the map and swap in the new one.
#[derive(Component)]
struct ValveMapHandled(pub Handle<ValveMap>);

#[derive(Default)]
pub struct ValveMapPlugin;

impl Plugin for ValveMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<ValveMapLoader>()
            .add_asset::<ValveMap>()
            .add_system(handle_loaded_maps);
    }
}

fn handle_loaded_maps(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<ValveMap>>,
    assets: ResMut<Assets<ValveMap>>,
    q: Query<(Entity, &Handle<ValveMap>)>,
    q_mod: Query<(Entity, &ValveMapHandled)>,
    q_players: Query<&mut Transform, With<ValveMapPlayer>>
) {
    for (entity, map_bundle) in q.iter() {
        if let Some(map) = assets.get(&map_bundle) {
            commands.entity(entity).remove::<ValveMapBundle>().insert((
                ValveMapHandled(map_bundle.clone()),
                TransformBundle::default(),
                VisibilityBundle::default(),
                Name::new("ValveMapRoot"),
            ));
            instantiate_map_entities(&mut commands, entity, map, q_players);
            return;
        }
    }

    for ev in ev_asset.iter() {
        if let AssetEvent::Modified { handle } = ev {
            for (entity, map_bundle) in q_mod.iter() {
                if map_bundle.0 != *handle {
                    continue;
                }
                commands.entity(entity).despawn_descendants();

                let map = assets.get(&map_bundle.0).unwrap();
                instantiate_map_entities(&mut commands, entity, map, q_players);
                return;
            }
        }
    }
}

fn instantiate_map_entities(commands: &mut Commands, entity: Entity, map: &ValveMap, mut q_players: Query<&mut Transform, With<ValveMapPlayer>>) {
    commands.entity(entity).with_children(|builder| {
        for map_entity in &map.entities {
            println!("------------ class: {:?}, visuals: {}", map_entity.fields.get_property("classname"), map_entity.visual_geometry.len());
            let is_sensor = map_entity.fields.is_sensor();

            // handle any point types
            if let Some("light") = map_entity.fields.get_property("classname") {
                builder.spawn(PointLightBundle {
                    point_light: PointLight {
                        color: map_entity.fields.get_color_property("color").unwrap_or(Color::WHITE),
                        intensity: map_entity.fields.get_f32_property("intensity").unwrap_or(800.),
                        range: map_entity.fields.get_f32_property("range").unwrap_or(20.),
                        shadows_enabled: map_entity.fields.get_bool_property("shadows_enabled").unwrap_or(false),
                        ..default()
                    },
                    transform: Transform::from_translation(map_entity.fields.get_vec3_property("origin").unwrap()),
                    ..default()
                });
            }

            if let Some("spawn_point") = map_entity.fields.get_property("classname") {
                let position = map_entity.fields.get_vec3_property("origin").unwrap();
                for mut tf in q_players.iter_mut() {
                    tf.translation = position;
                }
            }

            for visual_geo in &map_entity.visual_geometry {
                builder.spawn((
                    PbrBundle {
                        mesh: visual_geo.mesh.clone(),
                        material: visual_geo.material.clone(),
                        transform: Transform::from_translation(visual_geo.origin),
                        ..default()
                    },
                    Name::new("ValveMapBrush"),
                ));
            }

            for geo in &map_entity.collision_geometry {
                let mut entity = builder.spawn((
                    Collider::convex_hull(&geo.to_local(16.0)).unwrap(),
                    GlobalTransform::default(),
                    Transform::from_translation(geo.center_local(16.0)),
                    Name::new("ValveMapBrushCollider"),
                ));

                if is_sensor { entity.insert(Sensor); }
            }
        }
    });
}
