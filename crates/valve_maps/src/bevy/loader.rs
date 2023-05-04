use std::any::Any;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::{CompressedImageFormats, ImageSampler, ImageType},
    },
    utils::{BoxedFuture, HashMap},
};

use bevy_rapier3d::prelude::*;

use crate::{
    convert::{get_brush_entity_visual_geometry, MeshSurface},
    generate::{ConvexCollision, TextureInfo},
};

#[derive(Debug, TypeUuid)]
#[uuid = "49cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct ValveMap {}

#[derive(Default)]
pub struct ValveMapLoader;

impl AssetLoader for ValveMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let string = std::str::from_utf8(bytes)?;
            let map = super::super::parse(string).unwrap();

            let mut map_texture_info = TextureInfo::new();
            let mut materials = HashMap::new();

            // load all the textures since we will need their size then stuff them in materials
            for texture_name in map.get_texture_names() {
                let file = format!("textures/{}.png", texture_name);
                let bytes = load_context.read_asset_bytes(&file).await?;

                // load the texture and stick it in the AssetServer
                let mut texture = Image::from_buffer(
                    &bytes,
                    ImageType::Extension("png"),
                    CompressedImageFormats::all(),
                    false,
                )?;
                texture.sampler_descriptor = ImageSampler::Descriptor(texture_sampler());
                map_texture_info.add_texture(
                    &texture_name,
                    texture.texture_descriptor.size.width,
                    texture.texture_descriptor.size.height,
                );

                // create a material with texture
                let texture_handle = load_context
                    .set_labeled_asset(&format!("textures/{}.png", texture_name), LoadedAsset::new(texture));
                let material = StandardMaterial {
                    base_color_texture: Some(texture_handle.clone()),
                    alpha_mode: AlphaMode::Opaque,
                    ..default()
                };
                let material_handle =
                    load_context.set_labeled_asset(&format!("materials/{}", texture_name), LoadedAsset::new(material));
                materials.insert(texture_name.clone(), material_handle);
            }

            // build geometry
            let entity_geometry = map.build_entity_geometry(&map_texture_info);

            let collision_geometry: Vec<ConvexCollision> = entity_geometry
                .iter()
                .map(|geo| geo.get_convex_collision())
                .flatten() // without flattening we get a Vec per entity
                .collect();

            // build engine representation
            let mesh_surfaces: Vec<MeshSurface> = entity_geometry
                .iter()
                .map(|data| get_brush_entity_visual_geometry(&data))
                .flatten() // without flattening we get a Vec per entity
                .collect();

            let default_material_handle: Handle<StandardMaterial> =
                load_context.set_labeled_asset("valve_map_default", LoadedAsset::new(Color::rgb(1.0, 0.0, 1.0).into()));

            let mut world = World::default();
            for (i, mesh_surface) in mesh_surfaces.iter().enumerate() {
                let material = {
                    if let Some(tex_name) = &mesh_surface.texture {
                        materials.get(tex_name).unwrap().clone()
                    } else {
                        default_material_handle.clone()
                    }
                };

                let mesh = Mesh::from(mesh_surface);
                let mesh = load_context.set_labeled_asset(&format!("ValveMapMesh{}", i), LoadedAsset::new(mesh));

                world.spawn(PbrBundle {
                    mesh,
                    material,
                    transform: Transform::from_translation(mesh_surface.center_local(16.0)),
                    ..default()
                });
            }

            // let registry = world.resource_mut::<AppTypeRegistry>();
            // registry.write().register::<Collider>();

            // or use this: Collider::from_bevy_mesh
            for geo in collision_geometry {
                world.spawn(Collider::convex_hull(&geo.points).unwrap());
            }

            load_context.set_default_asset(LoadedAsset::new(Scene::new(world)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

fn texture_sampler<'a>() -> SamplerDescriptor<'a> {
    SamplerDescriptor {
        label: None,
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: Default::default(),
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: Default::default(),
        lod_min_clamp: 0.0,
        lod_max_clamp: std::f32::MAX,
        compare: None,
        anisotropy_clamp: None,
        border_color: None,
    }
}
