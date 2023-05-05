use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::{CompressedImageFormats, ImageSampler, ImageType},
    },
    utils::{BoxedFuture, HashMap},
};

use crate::{
    convert::{get_brush_entity_visual_geometry, MeshSurface},
    formats::shared::Fields,
    generate::{ConvexCollision, TextureInfo},
};

use super::ValveMap;

#[derive(Debug)]
pub struct ValveMapEntity {
    pub fields: Fields,
    pub collision_geometry: Vec<ConvexCollision>,
    pub visual_geometry: Vec<ValveMapVisualGeometry>,
}

impl ValveMapEntity {
    fn new(fields: Fields, collision_geometry: Vec<ConvexCollision>) -> ValveMapEntity {
        ValveMapEntity {
            fields,
            visual_geometry: Vec::new(),
            collision_geometry,
        }
    }
}

#[derive(Debug)]
pub struct ValveMapVisualGeometry {
    pub origin: Vec3,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl ValveMapVisualGeometry {
    fn new(origin: Vec3, mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> ValveMapVisualGeometry {
        ValveMapVisualGeometry { origin, mesh, material }
    }
}

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

            // build general geometry which will be used to generate Meshes and Colliders
            let entity_geometry = map.build_entity_geometry(&map_texture_info);

            // build collision geometry, a Vec of ConvexCollision per entity
            let collision_geometry: Vec<Vec<ConvexCollision>> =
                entity_geometry.iter().map(|geo| geo.get_convex_collision()).collect();

            // build visual geometry, a Vec of MeshSurfaces per entity
            let mesh_surfaces: Vec<Vec<MeshSurface>> = entity_geometry
                .iter()
                .enumerate()
                .map(|(i, geo)| {
                    if map.entities[i].fields.is_sensor() {
                        Vec::new()
                    } else {
                        get_brush_entity_visual_geometry(&geo)
                    }
                })
                .collect();

            let default_material_handle: Handle<StandardMaterial> =
                load_context.set_labeled_asset("valve_map_default", LoadedAsset::new(Color::rgb(1.0, 0.0, 1.0).into()));

            // collect all our bevy handles and data per entity
            let mut entities: Vec<ValveMapEntity> = map
                .entities
                .iter()
                .zip(collision_geometry)
                .map(|(e, cg)| ValveMapEntity::new(e.fields.clone(), cg))
                .collect();
            for (i, mesh_surfaces) in mesh_surfaces.iter().enumerate() {
                for (j, surface) in mesh_surfaces.iter().enumerate() {
                    let material = {
                        if let Some(tex_name) = &surface.texture {
                            materials.get(tex_name).unwrap().clone()
                        } else {
                            default_material_handle.clone()
                        }
                    };

                    let mesh = Mesh::from(surface);
                    let mesh_handle =
                        load_context.set_labeled_asset(&format!("ValveMapMesh{}_{}", i, j), LoadedAsset::new(mesh));

                    entities[i].visual_geometry.push(ValveMapVisualGeometry::new(
                        surface.center_local(16.0),
                        mesh_handle.clone(),
                        material.clone(),
                    ));
                }
            }

            let valve_map = ValveMap { entities };
            load_context.set_default_asset(LoadedAsset::new(valve_map));

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
