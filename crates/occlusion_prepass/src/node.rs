use bevy::ecs::prelude::*;
use bevy::ecs::query::QueryState;
use bevy::render::{
    camera::ExtractedCamera,
    prelude::Color,
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_phase::RenderPhase,
    render_resource::{
        LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    },
    renderer::RenderContext,
    view::ExtractedView,
};

use crate::core::OcclusionViewPrepassTextures;

use super::{AlphaMask3dPrepass, Opaque3dPrepass};

/// Render node used by the prepass.
///
/// By default, inserted before the main pass in the render graph.
pub struct OcclusionPrepassNode {
    main_view_query: QueryState<
        (
            &'static ExtractedCamera,
            &'static RenderPhase<Opaque3dPrepass>,
            &'static RenderPhase<AlphaMask3dPrepass>,
            &'static OcclusionViewPrepassTextures,
        ),
        With<ExtractedView>,
    >,
}

impl OcclusionPrepassNode {
    pub const IN_VIEW: &'static str = "view";
    pub const NAME: &str = "occlusion_prepass";

    pub fn new(world: &mut World) -> Self {
        Self {
            main_view_query: QueryState::new(world),
        }
    }
}

impl Node for OcclusionPrepassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let Ok((
            camera,
            opaque_prepass_phase,
            alpha_mask_prepass_phase,
            view_prepass_textures,
        )) = self.main_view_query.get_manual(world, view_entity) else {
            println!("------- failed to run, no matching entities");
            return Ok(());
        };

        println!("------- run");

        let mut color_attachments = vec![];
        if let Some(view_normals_texture) = &view_prepass_textures.normal {
            println!("----- has normal");
            color_attachments.push(Some(RenderPassColorAttachment {
                view: &view_normals_texture.default_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK.into()),
                    store: true,
                },
            }));
        }

        // TODO: should depth be Option?
        if let Some(view_depth_texture) = &view_prepass_textures.depth {
            // Set up the pass descriptor with the depth attachment and optional color attachments
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("occlusion_prepass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &view_depth_texture.default_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if let Some(viewport) = camera.viewport.as_ref() {
                render_pass.set_camera_viewport(viewport);
            }

            // Always run opaque pass to ensure screen is cleared
            {
                // Run the prepass, sorted front-to-back
                opaque_prepass_phase.render(&mut render_pass, world, view_entity);
            }

            if !alpha_mask_prepass_phase.items.is_empty() {
                // Run the prepass, sorted front-to-back
                alpha_mask_prepass_phase.render(&mut render_pass, world, view_entity);
            }
        }

        Ok(())
    }
}
