use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::{f32::consts::FRAC_PI_2, fs::File, io::BufReader, sync::Arc};

use camera::{Camera, CameraUniform};
use char::draw_char;
use char_model::CharModel;
use glam::{UVec2, Vec3, Vec4, uvec2, vec4};
use vfl::res::shape::nx::ResourceShape;
use vfl::res::tex::nx::ResourceTexture;
use vfl::{
    charinfo::nx::{BinRead, NxCharInfo},
    draw::{render_3d::ProgramState, wgpu_render::texture},
    res::{shape::nx::SHAPE_MID_DAT, tex::nx::TEXTURE_MID_SRGB_DAT},
};
use wgpu::{Backends, util::DeviceExt};
use wgpu::{CommandEncoder, TextureView};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

static CHAR_MODEL: OnceLock<Mutex<CharModel>> = OnceLock::new();

pub mod char;
pub mod char_model;
pub mod render;

const OVERLAY_REBECCA_PURPLE: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.12,
    b: 0.1,
    a: 0.3,
};
const DARK_REBECCA_PURPLE: wgpu::Color = wgpu::Color {
    r: 0.2,
    g: 0.1,
    b: 0.3,
    a: 1.0,
};

pub const fn wgpu_color_to_vec4(color: wgpu::Color) -> Vec4 {
    vec4(
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    )
}

const FACES: [&str; 3] = [
    // "testguy.charinfo",
    // "j0.charinfo",
    "charline.charinfo",
    "Jasmine.charinfo",
    "soyun.charinfo",
];

pub struct ResourceData {
    texture_header: ResourceTexture,
    shape_header: ResourceShape,
    texture_data: Vec<u8>,
    shape_data: Vec<u8>,
}

struct WgpuSubState<'a> {
    inner: (
        wgpu::Device,
        wgpu::Queue,
        &'a wgpu::BindGroupLayout,
        &'a wgpu::BindGroup,
        wgpu::TextureFormat,
        &'a texture::Texture,
    ),
}

pub struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: UVec2,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    char_info: NxCharInfo,
    char_model: Option<CharModel>,
    char_remake: bool,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: texture::Texture,
    camera_rotations: usize,
    resources: ResourceData,
}

impl ProgramState for State {
    fn device(&self) -> wgpu::Device {
        self.device.clone()
    }

    fn queue(&self) -> wgpu::Queue {
        self.queue.clone()
    }

    fn camera_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    fn camera_bg(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    fn surface_fmt(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    fn depth_texture(&self) -> &vfl::draw::wgpu_render::texture::Texture {
        &self.depth_texture
    }
}

impl<'a> ProgramState for WgpuSubState<'a> {
    fn device(&self) -> wgpu::Device {
        self.inner.0.clone()
    }

    fn queue(&self) -> wgpu::Queue {
        self.inner.1.clone()
    }

    fn camera_bgl(&self) -> &wgpu::BindGroupLayout {
        self.inner.2
    }

    fn camera_bg(&self) -> &wgpu::BindGroup {
        self.inner.3
    }

    fn surface_fmt(&self) -> wgpu::TextureFormat {
        self.inner.4.clone()
    }

    fn depth_texture(&self) -> &texture::Texture {
        self.inner.5
    }
}

impl State {
    fn wgpu_sub_state(&self) -> WgpuSubState {
        WgpuSubState {
            inner: (
                self.device(),
                self.queue(),
                self.camera_bgl(),
                self.camera_bg(),
                self.surface_fmt(),
                self.depth_texture(),
            ),
        }
    }

    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY | Backends::SECONDARY,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();
        let size = uvec2(size.width, size.height);

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let depth_texture = texture::Texture::create_depth_texture(&device, &size, "depth_texture");

        let mut char_info =
            File::open(format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), FACES[0])).unwrap();
        let char_info = NxCharInfo::read(&mut char_info).unwrap();

        let camera = Camera {
            eye: (0.0, 25.0, 100.0).into(),
            target: (0.0, 25.0, 0.0).into(),
            up: Vec3::Y,
            aspect: size.x as f32 / size.y as f32,
            fov_y_radians: FRAC_PI_2,
            znear: 0.1,
            zfar: 10000.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_rotations = 0;

        let shape_header = ResourceShape::read(&mut File::open(SHAPE_MID_DAT).unwrap()).unwrap();
        let texture_header =
            ResourceTexture::read(&mut File::open(TEXTURE_MID_SRGB_DAT).unwrap()).unwrap();
        let shape_data = (std::fs::read(SHAPE_MID_DAT).unwrap());
        let texture_data = (std::fs::read(TEXTURE_MID_SRGB_DAT).unwrap());

        let resources = ResourceData {
            shape_header,
            texture_header,
            shape_data,
            texture_data,
        };

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            char_info,
            char_model: None,
            camera,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            camera_bind_group_layout,
            depth_texture,
            camera_rotations,
            resources,
            char_remake: true,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.x,
            height: self.size.y,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = uvec2(new_size.width, new_size.height);

        // reconfigure the surface
        self.configure_surface();

        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.size, "depth_texture");
    }

    fn render(&mut self) {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Clear the screen, so we can layer a BUNCH of render passes...

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(DARK_REBECCA_PURPLE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            // Clear the depth buffer, too.
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        drop(renderpass);

        // Actually render a CharModel.

        if CHAR_MODEL.get().is_none() {
            let new_model = CharModel::new(self, &mut encoder);
            CHAR_MODEL.set(Mutex::new(new_model)).unwrap();
        }
        if self.char_remake {
            self.char_remake = false;

            let new_model = CharModel::new(self, &mut encoder);

            let mut state = CHAR_MODEL.get().unwrap().lock().unwrap();
            *state = new_model;
        }

        CHAR_MODEL
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .render(self, &texture_view, &mut encoder);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    fn update(&mut self) {
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        let right = forward_norm.cross(self.camera.up);

        const CAMERA_ROTATE_SPEED: f32 = 0.5;
        self.camera.eye =
            self.camera.target - (forward + right * CAMERA_ROTATE_SPEED).normalize() * forward_mag;

        let forward_new = self.camera.target - self.camera.eye;
        let forward_new_norm = forward_new.normalize();

        const ROTATION_POINT: f32 = 0.0;
        if forward_norm.x < ROTATION_POINT && forward_new_norm.x >= ROTATION_POINT {
            self.camera_rotations += 1;

            let mut char_info = File::open(format!(
                "{}/../{}",
                env!("CARGO_MANIFEST_DIR"),
                FACES[self.camera_rotations % FACES.len()]
            ))
            .unwrap();
            self.char_info = NxCharInfo::read(&mut char_info).unwrap();

            self.char_remake = true;
            // if CHAR_MODEL.get().is_none() {
            //     let new_model = CharModel::new(self, &mut encoder);
            //     CHAR_MODEL.set(Mutex::new(new_model)).unwrap();
            // }
        }

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Vee(FL)-Testing"))
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    // env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

mod camera {
    use glam::{Mat4, Vec3};

    pub struct Camera {
        pub eye: Vec3,
        pub target: Vec3,
        pub up: Vec3,
        pub aspect: f32,
        pub fov_y_radians: f32,
        pub znear: f32,
        pub zfar: f32,
    }

    impl Camera {
        pub fn build_view_projection_matrix(&self) -> Mat4 {
            let view = Mat4::look_at_rh(self.eye, self.target, self.up);
            let proj = Mat4::perspective_rh(self.fov_y_radians, self.aspect, self.znear, self.zfar);
            proj * view
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct CameraUniform {
        view_proj: [[f32; 4]; 4],
    }

    impl CameraUniform {
        pub fn new() -> Self {
            Self {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }
        }

        pub fn update_view_proj(&mut self, camera: &Camera) {
            self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
        }
    }
}

// mod texture {
//     use std::error::Error;

//     use image::GenericImageView;
//     use wgpu::TextureFormat;
//     use winit::dpi::PhysicalSize;

//     pub struct Texture {
//         #[allow(unused)]
//         pub texture: wgpu::Texture,
//         pub view: wgpu::TextureView,
//         pub sampler: wgpu::Sampler,
//     }

//     #[allow(unused)]
//     impl Texture {
//         pub fn from_bytes(
//             device: &wgpu::Device,
//             queue: &wgpu::Queue,
//             bytes: &[u8],
//             label: &str,
//         ) -> Result<Self, Box<dyn Error>> {
//             let img = image::load_from_memory(bytes)?;
//             Self::from_image(device, queue, &img, Some(label))
//         }

//         pub fn from_image(
//             device: &wgpu::Device,
//             queue: &wgpu::Queue,
//             img: &image::DynamicImage,
//             label: Option<&str>,
//         ) -> Result<Self, Box<dyn Error>> {
//             let rgba = img.to_rgba8();
//             let dimensions = img.dimensions();

//             let size = wgpu::Extent3d {
//                 width: dimensions.0,
//                 height: dimensions.1,
//                 depth_or_array_layers: 1,
//             };
//             let texture = device.create_texture(&wgpu::TextureDescriptor {
//                 label,
//                 size,
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 dimension: wgpu::TextureDimension::D2,
//                 format: wgpu::TextureFormat::Rgba8UnormSrgb,
//                 usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//                 view_formats: &[],
//             });

//             queue.write_texture(
//                 wgpu::TexelCopyTextureInfo {
//                     aspect: wgpu::TextureAspect::All,
//                     texture: &texture,
//                     mip_level: 0,
//                     origin: wgpu::Origin3d::ZERO,
//                 },
//                 &rgba,
//                 wgpu::TexelCopyBufferLayout {
//                     offset: 0,
//                     bytes_per_row: Some(4 * dimensions.0),
//                     rows_per_image: Some(dimensions.1),
//                 },
//                 size,
//             );

//             let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//             let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//                 address_mode_u: wgpu::AddressMode::ClampToEdge,
//                 address_mode_v: wgpu::AddressMode::ClampToEdge,
//                 address_mode_w: wgpu::AddressMode::ClampToEdge,
//                 mag_filter: wgpu::FilterMode::Linear,
//                 min_filter: wgpu::FilterMode::Nearest,
//                 mipmap_filter: wgpu::FilterMode::Nearest,
//                 ..Default::default()
//             });

//             Ok(Self {
//                 texture,
//                 view,
//                 sampler,
//             })
//         }

//         pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

//         pub fn create_depth_texture(
//             device: &wgpu::Device,
//             size: &PhysicalSize<u32>,
//             label: &str,
//         ) -> Self {
//             let size = wgpu::Extent3d {
//                 // 2.
//                 width: size.width.max(1),
//                 height: size.height.max(1),
//                 depth_or_array_layers: 1,
//             };
//             let desc = wgpu::TextureDescriptor {
//                 label: Some(label),
//                 size,
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 dimension: wgpu::TextureDimension::D2,
//                 format: Self::DEPTH_FORMAT,
//                 usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
//                        | wgpu::TextureUsages::TEXTURE_BINDING,
//                 view_formats: &[],
//             };
//             let texture = device.create_texture(&desc);

//             let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//             let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//                 // 4.
//                 address_mode_u: wgpu::AddressMode::ClampToEdge,
//                 address_mode_v: wgpu::AddressMode::ClampToEdge,
//                 address_mode_w: wgpu::AddressMode::ClampToEdge,
//                 mag_filter: wgpu::FilterMode::Linear,
//                 min_filter: wgpu::FilterMode::Linear,
//                 mipmap_filter: wgpu::FilterMode::Nearest,
//                 compare: Some(wgpu::CompareFunction::LessEqual), // 5.
//                 lod_min_clamp: 0.0,
//                 lod_max_clamp: 100.0,
//                 ..Default::default()
//             });

//             Self {
//                 texture,
//                 view,
//                 sampler,
//             }
//         }

//         pub fn create_texture(
//             device: &wgpu::Device,
//             size: &PhysicalSize<u32>,
//             label: &str,
//         ) -> Self {
//             let size = wgpu::Extent3d {
//                 // 2.
//                 width: size.width.max(1),
//                 height: size.height.max(1),
//                 depth_or_array_layers: 1,
//             };
//             let desc = wgpu::TextureDescriptor {
//                 label: Some(label),
//                 size,
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 dimension: wgpu::TextureDimension::D2,
//                 format: TextureFormat::Bgra8UnormSrgb,
//                 usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
//                        | wgpu::TextureUsages::TEXTURE_BINDING,
//                 view_formats: &[],
//             };
//             let texture = device.create_texture(&desc);

//             let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
//             let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//                 // 4.
//                 address_mode_u: wgpu::AddressMode::MirrorRepeat,
//                 address_mode_v: wgpu::AddressMode::MirrorRepeat,
//                 address_mode_w: wgpu::AddressMode::MirrorRepeat,
//                 mag_filter: wgpu::FilterMode::Linear,
//                 min_filter: wgpu::FilterMode::Linear,
//                 mipmap_filter: wgpu::FilterMode::Nearest,
//                 lod_min_clamp: 0.0,
//                 lod_max_clamp: 100.0,
//                 ..Default::default()
//             });

//             Self {
//                 texture,
//                 view,
//                 sampler,
//             }
//         }
//     }
// }
