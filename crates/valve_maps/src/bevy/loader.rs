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
    convert::{quake_point_to_bevy_point, MeshSurface},
    formats::shared::Fields,
    generate::{ConvexCollision, Geometry, TextureInfo},
};

use super::ValveMap;

#[derive(Debug)]
pub struct ValveMapEntity {
    pub fields: Fields,
    pub collision_geometry: Vec<ConvexCollision>,
    pub visual_geometry: Vec<VisualGeometry>,
}

impl ValveMapEntity {
    fn new(fields: Fields, collision_geometry: Vec<ConvexCollision>) -> ValveMapEntity {
        ValveMapEntity {
            fields,
            visual_geometry: Vec::new(),
            collision_geometry,
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&str> {
        if let Some(s) = self.fields.get(&String::from(name)) {
            return Some(&s[..]);
        }
        None
    }

    pub fn is_sensor(&self) -> bool {
        if let Some(prop) = self.fields.get("classname") {
            return prop == "sensor";
        }
        false
    }

    pub fn get_bool_property(&self, name: &str) -> Option<bool> {
        if let Some(prop) = self.fields.get(name) {
            return Some(prop == "1");
        }
        None
    }

    pub fn get_f32_property(&self, name: &str) -> Option<f32> {
        if let Some(prop) = self.fields.get(name) {
            return Some(prop.parse().unwrap_or(0.0));
        }
        None
    }

    pub fn get_vec3_property(&self, name: &str) -> Option<Vec3> {
        if let Some(prop) = self.fields.get(name) {
            let mut comps = prop.split(' ');
            let x: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            let y: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            let z: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            return Some(quake_point_to_bevy_point(Vec3::new(x, y, z), 16.0));
        }
        None
    }

    pub fn get_vec3_property_raw(&self, name: &str) -> Option<Vec3> {
        if let Some(prop) = self.fields.get(name) {
            let mut comps = prop.split(' ');
            let x: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            let y: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            let z: f32 = comps.next().unwrap_or("0.0").parse().unwrap_or(0.0);
            return Some(Vec3::new(x, y, z));
        }
        None
    }

    pub fn get_color_property(&self, name: &str) -> Option<Color> {
        if let Some(prop) = self.fields.get(name) {
            let mut comps = prop.split(' ');
            let r: u8 = comps.next().unwrap_or("255").parse().unwrap_or(255);
            let g: u8 = comps.next().unwrap_or("255").parse().unwrap_or(0);
            let b: u8 = comps.next().unwrap_or("255").parse().unwrap_or(255);
            return Some(Color::rgb_u8(r, g, b));
        }
        None
    }
}

#[derive(Debug)]
pub struct VisualGeometry {
    pub origin: Vec3,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl VisualGeometry {
    fn new(origin: Vec3, mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> VisualGeometry {
        VisualGeometry { origin, mesh, material }
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
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
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

/// loads all Textures and creates a StandardMaterial per Texture. Grabs the texture dimensions as well for uv calculations
async fn load_textures(
    map: &crate::Map,
    load_context: &mut LoadContext<'_>,
) -> Result<(TextureInfo, HashMap<String, Handle<StandardMaterial>>), bevy::asset::Error> {
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
        let texture_handle = load_context.set_labeled_asset(&file, LoadedAsset::new(texture));

        let material = StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        };

        let material_handle =
            load_context.set_labeled_asset(&format!("materials/{}", texture_name), LoadedAsset::new(material));
        materials.insert(texture_name.clone(), material_handle);
    }

    Ok((map_texture_info, materials))
}

async fn load_obj<'a, 'b>(bytes: &'a [u8], load_context: &'a mut LoadContext<'b>) -> Result<(), bevy::asset::Error> {
    let string = std::str::from_utf8(bytes)?;
    let map = super::super::parse(string).unwrap();

    // load all the textures since we will need their size then stuff them in materials
    let (map_texture_info, materials) = load_textures(&map, load_context).await?;

    // build general geometry which will be used to generate Meshes and Colliders
    let entity_geometry = map.build_entity_geometry(&map_texture_info);

    // build collision geometry, a Vec of ConvexCollision per entity
    let collision_geometry: Vec<Vec<ConvexCollision>> =
        entity_geometry.iter().map(Geometry::get_collision_geometry).collect();

    // build visual geometry, a Vec of MeshSurfaces per entity
    let mesh_surfaces: Vec<Vec<MeshSurface>> = entity_geometry
        .iter()
        .enumerate()
        .map(
            |(i, geo)| {
                if map.entities[i].fields.is_sensor() {
                    Vec::new()
                } else {
                    geo.get_visual_geometry()
                }
            },
        )
        .collect();

    let default_material_handle: Handle<StandardMaterial> =
        load_context.set_labeled_asset("valve_map_default", LoadedAsset::new(Color::rgb(1.0, 0.0, 1.0).into()));

    // collect all our bevy handles and data per entity
    let mut entities: Vec<ValveMapEntity> = map
        .entities
        .into_iter()
        .zip(collision_geometry)
        .map(|(e, cg)| ValveMapEntity::new(e.fields, cg))
        .collect();

    for (i, mesh_surface) in mesh_surfaces.iter().enumerate() {
        for (j, surface) in mesh_surface.iter().enumerate() {
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

            entities[i].visual_geometry.push(VisualGeometry::new(
                surface.center(),
                mesh_handle.clone(),
                material.clone(),
            ));
        }
    }

    let valve_map = ValveMap { entities };
    load_context.set_default_asset(LoadedAsset::new(valve_map));

    Ok(())
}
