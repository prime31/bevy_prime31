use crate::generate::ConvexCollision;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_rapier3d::prelude::Collider;

use self::loader::ValveMapLoader;

pub mod loader;


#[derive(Debug, TypeUuid)]
#[uuid = "44cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct ValveMap {
    pub collision_geometry: Vec<ConvexCollision>,
    pub entities: Vec<(Vec3, Handle<Mesh>, Handle<StandardMaterial>)>,
}


#[derive(Default, Bundle)]
pub struct ValveMapBundle {
    pub map: Handle<ValveMap>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}




#[derive(Component)]
pub struct ValveMapHandled(pub Handle<ValveMap>);

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
) {
    for (entity, map_bundle) in q.iter() {
        if let Some(map) = assets.get(&map_bundle) {
            commands.entity(entity).remove::<ValveMapBundle>().insert((
                ValveMapHandled(map_bundle.clone()),
                TransformBundle::default(),
                VisibilityBundle::default(),
                Name::new("ValveMapRoot"),
            ));
            instantiate_map_entities(&mut commands, entity, map);
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
                instantiate_map_entities(&mut commands, entity, map);
            }
        }
    }
}

fn instantiate_map_entities(commands: &mut Commands, entity: Entity, map: &ValveMap) {
    commands.entity(entity).with_children(|builder| {
        for (pos, mesh, material) in &map.entities {
            builder.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_translation(*pos),
                    ..default()
                },
                Name::new("ValveMapBrush"),
            ));
        }

        for geo in &map.collision_geometry {
            builder.spawn((
                Collider::convex_hull(&geo.to_local(16.0)).unwrap(),
                GlobalTransform::default(),
                Transform::from_translation(geo.center_local(16.0)),
                Name::new("ValveMapBrushCollider"),
            ));
        }
    });
}
