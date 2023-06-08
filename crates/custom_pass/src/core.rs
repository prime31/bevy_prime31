use std::cmp::Reverse;

use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_phase::{CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItem},
    render_resource::{CachedRenderPipelineId, Extent3d, TextureFormat},
    texture::CachedTexture,
};
use bevy::utils::FloatOrd;

pub const DEPTH_PREPASS_FORMAT: TextureFormat = TextureFormat::Depth32Float;
pub const NORMAL_PREPASS_FORMAT: TextureFormat = TextureFormat::Rgb10a2Unorm;

#[derive(Component, Default, Reflect, Clone, ExtractComponent)]
pub struct OcclusionPrepassLight;

#[derive(Component, Default, Reflect, Clone, ExtractComponent)]
pub struct OcclusionPrepassOccluder;

/// If added to a [`crate::prelude::Camera3d`] then depth values will be copied to a separate texture available to the main pass.
#[derive(Component, Default, Reflect)]
pub struct OcclusionDepthPrepass;

/// If added to a [`crate::prelude::Camera3d`] then vertex world normals will be copied to a separate texture available to the main pass.
/// Normals will have normal map textures already applied.
#[derive(Component, Default, Reflect)]
pub struct OcclusionNormalPrepass;

/// Textures that are written to by the prepass.
///
/// This component will only be present if any of the relevant prepass components are also present.
#[derive(Component)]
pub struct OcclusionViewPrepassTextures {
    /// The depth texture generated by the prepass.
    /// Exists only if [`DepthPrepass`] is added to the `ViewTarget`
    pub depth: Option<CachedTexture>,
    /// The normals texture generated by the prepass.
    /// Exists only if [`NormalPrepass`] is added to the `ViewTarget`
    pub normal: Option<CachedTexture>,
    /// The size of the textures.
    pub size: Extent3d,
}

/// Opaque phase of the 3D prepass.
///
/// Sorted front-to-back by the z-distance in front of the camera.
///
/// Used to render all 3D meshes with materials that have no transparency.
pub struct Opaque3dPrepass {
    pub distance: f32,
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for Opaque3dPrepass {
    // NOTE: Values increase towards the camera. Front-to-back ordering for opaque means we need a descending sort.
    type SortKey = Reverse<FloatOrd>;

    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        Reverse(FloatOrd(self.distance))
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        // Key negated to match reversed SortKey ordering
        radsort::sort_by_key(items, |item| -item.distance);
    }
}

impl CachedRenderPipelinePhaseItem for Opaque3dPrepass {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline_id
    }
}

/// Alpha mask phase of the 3D prepass.
///
/// Sorted front-to-back by the z-distance in front of the camera.
///
/// Used to render all meshes with a material with an alpha mask.
pub struct AlphaMask3dPrepass {
    pub distance: f32,
    pub entity: Entity,
    pub pipeline_id: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for AlphaMask3dPrepass {
    // NOTE: Values increase towards the camera. Front-to-back ordering for alpha mask means we need a descending sort.
    type SortKey = Reverse<FloatOrd>;

    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        Reverse(FloatOrd(self.distance))
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        // Key negated to match reversed SortKey ordering
        radsort::sort_by_key(items, |item| -item.distance);
    }
}

impl CachedRenderPipelinePhaseItem for AlphaMask3dPrepass {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline_id
    }
}
