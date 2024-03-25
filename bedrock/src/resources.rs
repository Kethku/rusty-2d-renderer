use std::sync::Arc;

use glam::{vec2, Vec4};
use shader::ShaderConstants;
use wgpu::*;
use winit::{event::Event, window::Window};

use crate::{
    renderer::Drawable, surface_wrapper::SurfaceResourcesManager, Asset, Scene, ATLAS_SIZE,
};

pub struct Resources {
    pub window: Arc<Window>,
    pub instance: Instance,
    pub surface_resources_manager: SurfaceResourcesManager,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub shader: ShaderModule,
    pub sampler: Sampler,
    pub universal_bind_group_layout: BindGroupLayout,
}

impl Resources {
    pub async fn new(window: Arc<Window>) -> Self {
        // The instance is a handle to our GPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::PUSH_CONSTANTS
                        | Features::SPIRV_SHADER_PASSTHROUGH
                        | Features::VERTEX_WRITABLE_STORAGE
                        | Features::CLEAR_TEXTURE,
                    required_limits: Limits {
                        max_push_constant_size: 256,
                        ..Default::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: util::make_spirv(
                &Asset::get("shader.spv")
                    .expect("Could not load shader")
                    .data,
            ),
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let universal_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Universal bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            window,
            instance,
            surface_resources_manager: SurfaceResourcesManager::new(),
            adapter,
            device,
            queue,
            shader,
            sampler,
            universal_bind_group_layout,
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> bool {
        self.surface_resources_manager.handle_event(
            event,
            self.window.clone(),
            &self.instance,
            &self.adapter,
            &self.device,
            &self.sampler,
            &self.universal_bind_group_layout,
            false,
        )
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        drawables: &mut [Box<dyn Drawable>],
    ) -> Result<(), SurfaceError> {
        let frame = self.surface_resources_manager.surface_texture(
            &self.device,
            &self.sampler,
            &self.universal_bind_group_layout,
        );

        let frame_view = frame.texture.create_view(&Default::default());
        let multisampled_view = self
            .surface_resources_manager
            .multisampled_texture()
            .create_view(&Default::default());

        let constants = ShaderConstants {
            surface_size: vec2(frame.texture.width() as f32, frame.texture.height() as f32),
            atlas_size: ATLAS_SIZE,
            clip: Vec4::ZERO,
        };

        let mut first = true;
        for layer in scene.layers.iter() {
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            for drawable in drawables.iter_mut() {
                // Either clear the offscreen texture or copy the previous layer to it
                if first {
                    encoder.clear_texture(
                        self.surface_resources_manager.offscreen_texture(),
                        &ImageSubresourceRange {
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    );
                } else {
                    encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &frame.texture,
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                            aspect: Default::default(),
                        },
                        ImageCopyTexture {
                            texture: self.surface_resources_manager.offscreen_texture(),
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                            aspect: Default::default(),
                        },
                        Extent3d {
                            width: frame.texture.width(),
                            height: frame.texture.height(),
                            depth_or_array_layers: 1,
                        },
                    );
                }

                // The first drawable should clear the output texture
                let attachment_op = if first {
                    Operations::<Color> {
                        load: LoadOp::<_>::Clear(Color::WHITE),
                        store: StoreOp::Store,
                    }
                } else {
                    Operations::<Color> {
                        load: LoadOp::<_>::Load,
                        store: StoreOp::Store,
                    }
                };

                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &multisampled_view,
                        resolve_target: Some(&frame_view),
                        ops: attachment_op,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                if let Some(clip) = layer.clip {
                    render_pass.set_scissor_rect(
                        clip.x.max(0.0) as u32,
                        clip.y.max(0.0) as u32,
                        (clip.z as u32).min(frame.texture.width()),
                        (clip.w as u32).min(frame.texture.height()),
                    );
                }

                drawable.draw(
                    &self.queue,
                    &mut render_pass,
                    constants,
                    self.surface_resources_manager.universal_bind_group(),
                    &layer,
                );

                first = false;
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }

        frame.present();

        Ok(())
    }
}
