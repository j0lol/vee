use crate::camera::{Camera, CameraUniform};
use crate::{DARK_REBECCA_PURPLE, FACES};
use glam::{UVec2, Vec3, uvec2};
use nest_struct::nest_struct;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::{f32::consts::FRAC_PI_2, fs::File, sync::Arc};
use vfl::impl_wgpu::ProgramState;
use vfl::impl_wgpu::draw::CharModel;
use vfl::impl_wgpu::texture::TextureBundle;
use vfl::parse::{BinRead, NxCharInfo};
use vfl::res::shape::ResourceShape;
use vfl::res::tex::ResourceTexture;
use wgpu::{Backends, Features, util::DeviceExt};
use winit::window::Window;

/// Yeah, yeah.
///
/// - _What is this?_
///
///   It's a `CharModel` wrapped in a `Mutex` and a `OnceLock`.
///   A `Mutex` allows multiple references to a value that can be mutated.
///   This is normally not allowed in Rust (see `&mut` rules.)
///   A `OnceLock` is basically a wrapper for a maybe uninitialised value.
///   We use it to initialize the model "later" (once we can render.)
///   It's in a `static` so we can use it anywhere.
///   It's like a constant pointer to a location in memory.
///
/// - _Why isn't this in `State`?_
///
///   It needs to be global (so we can reference it in rendering),
///   but it needs to take a `&mut` reference to `State` to be initialized.
///   The compiler basically complained at me too much.
static CHAR_MODEL: OnceLock<Mutex<CharModel>> = OnceLock::new();

// Static lifetime 'cos it's a static.
pub fn char_model() -> MutexGuard<'static, CharModel> {
    CHAR_MODEL.get().unwrap().lock().unwrap()
}

/// `wgpu` requires a LOT of state for rendering. It's kind of annoying.
/// We put our own stuff in here too (camera, resource data, etc.)
///
/// Nested struct definitions requires a macro...
/// - See: <https://github.com/rust-lang/rfcs/pull/2584>
///
/// There are a few macros that implement this, I might swap it later
/// - See: {`nest_struct`, `structstruck`, `nestify`}
#[nest_struct]
pub struct State {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: UVec2,
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub char_info: NxCharInfo,
    pub char_remake: bool,
    pub egui: EguiState! {
        pub winit_state: egui_winit::State,
        pub wgpu_renderer: egui_wgpu::Renderer,
        frame_started: bool,
        scale_factor: f32,
    },
    pub camera: CameraState! {
        pub this: Camera,
        pub buffer: wgpu::Buffer,
        pub bind_group: wgpu::BindGroup,
        pub uniform: CameraUniform,
        pub bind_group_layout: wgpu::BindGroupLayout,
        pub depth_texture: TextureBundle,
        pub rotation_count: usize,
    },
    pub resources: ResourceData! {
        pub texture_header: ResourceTexture,
        pub shape_header: ResourceShape,
        pub texture_data: Rc<Vec<u8>>,
        pub shape_data: Rc<Vec<u8>>,
    },
}

impl State {
    /// All of our GPU setup code. Async because `winit` is annoying.
    pub async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY | Backends::SECONDARY,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: Features::SHADER_F16,
                ..Default::default()
            })
            .await
            .unwrap();

        let size = window.inner_size();
        let size = uvec2(size.width, size.height);

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let mut char_info = File::open(format!(
            "{}/resources_here/{}",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            ),
            FACES[0]
        ))
        .unwrap();
        let char_info = NxCharInfo::read(&mut char_info).unwrap();

        let camera = {
            let depth_texture =
                TextureBundle::create_depth_texture(&device, &size, "depth_texture");

            let camera = Camera {
                eye: (0.0, 25.0, 100.0).into(),
                target: (0.0, 25.0, 0.0).into(),
                up: Vec3::Y,
                aspect: size.x as f32 / size.y as f32,
                fov_y_radians: FRAC_PI_2,
                znear: 0.1,
                zfar: 10000.0,
            };

            let mut uniform = CameraUniform::new();
            uniform.update_view_proj(&camera);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group_layout =
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

            let rotation_count = 0;
            CameraState {
                this: camera,
                buffer,
                uniform,
                depth_texture,
                bind_group,
                bind_group_layout,
                rotation_count,
            }
        };

        let shape_file_path = format!(
            "{}/resources_here/ShapeMid.dat",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            ),
        );
        let tex_file_path = format!(
            "{}/resources_here/NXTextureMidSRGB.dat",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            ),
        );
        let shape_header = ResourceShape::read(&mut File::open(&shape_file_path).unwrap()).unwrap();
        let texture_header =
            ResourceTexture::read(&mut File::open(&tex_file_path).unwrap()).unwrap();
        let shape_data = Rc::new(std::fs::read(&shape_file_path).unwrap());
        let texture_data = Rc::new(std::fs::read(&tex_file_path).unwrap());

        let egui = {
            let egui_ctx = egui::Context::default();
            let winit_state = egui_winit::State::new(
                egui_ctx,
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                Some(2 * 1024),
            );
            let wgpu_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

            EguiState {
                winit_state,
                wgpu_renderer,
                frame_started: false,
                scale_factor: 1.0,
            }
        };

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
            camera,
            resources,
            egui,
            char_remake: true,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn render(&mut self) {
        let mut encoder = self.device.create_command_encoder(&Default::default());

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
        {
            // Create the renderpass which will clear the screen.
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    view: &self.camera.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        let char_info = self.char_info.clone();

        // Instantiate a CharModel if we need it.
        if CHAR_MODEL.get().is_none() {
            let new_model = CharModel::new(self, &char_info, &mut encoder);
            CHAR_MODEL.set(Mutex::new(new_model)).unwrap();
        }
        if self.char_remake {
            self.char_remake = false;

            let new_model = CharModel::new(self, &char_info, &mut encoder);

            let mut state = CHAR_MODEL.get().unwrap().lock().unwrap();
            *state = new_model;
        }

        // Actually render a CharModel.
        let mut model = char_model();
        model.render(self, &texture_view, &mut encoder);

        // Egui
        let input = self.egui.winit_state.take_egui_input(&self.window);
        let full_output = self.egui.winit_state.egui_ctx().run(input, |ctx| {
            egui::Window::new("Editor")
                .resizable(true)
                .vscroll(true)
                .default_open(true)
                .show(ctx, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.char_info.eye_type, 0..=61).text("Eye type"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.eyebrow_type, 0..=23)
                            .text("Eyebrow type"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.mouth_type, 0..=36)
                            .text("Mouth Type"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.nose_scale, 0..=255)
                            .text("Nose Scale"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.eyebrow_scale, 0..=255)
                            .text("Eyebrow Scale"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.eye_scale, 0..=255).text("Eye Scale"),
                    );

                    ui.add(
                        egui::Slider::new(&mut self.char_info.mouth_scale, 0..=255)
                            .text("Mouth Scale"),
                    );
                });
        });

        let pixels_per_point = self.window.scale_factor() as f32 * self.egui.scale_factor;

        self.egui
            .winit_state
            .egui_ctx()
            .set_pixels_per_point(pixels_per_point);

        self.egui
            .winit_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self.egui.winit_state.egui_ctx().tessellate(
            full_output.shapes,
            self.egui.winit_state.egui_ctx().pixels_per_point(),
        );

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui
                .wgpu_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let screen_descriptor = &egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.x, self.size.y],
            pixels_per_point,
        };

        self.egui.wgpu_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.egui.wgpu_renderer.render(
            &mut render_pass.forget_lifetime(),
            &tris,
            &screen_descriptor,
        );
        for x in &full_output.textures_delta.free {
            self.egui.wgpu_renderer.free_texture(x)
        }

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we're going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.x,
            height: self.size.y,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = uvec2(new_size.width, new_size.height);

        // reconfigure the surface
        self.configure_surface();

        self.camera.depth_texture =
            TextureBundle::create_depth_texture(&self.device, &self.size, "depth_texture");
    }

    pub fn update(&mut self) {
        let forward = self.camera.this.target - self.camera.this.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        let right = forward_norm.cross(self.camera.this.up);

        const CAMERA_ROTATE_SPEED: f32 = 0.0;
        self.camera.this.eye = self.camera.this.target
            - (forward + right * CAMERA_ROTATE_SPEED).normalize() * forward_mag;

        let forward_new = self.camera.this.target - self.camera.this.eye;
        let forward_new_norm = forward_new.normalize();

        const ROTATION_POINT: f32 = 0.0;
        if forward_norm.x < ROTATION_POINT && forward_new_norm.x >= ROTATION_POINT {
            self.camera.rotation_count += 1;

            let mut char_info = File::open(format!(
                "{}/resources_here/{}",
                std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                    std::env::current_dir()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                ),
                FACES[self.camera.rotation_count % FACES.len()]
            ))
            .unwrap();
            self.char_info = NxCharInfo::read(&mut char_info).unwrap();

            self.char_remake = true;
        }

        // dbg
        self.char_remake = true;

        self.camera.uniform.update_view_proj(&self.camera.this);
        self.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform]),
        );
    }
}

impl ProgramState for State {
    fn device(&self) -> wgpu::Device {
        self.device.clone()
    }

    fn queue(&self) -> wgpu::Queue {
        self.queue.clone()
    }

    fn camera_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.camera.bind_group_layout
    }

    fn camera_bg(&self) -> &wgpu::BindGroup {
        &self.camera.bind_group
    }

    fn surface_fmt(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    fn depth_texture(&self) -> &TextureBundle {
        &self.camera.depth_texture
    }

    fn texture_header(&self) -> ResourceTexture {
        self.resources.texture_header
    }

    fn shape_header(&self) -> ResourceShape {
        self.resources.shape_header
    }

    fn texture_data(&self) -> Rc<Vec<u8>> {
        self.resources.texture_data.clone()
    }

    fn shape_data(&self) -> Rc<Vec<u8>> {
        self.resources.shape_data.clone()
    }
}
