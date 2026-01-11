use glam::{Vec3, uvec2};
use std::error::Error;
use std::fs::File;
use std::rc::Rc;
use vfl::impl_wgpu::ProgramState;
use vfl::impl_wgpu::draw::CharModel;
use vfl::impl_wgpu::texture::TextureBundle;
use vfl::parse::{BinRead, NxCharInfo};
use vfl::res::shape::ResourceShape;
use vfl::res::tex::ResourceTexture;
use wgpu::{Backends, Features, util::DeviceExt};

const BODY_SCALE: f32 = 10.0;

struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fov_y_radians: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = glam::Mat4::look_at_rh(camera.eye, camera.target, camera.up);
        let proj = glam::Mat4::perspective_rh(
            camera.fov_y_radians,
            camera.aspect,
            camera.znear,
            camera.zfar,
        );
        self.view_proj = (proj * view).to_cols_array_2d();
    }
}

struct RenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    surface_format: wgpu::TextureFormat,
    depth_texture: TextureBundle,
    texture_header: ResourceTexture,
    shape_header: ResourceShape,
    texture_data: Rc<Vec<u8>>,
    shape_data: Rc<Vec<u8>>,
}

impl ProgramState for RenderState {
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

    fn depth_texture(&self) -> &TextureBundle {
        &self.depth_texture
    }

    fn texture_header(&self) -> ResourceTexture {
        self.texture_header
    }

    fn shape_header(&self) -> ResourceShape {
        self.shape_header
    }

    fn texture_data(&self) -> Rc<Vec<u8>> {
        self.texture_data.clone()
    }

    fn shape_data(&self) -> Rc<Vec<u8>> {
        self.shape_data.clone()
    }
}

/// Renders a Character to a texture and returns the image
pub async fn render_to_texture(
    char_info: &[u8],
    resources_path: &str,
    width: u32,
    height: u32,
) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::PRIMARY | Backends::SECONDARY,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .or(Err("Failed to find an appropriate adapter"))?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_features: Features::SHADER_F16,
            ..Default::default()
        })
        .await?;

    let size = uvec2(width, height);
    let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    // Load resources
    let shape_file_path = format!("{}/ShapeMid.dat", resources_path);
    let tex_file_path = format!("{}/NXTextureMidSRGB.dat", resources_path);

    let shape_header = ResourceShape::read(&mut File::open(&shape_file_path)?)?;
    let texture_header = ResourceTexture::read(&mut File::open(&tex_file_path)?)?;
    let shape_data = Rc::new(std::fs::read(&shape_file_path)?);
    let texture_data = Rc::new(std::fs::read(&tex_file_path)?);

    let char_info = NxCharInfo::read(&mut std::io::Cursor::new(char_info))?;

    // Match FFL makeIcon camera configuration
    // getFaceCamera(): (0, 4.805, 57.553)
    let aspect = width as f32 / height as f32;
    let fovy = 15.0f32.to_radians(); // 15 degrees FOV

    let mut camera = Camera {
        eye: (0.0, 4.805 * BODY_SCALE, 57.553 * BODY_SCALE).into(),
        target: (0.0, 4.805 * BODY_SCALE, 0.0).into(),
        up: Vec3::Y,
        aspect,
        fov_y_radians: fovy,
        znear: 50.0,
        zfar: 1000.0,
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

    let depth_texture = TextureBundle::create_depth_texture(&device, &size, "depth_texture");

    let render_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("Render Texture"),
        // Request compatibility with the sRGB-format texture view we're going to create later.
        view_formats: &[surface_format],
    });

    let texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(surface_format),
        ..Default::default()
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    // Create CharModel and render in a scope to drop state before await.
    // Rendering across an await-point will cause a compile error.
    {
        let mut state = RenderState {
            device: device.clone(),
            queue: queue.clone(),
            camera_bind_group_layout,
            camera_bind_group,
            surface_format,
            depth_texture,
            texture_header,
            shape_header,
            texture_data,
            shape_data,
        };

        let mut char_model = CharModel::new(&mut state, &char_info, &mut encoder);

        // Adjust camera for body/head
        let head_pos = char_model.head_transform.transform_point3(Vec3::ZERO) * BODY_SCALE;
        camera.eye.y += head_pos.y;
        camera.target.y += head_pos.y;

        camera_uniform.update_view_proj(&camera);
        state
            .queue
            .write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.1,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.depth_texture.view,
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

        char_model.render(&mut state, &texture_view, &mut encoder);
    } // drop RenderState

    // Copy texture to a buffer so we can read it to an image
    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: (u32_size * width * height) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: Some("Output Buffer"),
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(u32_size * width),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = tokio::sync::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    device.poll(wgpu::PollType::Wait).unwrap();
    rx.await??;

    let data = buffer_slice.get_mapped_range();

    // Convert BGRA to RGBA by swapping red and blue channels
    // Unfortunate workaround :-(
    let mut rgba_data = Vec::with_capacity(data.len());
    for chunk in data.chunks_exact(4) {
        rgba_data.push(chunk[2]); // R (was B)
        rgba_data.push(chunk[1]); // G
        rgba_data.push(chunk[0]); // B (was R)
        rgba_data.push(chunk[3]); // A
    }

    let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, rgba_data)
        .ok_or("Failed to create image buffer")?;

    Ok(buffer)
}
