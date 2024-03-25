use std::sync::Arc;

use rust_embed::RustEmbed;
use wgpu::*;

use glam::*;
use shader::ShaderConstants;
use winit::{event::Event, window::Window};

pub use crate::resources::Resources;
use crate::{
    glyph::GlyphState, path::PathState, quad::QuadState, scene::Layer, sprite::SpriteState, Scene,
};

pub trait Drawable {
    fn new(resources: &Resources) -> Self
    where
        Self: Sized;

    fn surface_updated(&mut self, resources: &Resources);

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    );
}

pub struct Renderer {
    pub(crate) resources: Resources,
    pub(crate) drawables: Vec<Box<dyn Drawable>>,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> Self {
        let resources = Resources::new(window).await;

        Self {
            resources,
            drawables: Vec::new(),
        }
    }

    pub fn with_drawable<T: Drawable + 'static>(mut self) -> Self {
        let drawable = T::new(&self.resources);
        self.drawables.push(Box::new(drawable));
        self
    }

    pub fn with_default_drawables<A: RustEmbed + 'static>(self) -> Self {
        self.with_drawable::<QuadState>()
            .with_drawable::<GlyphState>()
            .with_drawable::<PathState>()
            .with_drawable::<SpriteState<A>>()
    }

    pub fn draw_scene(&mut self, scene: &Scene) -> bool {
        if let Err(render_error) = self.resources.render(scene, self.drawables.as_mut_slice()) {
            eprintln!("Render error: {:?}", render_error);
            false
        } else {
            true
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        if self.resources.handle_event(event) {
            for drawable in self.drawables.iter_mut() {
                drawable.surface_updated(&self.resources);
            }
        }
    }
}
