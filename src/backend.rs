use crate::*;

//https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#first-some-housekeeping-state
struct CurrentWindowState{
    surface: wgpu::Surface <'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    configuration: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Arc<Window>,
}

// struct CloneableWindow{
//     internal: Window
// }

// impl Clone for CloneableWindow{
//     fn clone(&self) -> Self {
//         CloneableWindow { internal: self.internal }
//     }
// }

// impl CloneableWindow{
//     fn from_window(window: &Window) -> Self{
//         Self { internal: windo }
//     }

//     fn to_window(&self) -> Window{
//         self.internal
//     }
// }

impl CurrentWindowState{
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = wgpu::Instance
            ::create_surface(&instance, window.clone())
            .expect("Creation of surface failed!");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase{
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
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_fmt,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        // let new_window = CloneableWindow::from_window(window).clone().to_window();
        return Self{
            surface: surface,
            device: device,
            queue: queue,
            configuration: config,
            size: size,
            window: window
        }
    }
    fn resize(&mut self, new: PhysicalSize<u32>) {
        todo!()
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }
    fn update(&mut self) {
        todo!()
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }
}
