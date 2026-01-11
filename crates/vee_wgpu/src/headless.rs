//! Constructs for rendering without a surface (headlessly) i.e. on a server.

use crate::ProgramState;
use crate::texture::TextureBundle;
use camera::{Camera, CameraUniform};
use glam::{UVec2, Vec3, uvec2};
use image::{DynamicImage, RgbaImage};
use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::rc::Rc;
use vee_parse::BinRead;
use vee_resources::shape::ResourceShape;
use vee_resources::tex::ResourceTexture;
use wgpu::{CommandEncoder, DeviceDescriptor, util::DeviceExt};

pub(crate) struct ResourceData {
    pub(crate) texture_header: ResourceTexture,
    pub(crate) shape_header: ResourceShape,
    pub(crate) texture_data: Rc<Vec<u8>>,
    pub(crate) shape_data: Rc<Vec<u8>>,
}

/// Contains all the state required for rendering headlessly.
pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    camera_bgl: wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,
    surface_fmt: wgpu::TextureFormat,
    depth_texture: TextureBundle,
    resource_data: ResourceData,
}

impl ProgramState for HeadlessRenderer {
    fn device(&self) -> wgpu::Device {
        self.device.clone()
    }

    fn queue(&self) -> wgpu::Queue {
        self.queue.clone()
    }

    fn camera_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bgl
    }

    fn camera_bg(&self) -> &wgpu::BindGroup {
        &self.camera_bg
    }

    fn surface_fmt(&self) -> wgpu::TextureFormat {
        self.surface_fmt
    }

    fn depth_texture(&self) -> &TextureBundle {
        &self.depth_texture
    }

    fn texture_header(&self) -> ResourceTexture {
        self.resource_data.texture_header
    }

    fn shape_header(&self) -> ResourceShape {
        self.resource_data.shape_header
    }

    fn texture_data(&self) -> Rc<Vec<u8>> {
        self.resource_data.texture_data.clone()
    }

    fn shape_data(&self) -> Rc<Vec<u8>> {
        self.resource_data.shape_data.clone()
    }
}

impl HeadlessRenderer {
    /// Instantiate a `HeadlessRenderer`.
    /// Requires the shape file and texture file paths.
    pub fn new(shape_file: &str, texture_file: &str) -> HeadlessRenderer {
        pollster::block_on(HeadlessRenderer::async_new(shape_file, texture_file))
    }
    async fn async_new(shape_file: &str, texture_file: &str) -> HeadlessRenderer {
        const SIZE: UVec2 = uvec2(512, 512);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap();

        // let surface = instance.create_surface(window.clone()).unwrap();
        // let cap = surface.get_capabilities(&adapter);
        // let surface_fmt = cap.formats[0];
        let surface_fmt = wgpu::TextureFormat::Bgra8Unorm;

        let depth_texture = TextureBundle::create_depth_texture(&device, &SIZE, "depth_texture");

        let camera = Camera {
            eye: (0.0, 25.0, 100.0).into(),
            target: (0.0, 25.0, 0.0).into(),
            up: Vec3::Y,
            aspect: SIZE.x as f32 / SIZE.y as f32,
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

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shape_header = ResourceShape::read(&mut File::open(shape_file).unwrap()).unwrap();
        let texture_header = ResourceTexture::read(&mut File::open(texture_file).unwrap()).unwrap();
        let shape_data = Rc::new(std::fs::read(shape_file).unwrap());
        let texture_data = Rc::new(std::fs::read(texture_file).unwrap());

        let resource_data = ResourceData {
            shape_header,
            texture_header,
            shape_data,
            texture_data,
        };

        HeadlessRenderer {
            device,
            queue,
            camera_bgl,
            camera_bg,
            surface_fmt,
            depth_texture,
            resource_data,
        }
    }

    /// After rendering, this function will consume the encoder and output commands to the GPU.
    /// Requires a `TextureBundle` to render to.
    #[allow(unused)]
    pub fn output_texture(
        &mut self,
        texture: &TextureBundle,
        mut encoder: CommandEncoder,
    ) -> DynamicImage {
        pollster::block_on(async {
            let u32_size = size_of::<u32>() as u32;
            let output_buffer_size = wgpu::BufferAddress::from(
                u32_size * texture.texture.size().width * texture.texture.size().height,
            );
            let output_buffer_desc = wgpu::BufferDescriptor {
                size: output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST
                        // this tells `wpgu` that we want to read this buffer from the CPU
                        | wgpu::BufferUsages::MAP_READ,
                label: None,
                mapped_at_creation: false,
            };
            let output_buffer = self.device.create_buffer(&output_buffer_desc);

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &output_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(u32_size * texture.texture.size().width),
                        rows_per_image: Some(texture.texture.size().height),
                    },
                },
                texture.texture.size(),
            );

            self.queue.submit(Some(encoder.finish()));

            let image;
            // We need to scope the mapping variables so that we can
            // unmap the buffer
            {
                let buffer_slice = output_buffer.slice(..);

                // NOTE: We have to create the mapping THEN device.poll() before await
                // the future. Otherwise, the application will freeze.
                let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    tx.send(result).unwrap();
                });
                self.device.poll(wgpu::PollType::Wait).unwrap();
                rx.receive().await.unwrap().unwrap();

                let data = &buffer_slice.get_mapped_range()[..];

                use image::{ImageBuffer, Rgba};
                let buffer: RgbaImage = ImageBuffer::<Rgba<u8>, _>::from_raw(
                    texture.texture.size().width,
                    texture.texture.size().height,
                    data.to_owned(),
                )
                .unwrap();
                image = Some(DynamicImage::ImageRgba8(buffer));
            }
            output_buffer.unmap();

            image.unwrap()
        })
    }
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
