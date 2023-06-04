use bevy::{
    prelude::*,
    render::render_resource::{Texture, TextureView},
};

#[derive(Component)]
pub struct ViewOcclusionTexture {
    pub texture: Texture,
    pub view: TextureView,
}
