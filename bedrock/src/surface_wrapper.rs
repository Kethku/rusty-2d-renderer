use std::sync::Arc;

use wgpu::*;
use winit::{
    event::{Event, StartCause, WindowEvent},
    window::Window,
};

pub struct SurfaceResources {
    surface: Surface<'static>,
    offscreen_texture: Texture,
    multisampled_texture: Texture,
    universal_bind_group: BindGroup,
}

impl SurfaceResources {
    pub fn new(
        device: &Device,
        sampler: &Sampler,
        surface: Surface<'static>,
        config: &SurfaceConfiguration,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        surface.configure(device, &config);
        let offscreen_texture = create_texture(
            device,
            config.width,
            config.height,
            config.format,
            1,
            "Offscreen Texture",
        );
        let multisampled_texture = create_texture(
            device,
            config.width,
            config.height,
            config.format,
            4,
            "Output Texture",
        );

        let universal_bind_group = create_bind_group(
            device,
            &offscreen_texture,
            sampler,
            universal_bind_group_layout,
        );

        Self {
            surface,
            offscreen_texture,
            multisampled_texture,
            universal_bind_group,
        }
    }

    fn acquire(&self) -> Result<SurfaceTexture, SurfaceError> {
        match self.surface.get_current_texture() {
            Ok(frame) => Ok(frame),
            // If we timed out, just try again
            Err(SurfaceError::Timeout) => Ok(self
                .surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture")),
            Err(e) => Err(e),
        }
    }
}

// Wrapper for the wgpu surface and configuration taken from the wgpu example code
pub struct SurfaceResourcesManager {
    surface_resources: Option<SurfaceResources>,
    config: Option<SurfaceConfiguration>,
}

impl SurfaceResourcesManager {
    pub fn new() -> Self {
        Self {
            surface_resources: None,
            config: None,
        }
    }

    pub fn surface_texture(
        &mut self,
        device: &Device,
        sampler: &Sampler,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> SurfaceTexture {
        match self.surface_resources.as_ref().unwrap().acquire() {
            Ok(frame) => frame,
            Err(SurfaceError::Outdated | SurfaceError::Lost | SurfaceError::OutOfMemory) => {
                let surface = self.surface_resources.take().unwrap().surface;
                let config = self.config.as_ref().unwrap();
                self.surface_resources = Some(SurfaceResources::new(
                    device,
                    sampler,
                    surface,
                    config,
                    universal_bind_group_layout,
                ));
                self.surface_resources
                    .as_ref()
                    .unwrap()
                    .acquire()
                    .expect("Could not acquire next surface texture after reconfiguring")
            }
            Err(e) => panic!("Unexpected surface error: {:?}", e),
        }
    }

    pub fn offscreen_texture(&self) -> &Texture {
        &self.surface_resources.as_ref().unwrap().offscreen_texture
    }

    pub fn multisampled_texture(&self) -> &Texture {
        &self
            .surface_resources
            .as_ref()
            .unwrap()
            .multisampled_texture
    }

    pub fn universal_bind_group(&self) -> &BindGroup {
        &self
            .surface_resources
            .as_ref()
            .unwrap()
            .universal_bind_group
    }

    pub fn format(&self) -> TextureFormat {
        self.config.as_ref().unwrap().format
    }

    pub fn ready(&self) -> bool {
        self.surface_resources.is_some() && self.config.is_some()
    }

    pub fn handle_event(
        &mut self,
        event: &Event<()>,
        window: Arc<Window>,
        instance: &Instance,
        adapter: &Adapter,
        device: &Device,
        sampler: &Sampler,
        universal_bind_group_layout: &BindGroupLayout,
        srgb: bool,
    ) -> bool {
        match event {
            Event::NewEvents(StartCause::Init) | Event::Resumed => {
                // Window size is only actually valid after we enter the event loop.
                let window_size = window.inner_size();
                let width = window_size.width.max(1);
                let height = window_size.height.max(1);

                let surface = instance.create_surface(window).unwrap();

                // Get the default configuration,
                let mut config = surface
                    .get_default_config(adapter, width, height)
                    .expect("Surface isn't supported by the adapter.");

                config.usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC;

                //                 let surface_config = SurfaceConfiguration {
                //                     usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
                //                     format: swapchain_format,
                //                     width,
                //                     height,
                //                     present_mode: PresentMode::Fifo,
                //                     alpha_mode: swapchain_capabilities.alpha_modes[0],
                //                     view_formats: vec![],
                //                     desired_maximum_frame_latency: 2,
                //                 };
                if srgb {
                    // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
                    let view_format = config.format.add_srgb_suffix();
                    config.view_formats.push(view_format);
                } else {
                    // All platforms support non-sRGB swapchains, so we can just use the format directly.
                    let format = config.format.remove_srgb_suffix();
                    config.format = format;
                    config.view_formats.push(format);
                };

                self.surface_resources = Some(SurfaceResources::new(
                    device,
                    sampler,
                    surface,
                    &config,
                    universal_bind_group_layout,
                ));
                self.config = Some(config);
                true
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let config = self.config.as_mut().unwrap();
                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);

                let surface = self.surface_resources.take().unwrap().surface;

                self.surface_resources = Some(SurfaceResources::new(
                    device,
                    sampler,
                    surface,
                    config,
                    universal_bind_group_layout,
                ));

                true
            }
            _ => false,
        }
    }
}

fn create_texture(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
    samples: u32,
    label: &'static str,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT,
        label: Some(label),
        view_formats: &[],
    })
}

fn create_bind_group(
    device: &Device,
    offscreen_texture: &Texture,
    sampler: &Sampler,
    universal_bind_group_layout: &BindGroupLayout,
) -> BindGroup {
    let offscreen_texture_view = offscreen_texture.create_view(&TextureViewDescriptor::default());

    device.create_bind_group(&BindGroupDescriptor {
        label: Some("Universal bind group"),
        layout: universal_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&offscreen_texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}
