pub mod core;
pub mod node;
pub mod phase_items;

use bevy::app::{IntoSystemAppConfig, Plugin};
use bevy::asset::{load_internal_asset, AssetServer, Handle, HandleUntyped};
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::prelude::Camera3d;
use bevy::ecs::{
    prelude::*,
    system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
};
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::render_graph::RenderGraph;

use bevy::render::{
    camera::ExtractedCamera,
    mesh::MeshVertexBufferLayout,
    prelude::{Camera, Mesh},
    render_asset::RenderAssets,
    render_phase::{
        sort_phase_system, AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase,
        SetItemPipeline, TrackedRenderPass,
    },
    render_resource::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
        BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType, ColorTargetState,
        ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Extent3d, FragmentState, FrontFace,
        MultisampleState, PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor, Shader, ShaderDefVal,
        ShaderRef, ShaderStages, ShaderType, SpecializedMeshPipeline, SpecializedMeshPipelineError,
        SpecializedMeshPipelines, StencilFaceState, StencilState, TextureDescriptor, TextureDimension, TextureFormat,
        TextureSampleType, TextureUsages, TextureViewDimension, VertexState,
    },
    renderer::RenderDevice,
    texture::{FallbackImagesDepth, FallbackImagesMsaa, TextureCache},
    view::{ExtractedView, Msaa, ViewUniform, ViewUniformOffset, ViewUniforms, VisibleEntities},
    Extract, ExtractSchedule, RenderApp, RenderSet,
};
use bevy::utils::{tracing::error, HashMap};

use bevy::pbr::{
    AlphaMode, DrawMesh, Material, MaterialPipeline, MaterialPipelineKey, MeshPipeline, MeshPipelineKey, MeshUniform,
    RenderMaterials, SetMaterialBindGroup, SetMeshBindGroup, MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS,
};
use node::OcclusionPrepassNode;
use phase_items::{CustomLightOpaque3dPrepass, CustomOpaque3dPrepass};

use crate::core::{OcclusionDepthPrepass, OcclusionNormalPrepass, NORMAL_PREPASS_FORMAT};
use crate::core::{OcclusionPrepassLight, OcclusionPrepassOccluder};
use crate::core::{OcclusionViewPrepassTextures, DEPTH_PREPASS_FORMAT};
use std::{hash::Hash, marker::PhantomData};

pub const PREPASS_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 921124473254008984);

pub const PREPASS_BINDINGS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5533152893177403495);

pub const PREPASS_UTILS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4603948296044545);

pub struct OcclusionPrepassPlugin;

impl Plugin for OcclusionPrepassPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<OcclusionPrepassLight>::default());
        app.add_plugin(ExtractComponentPlugin::<OcclusionPrepassOccluder>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        let prepass_node = OcclusionPrepassNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let core_3d_graph = graph.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

        // add ourself to the core 3d graph
        core_3d_graph.add_node(OcclusionPrepassNode::NAME, prepass_node);

        core_3d_graph.add_slot_edge(
            core_3d_graph.input_node().id,
            core_3d::graph::input::VIEW_ENTITY,
            OcclusionPrepassNode::NAME,
            OcclusionPrepassNode::IN_VIEW,
        );

        // add node edges so we run after PREPASS and before MAIN_PASS
        core_3d_graph.add_node_edge(core_3d::graph::node::PREPASS, OcclusionPrepassNode::NAME);
        core_3d_graph.add_node_edge(OcclusionPrepassNode::NAME, core_3d::graph::node::MAIN_PASS);
    }
}

/// Sets up everything required to use the prepass pipeline.
///
/// This does not add the actual prepasses, see [`PrepassPlugin`] for that.
pub struct PrepassPipelinePlugin<M: Material>(PhantomData<M>);

impl<M: Material> Default for PrepassPipelinePlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: Material> Plugin for PrepassPipelinePlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut bevy::app::App) {
        load_internal_asset!(app, PREPASS_SHADER_HANDLE, "prepass.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            PREPASS_BINDINGS_SHADER_HANDLE,
            "prepass_bindings.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PREPASS_UTILS_SHADER_HANDLE,
            "prepass_utils.wgsl",
            Shader::from_wgsl
        );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
                return;
            };

        render_app
            .add_system(queue_prepass_view_bind_group::<M>.in_set(RenderSet::Queue))
            .init_resource::<OcclusionPrepassPipeline<M>>()
            .init_resource::<OcclusionPrepassViewBindGroup>()
            .init_resource::<SpecializedMeshPipelines<OcclusionPrepassPipeline<M>>>();
    }
}

/// Sets up the prepasses for a [`Material`].
///
/// This depends on the [`PrepassPipelinePlugin`].
pub struct PrepassPlugin<M: Material>(PhantomData<M>);

impl<M: Material> Default for PrepassPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: Material> Plugin for PrepassPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut bevy::app::App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_system(extract_camera_prepass_phase.in_schedule(ExtractSchedule))
            .add_system(
                prepare_prepass_textures
                    .in_set(RenderSet::Prepare)
                    .after(bevy::render::view::prepare_windows),
            )
            .add_system(queue_prepass_material_meshes::<M>.in_set(RenderSet::Queue))
            .add_system(sort_phase_system::<CustomOpaque3dPrepass>.in_set(RenderSet::PhaseSort))
            .add_system(sort_phase_system::<CustomLightOpaque3dPrepass>.in_set(RenderSet::PhaseSort))
            .init_resource::<DrawFunctions<CustomOpaque3dPrepass>>()
            .init_resource::<DrawFunctions<CustomLightOpaque3dPrepass>>()
            .add_render_command::<CustomOpaque3dPrepass, DrawOcclusionPrepass<M>>()
            .add_render_command::<CustomLightOpaque3dPrepass, DrawOcclusionPrepass<M>>();
    }
}

#[derive(Resource)]
pub struct OcclusionPrepassPipeline<M: Material> {
    pub view_layout: BindGroupLayout,
    pub mesh_layout: BindGroupLayout,
    pub skinned_mesh_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
    pub material_vertex_shader: Option<Handle<Shader>>,
    pub material_fragment_shader: Option<Handle<Shader>>,
    pub material_pipeline: MaterialPipeline<M>,
    _marker: PhantomData<M>,
}

impl<M: Material> FromWorld for OcclusionPrepassPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("prepass_view_layout"),
        });

        let mesh_pipeline = world.resource::<MeshPipeline>();

        OcclusionPrepassPipeline {
            view_layout,
            mesh_layout: mesh_pipeline.mesh_layout.clone(),
            skinned_mesh_layout: mesh_pipeline.skinned_mesh_layout.clone(),
            material_vertex_shader: match M::prepass_vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            material_fragment_shader: match M::prepass_fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            material_layout: M::bind_group_layout(render_device),
            material_pipeline: world.resource::<MaterialPipeline<M>>().clone(),
            _marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedMeshPipeline for OcclusionPrepassPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut bind_group_layout = vec![self.view_layout.clone()];
        let mut shader_defs = Vec::new();
        let mut vertex_attributes = Vec::new();

        // NOTE: Eventually, it would be nice to only add this when the shaders are overloaded by the Material.
        // The main limitation right now is that bind group order is hardcoded in shaders.
        bind_group_layout.insert(1, self.material_layout.clone());

        if key.mesh_key.contains(MeshPipelineKey::DEPTH_PREPASS) {
            shader_defs.push("DEPTH_PREPASS".into());
        }

        if key.mesh_key.contains(MeshPipelineKey::ALPHA_MASK) {
            shader_defs.push("ALPHA_MASK".into());
        }

        let blend_key = key.mesh_key.intersection(MeshPipelineKey::BLEND_RESERVED_BITS);
        if blend_key == MeshPipelineKey::BLEND_PREMULTIPLIED_ALPHA {
            shader_defs.push("BLEND_PREMULTIPLIED_ALPHA".into());
        }
        if blend_key == MeshPipelineKey::BLEND_ALPHA {
            shader_defs.push("BLEND_ALPHA".into());
        }

        if layout.contains(Mesh::ATTRIBUTE_POSITION) {
            shader_defs.push("VERTEX_POSITIONS".into());
            vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }

        shader_defs.push(ShaderDefVal::Int(
            "MAX_DIRECTIONAL_LIGHTS".to_string(),
            MAX_DIRECTIONAL_LIGHTS as i32,
        ));
        shader_defs.push(ShaderDefVal::Int(
            "MAX_CASCADES_PER_LIGHT".to_string(),
            MAX_CASCADES_PER_LIGHT as i32,
        ));
        if key.mesh_key.contains(MeshPipelineKey::DEPTH_CLAMP_ORTHO) {
            shader_defs.push("DEPTH_CLAMP_ORTHO".into());
        }

        if layout.contains(Mesh::ATTRIBUTE_UV_0) {
            shader_defs.push("VERTEX_UVS".into());
            vertex_attributes.push(Mesh::ATTRIBUTE_UV_0.at_shader_location(1));
        }

        if key.mesh_key.contains(MeshPipelineKey::NORMAL_PREPASS) {
            vertex_attributes.push(Mesh::ATTRIBUTE_NORMAL.at_shader_location(2));
            shader_defs.push("NORMAL_PREPASS".into());

            if layout.contains(Mesh::ATTRIBUTE_TANGENT) {
                shader_defs.push("VERTEX_TANGENTS".into());
                vertex_attributes.push(Mesh::ATTRIBUTE_TANGENT.at_shader_location(3));
            }
        }

        if layout.contains(Mesh::ATTRIBUTE_JOINT_INDEX) && layout.contains(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
            shader_defs.push("SKINNED".into());
            vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(4));
            vertex_attributes.push(Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(5));
            bind_group_layout.insert(2, self.skinned_mesh_layout.clone());
        } else {
            bind_group_layout.insert(2, self.mesh_layout.clone());
        }

        let vertex_buffer_layout = layout.get_layout(&vertex_attributes)?;

        // The fragment shader is only used when the normal prepass is enabled
        // or the material uses alpha cutoff values and doesn't rely on the standard prepass shader
        let fragment = if key.mesh_key.contains(MeshPipelineKey::NORMAL_PREPASS)
            || ((key.mesh_key.contains(MeshPipelineKey::ALPHA_MASK)
                || blend_key == MeshPipelineKey::BLEND_PREMULTIPLIED_ALPHA
                || blend_key == MeshPipelineKey::BLEND_ALPHA)
                && self.material_fragment_shader.is_some())
        {
            // Use the fragment shader from the material if present
            let frag_shader_handle = if let Some(handle) = &self.material_fragment_shader {
                handle.clone()
            } else {
                PREPASS_SHADER_HANDLE.typed::<Shader>()
            };

            let mut targets = vec![];
            // When the normal prepass is enabled we need a target to be able to write to it.
            if key.mesh_key.contains(MeshPipelineKey::NORMAL_PREPASS) {
                targets.push(Some(ColorTargetState {
                    format: TextureFormat::Rgb10a2Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }));
            }

            Some(FragmentState {
                shader: frag_shader_handle,
                entry_point: "fragment".into(),
                shader_defs: shader_defs.clone(),
                targets,
            })
        } else {
            None
        };

        // Use the vertex shader from the material if present
        let vert_shader_handle = if let Some(handle) = &self.material_vertex_shader {
            handle.clone()
        } else {
            PREPASS_SHADER_HANDLE.typed::<Shader>()
        };

        let mut descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                shader: vert_shader_handle,
                entry_point: "vertex".into(),
                shader_defs,
                buffers: vec![vertex_buffer_layout],
            },
            fragment,
            layout: bind_group_layout,
            primitive: PrimitiveState {
                topology: key.mesh_key.primitive_topology(),
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_PREPASS_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.mesh_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            push_constant_ranges: Vec::new(),
            label: Some("prepass_pipeline".into()),
        };

        // This is a bit risky because it's possible to change something that would
        // break the prepass but be fine in the main pass.
        // Since this api is pretty low-level it doesn't matter that much, but it is a potential issue.
        M::specialize(&self.material_pipeline, &mut descriptor, layout, key)?;

        Ok(descriptor)
    }
}

pub fn get_bind_group_layout_entries(bindings: [u32; 2], multisampled: bool) -> [BindGroupLayoutEntry; 2] {
    [
        // Depth texture
        BindGroupLayoutEntry {
            binding: bindings[0],
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled,
                sample_type: TextureSampleType::Depth,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        },
        // Normal texture
        BindGroupLayoutEntry {
            binding: bindings[1],
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled,
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        },
    ]
}

pub fn get_bindings<'a>(
    prepass_textures: Option<&'a OcclusionViewPrepassTextures>,
    fallback_images: &'a mut FallbackImagesMsaa,
    fallback_depths: &'a mut FallbackImagesDepth,
    msaa: &'a Msaa,
    bindings: [u32; 2],
) -> [BindGroupEntry<'a>; 2] {
    let depth_view = match prepass_textures.and_then(|x| x.depth.as_ref()) {
        Some(texture) => &texture.default_view,
        None => &fallback_depths.image_for_samplecount(msaa.samples()).texture_view,
    };

    let normal_view = match prepass_textures.and_then(|x| x.normal.as_ref()) {
        Some(texture) => &texture.default_view,
        None => &fallback_images.image_for_samplecount(msaa.samples()).texture_view,
    };

    [
        BindGroupEntry {
            binding: bindings[0],
            resource: BindingResource::TextureView(depth_view),
        },
        BindGroupEntry {
            binding: bindings[1],
            resource: BindingResource::TextureView(normal_view),
        },
    ]
}

// Extract the render phases for the prepass
pub fn extract_camera_prepass_phase(
    mut commands: Commands,
    cameras_3d: Extract<
        Query<
            (
                Entity,
                &Camera,
                Option<&OcclusionDepthPrepass>,
                Option<&OcclusionNormalPrepass>,
            ),
            With<Camera3d>,
        >,
    >,
) {
    for (entity, camera, depth_prepass, normal_prepass) in cameras_3d.iter() {
        if !camera.is_active {
            continue;
        }

        let mut entity = commands.get_or_spawn(entity);
        if depth_prepass.is_some() || normal_prepass.is_some() {
            entity.insert(RenderPhase::<CustomOpaque3dPrepass>::default());
            entity.insert(RenderPhase::<CustomLightOpaque3dPrepass>::default());
        }
        if depth_prepass.is_some() {
            entity.insert(OcclusionDepthPrepass);
        }
        if normal_prepass.is_some() {
            entity.insert(OcclusionNormalPrepass);
        }
    }
}

// Prepares the textures used by the prepass
pub fn prepare_prepass_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    views_3d: Query<
        (
            Entity,
            &ExtractedCamera,
            Option<&OcclusionDepthPrepass>,
            Option<&OcclusionNormalPrepass>,
        ),
        With<RenderPhase<CustomOpaque3dPrepass>>,
    >,
) {
    let mut depth_textures = HashMap::default();
    let mut normal_textures = HashMap::default();
    for (entity, camera, depth_prepass, normal_prepass) in &views_3d {
        let Some(physical_target_size) = camera.physical_target_size else {
            continue;
        };

        let size = Extent3d {
            depth_or_array_layers: 1,
            width: physical_target_size.x,
            height: physical_target_size.y,
        };

        let cached_depth_texture = depth_prepass.is_some().then(|| {
            depth_textures
                .entry(camera.target.clone())
                .or_insert_with(|| {
                    let descriptor = TextureDescriptor {
                        label: Some("prepass_depth_texture"),
                        size,
                        mip_level_count: 1,
                        sample_count: msaa.samples(),
                        dimension: TextureDimension::D2,
                        format: DEPTH_PREPASS_FORMAT,
                        usage: TextureUsages::COPY_DST
                            | TextureUsages::RENDER_ATTACHMENT
                            | TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    };
                    texture_cache.get(&render_device, descriptor)
                })
                .clone()
        });

        let cached_normals_texture = normal_prepass.is_some().then(|| {
            normal_textures
                .entry(camera.target.clone())
                .or_insert_with(|| {
                    texture_cache.get(
                        &render_device,
                        TextureDescriptor {
                            label: Some("prepass_normal_texture"),
                            size,
                            mip_level_count: 1,
                            sample_count: msaa.samples(),
                            dimension: TextureDimension::D2,
                            format: NORMAL_PREPASS_FORMAT,
                            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        },
                    )
                })
                .clone()
        });

        commands.entity(entity).insert(OcclusionViewPrepassTextures {
            depth: cached_depth_texture,
            normal: cached_normals_texture,
            size,
        });
    }
}

#[derive(Default, Resource)]
pub struct OcclusionPrepassViewBindGroup {
    bind_group: Option<BindGroup>,
}

pub fn queue_prepass_view_bind_group<M: Material>(
    render_device: Res<RenderDevice>,
    prepass_pipeline: Res<OcclusionPrepassPipeline<M>>,
    view_uniforms: Res<ViewUniforms>,
    mut prepass_view_bind_group: ResMut<OcclusionPrepassViewBindGroup>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        prepass_view_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("occlusion_prepass_view_bind_group"),
            layout: &prepass_pipeline.view_layout,
        }));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_prepass_material_meshes<M: Material>(
    opaque_draw_functions: Res<DrawFunctions<CustomOpaque3dPrepass>>,
    prepass_pipeline: Res<OcclusionPrepassPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<OcclusionPrepassPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<RenderMaterials<M>>,
    material_meshes: Query<(&Handle<M>, &Handle<Mesh>, &MeshUniform)>,
    occluder_components: Query<(Option<&OcclusionPrepassLight>, Option<&OcclusionPrepassOccluder>)>,
    mut views: Query<(
        &ExtractedView,
        &VisibleEntities,
        &mut RenderPhase<CustomOpaque3dPrepass>,
        &mut RenderPhase<CustomLightOpaque3dPrepass>,
        Option<&OcclusionDepthPrepass>,
        Option<&OcclusionNormalPrepass>,
    )>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    println!("-- queue_prepass_material_meshes --");
    let opaque_draw_prepass = opaque_draw_functions
        .read()
        .get_id::<DrawOcclusionPrepass<M>>()
        .unwrap();
    for (view, visible_entities, mut opaque_phase, mut light_opaque_phase, depth_prepass, normal_prepass) in &mut views
    {
        let mut view_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        if depth_prepass.is_some() {
            view_key |= MeshPipelineKey::DEPTH_PREPASS;
        }
        if normal_prepass.is_some() {
            view_key |= MeshPipelineKey::NORMAL_PREPASS;
        }

        let rangefinder = view.rangefinder3d();

        for visible_entity in &visible_entities.entities {
            let Ok((material_handle, mesh_handle, mesh_uniform)) = material_meshes.get(*visible_entity) else {
                continue;
            };

            let (Some(material), Some(mesh)) = (
                render_materials.get(material_handle),
                render_meshes.get(mesh_handle),
            ) else {
                continue;
            };

            let Ok((is_light, is_occluder)) = occluder_components.get(*visible_entity) else {
                println!("------ fuuuuuck nothing found");
                continue;
            };
            println!(
                "------ maybe found something extracted. entity: {:?}, light: {:?}, occluder: {:?}",
                visible_entity,
                is_light.is_some(),
                is_occluder.is_some()
            );

            let mut mesh_key = MeshPipelineKey::from_primitive_topology(mesh.primitive_topology) | view_key;
            let alpha_mode = material.properties.alpha_mode;
            match alpha_mode {
                AlphaMode::Opaque => {}
                AlphaMode::Mask(_) => mesh_key |= MeshPipelineKey::ALPHA_MASK,
                AlphaMode::Blend | AlphaMode::Premultiplied | AlphaMode::Add | AlphaMode::Multiply => continue,
            }

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &prepass_pipeline,
                MaterialPipelineKey {
                    mesh_key,
                    bind_group_data: material.key.clone(),
                },
                &mesh.layout,
            );
            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };

            let distance = rangefinder.distance(&mesh_uniform.transform) + material.properties.depth_bias;
            match alpha_mode {
                AlphaMode::Opaque => {
                    if is_occluder.is_some() {
                        opaque_phase.add(CustomOpaque3dPrepass {
                            entity: *visible_entity,
                            draw_function: opaque_draw_prepass,
                            pipeline_id,
                            distance,
                        });
                    }

                    if is_light.is_some() {
                        light_opaque_phase.add(CustomLightOpaque3dPrepass {
                            entity: *visible_entity,
                            draw_function: opaque_draw_prepass,
                            pipeline_id,
                            distance,
                        });
                    }
                }
                AlphaMode::Mask(_) => todo!(),
                AlphaMode::Blend | AlphaMode::Premultiplied | AlphaMode::Add | AlphaMode::Multiply => {}
            }
        }
    }
}

pub struct SetPrepassViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetPrepassViewBindGroup<I> {
    type Param = SRes<OcclusionPrepassViewBindGroup>;
    type ViewWorldQuery = Read<ViewUniformOffset>;
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        view_uniform_offset: &'_ ViewUniformOffset,
        _entity: (),
        prepass_view_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let prepass_view_bind_group = prepass_view_bind_group.into_inner();
        pass.set_bind_group(
            I,
            prepass_view_bind_group.bind_group.as_ref().unwrap(),
            &[view_uniform_offset.offset],
        );
        RenderCommandResult::Success
    }
}

pub type DrawOcclusionPrepass<M> = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    SetMaterialBindGroup<M, 1>,
    SetMeshBindGroup<2>,
    DrawMesh,
);
