use crate::*;

//https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#first-some-housekeeping-state
pub(crate) struct CurrentWindowState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    configuration: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Arc<Window>,
}

impl CurrentWindowState {
    pub(crate) async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = wgpu::Instance::create_surface(&instance, window.clone())
            .expect("Creation of surface failed!");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Requesting GPU preferences failed!");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(), //TODO: might want to change this when building for web
                    label: None,
                },
                None,
            )
            .await
            .expect("Requesting device details failed!");
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_fmt = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(
                surface_capabilities
                    .formats
                    .get(0)
                    .expect("surface supported no formats")
                    .clone(),
            );
        let configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_fmt,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities
                .alpha_modes
                .get(0)
                .expect("surface supported no alpha modes")
                .clone(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // let new_window = CloneableWindow::from_window(window).clone().to_window();
        return Self {
            surface,
            device,
            queue,
            configuration,
            size,
            window,
        };
    }
    pub(crate) fn window(&self) -> &Window {
        &self.window.as_ref()
    }
    pub(crate) fn resize(&mut self, new: PhysicalSize<u32>) {
        if new.width > 0 && new.height > 0 {
            self.size = new;
            self.configuration.width = new.width;
            self.configuration.height = new.height;
            self.surface.configure(&self.device, &self.configuration);
        }
    }
    pub(crate) fn get_size(&self) -> PhysicalSize<u32> {
        self.size
    }
    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }
    pub(crate) fn update(&mut self) {
        todo!()
    }
    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut cmdenconder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = cmdenconder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
        self.queue.submit(std::iter::once(cmdenconder.finish()));
        output.present();
        Ok(())
    }
}
