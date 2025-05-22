use super::TEX_SCALE_X;
use super::TEX_SCALE_Y;
use super::mask::{FacePart, ImageOrigin, MaskFaceParts};
use super::render_2d::Rendered2dShape;
use super::render_3d::ProgramState;
use super::render_3d::Rendered3dShape;
use crate::{
    charinfo::nx::NxCharInfo,
    color::{
        cafe::{
            EYE_COLOR_B, EYE_COLOR_G, EYE_COLOR_R, GLASS_COLOR_R, HAIR_COLOR, MOUTH_COLOR_B,
            MOUTH_COLOR_G, MOUTH_COLOR_R,
        },
        nx::{ColorModulated, modulate},
    },
    res::shape::nx::ResourceShape,
    res::tex::nx::{ResourceTexture, ResourceTextureFormat, TextureElement},
};
use binrw::BinRead;
use camera::Camera;
use camera::CameraUniform;
use glam::Vec3;
use glam::{UVec2, uvec2, vec3};
use image::{DynamicImage, RgbaImage};
use nalgebra::Matrix4;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;
use std::{error::Error, fs::File, io::BufReader};
use wgpu::CommandEncoder;
use wgpu::{DeviceDescriptor, TexelCopyTextureInfo, util::DeviceExt};

pub const FACE_OUTPUT_SIZE: u16 = 512;
pub const SHADER: &str = include_str!("./shader.wgsl");
pub use bytemuck::cast_slice;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureTransformUniform {
    pub mvp_matrix: [[f32; 4]; 4],
    pub channel_replacements_r: [f32; 4],
    pub channel_replacements_g: [f32; 4],
    pub channel_replacements_b: [f32; 4],
    pub texture_type: u32,
    pub pad: [u32; 3],
}

const NON_REPLACEMENT: [f32; 4] = [f32::NAN, f32::NAN, f32::NAN, f32::NAN];
// #[derive(Debug)]
// #[deprecated]
// pub struct RenderShape {
//     pub vertices: Vec<Vertex>,
//     pub indices: Vec<u32>,
//     pub tex: DynamicImage,
//     pub mvp_matrix: Matrix4<f32>,
//     pub texture_type: ResourceTextureFormat,
//     pub channel_replacements: [[f32; 4]; 3],
// }

pub struct RenderContext {
    pub size: UVec2,
    pub shape: Vec<Rendered2dShape>,
}
impl RenderContext {
    pub fn from_shapes(shape: Vec<Rendered2dShape>) -> RenderContext {
        RenderContext {
            size: uvec2(FACE_OUTPUT_SIZE.into(), FACE_OUTPUT_SIZE.into()),
            shape,
        }
    }
}

impl RenderContext {
    #[allow(clippy::too_many_lines)]
    pub fn new(
        char: &NxCharInfo,
        res_texture: &ResourceTexture,
        res_shape: &ResourceShape,
        file_texture: &Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let mask = MaskFaceParts::init(char, 256.0);

        let mut make_shape =
            |part: FacePart, modulated: ColorModulated, tex_data: TextureElement| {
                let (vertices, indices, mtx) = quad(
                    part.x,
                    part.y,
                    part.width,
                    part.height,
                    part.angle_deg,
                    part.origin,
                    256.0,
                );

                let tex = tex_data.get_image(file_texture).unwrap().unwrap();

                Rendered2dShape {
                    vertices,
                    indices,
                    tex: image::DynamicImage::ImageRgba8(tex),
                    mvp_matrix: mtx,
                    modulation: modulate(modulated, char),
                    opaque: None,
                }
            };

        let left_eye = make_shape(
            mask.eye[0],
            ColorModulated::Eye,
            res_texture.eye[char.eye_type as usize],
        );
        let right_eye = make_shape(
            mask.eye[1],
            ColorModulated::Eye,
            res_texture.eye[char.eye_type as usize],
        );

        let left_brow = make_shape(
            mask.eyebrow[0],
            ColorModulated::Eyebrow,
            res_texture.eyebrow[char.eyebrow_type as usize],
        );
        let right_brow = make_shape(
            mask.eyebrow[1],
            ColorModulated::Eyebrow,
            res_texture.eyebrow[char.eyebrow_type as usize],
        );

        let mouth = make_shape(
            mask.mouth,
            ColorModulated::Mouth,
            res_texture.mouth[char.mouth_type as usize],
        );

        Ok(RenderContext {
            size: uvec2(FACE_OUTPUT_SIZE.into(), FACE_OUTPUT_SIZE.into()),
            shape: vec![left_eye, right_eye, left_brow, right_brow, mouth],
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn new_faceline(
        char: &NxCharInfo,
        res_texture: &ResourceTexture,
        res_shape: &ResourceShape,
        file_texture: &Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let mask = MaskFaceParts::init(char, 256.0);
        let makeup = char.faceline_make;

        let mut make_shape =
            |part: FacePart, modulated: ColorModulated, tex_data: TextureElement| {
                let (vertices, indices, mtx) = quad(
                    part.x,
                    part.y,
                    part.width,
                    part.height,
                    part.angle_deg,
                    part.origin,
                    256.0,
                );

                let tex = tex_data.get_image(file_texture).unwrap().unwrap();

                Rendered2dShape {
                    vertices,
                    indices,
                    tex: image::DynamicImage::ImageRgba8(tex),
                    mvp_matrix: mtx,
                    modulation: modulate(modulated, char),
                    opaque: None,
                }
            };

        let mouth = make_shape(
            mask.mouth,
            ColorModulated::Mouth,
            res_texture.mouth[char.mouth_type as usize],
        );

        Ok(RenderContext {
            size: uvec2(FACE_OUTPUT_SIZE.into(), FACE_OUTPUT_SIZE.into()),
            shape: vec![mouth],
        })
    }

    pub fn new_glasses(
        char: &NxCharInfo,
        res_texture: &ResourceTexture,
        res_shape: &ResourceShape,
        file_texture: &Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let nose_translate =
            res_shape.face_line_transform[char.faceline_type as usize].nose_translate;

        let glasses = dbg!(MaskFaceParts::init_glasses(char, 256.0, nose_translate));

        // let mask = FaceParts::init(&char, 256.0);
        // let part = mask[0];
        let tex_data = res_texture.glass[char.glass_type as usize];
        let mut glass_l = glasses[0];

        let (vertices, indices, mtx) = quad(
            glass_l.x,
            glass_l.y,
            glass_l.width,
            glass_l.height,
            glass_l.angle_deg,
            glass_l.origin,
            256.0,
        );

        let tex = tex_data.get_image(file_texture)?.unwrap();

        let glass_l_render_shape = Rendered2dShape {
            vertices,
            indices,
            tex: image::DynamicImage::ImageRgba8(tex.clone()),
            mvp_matrix: mtx,
            modulation: modulate(ColorModulated::Glass, &char),
            opaque: None,
        };

        let glass_r = glasses[1];

        let (vertices, indices, mtx) = quad(
            glass_r.x,
            glass_r.y,
            glass_r.width,
            glass_r.height,
            glass_r.angle_deg,
            glass_r.origin,
            256.0,
        );

        let glass_r_render_shape = Rendered2dShape {
            vertices,
            indices,
            tex: image::DynamicImage::ImageRgba8(tex),
            mvp_matrix: mtx,
            modulation: modulate(ColorModulated::Glass, &char),
            opaque: None,
        };

        Ok(RenderContext {
            size: uvec2(512, 256),
            shape: vec![glass_l_render_shape, glass_r_render_shape],
        })
    }
}

pub fn model_view_matrix(
    translation: mint::Vector3<f32>,
    scale: mint::Vector3<f32>,
    rot_z: f32,
) -> nalgebra::Matrix4<f32> {
    let mut scale = nalgebra::Vector3::<f32>::from(scale);
    let translation = nalgebra::Vector3::<f32>::from(translation);

    let mut mtx = nalgebra::Matrix4::identity();
    mtx.append_nonuniform_scaling_mut(&scale);
    mtx *= nalgebra::Rotation3::from_euler_angles(0.0, 0.0, rot_z.to_radians()).to_homogeneous();
    mtx.append_nonuniform_scaling_mut(&nalgebra::Vector3::new(TEX_SCALE_X, TEX_SCALE_Y, 1.0));
    mtx.append_translation_mut(&translation);

    mtx
}

fn v2(x: f32, y: f32) -> [f32; 3] {
    [x, y, 0.0]
}

// https://github.com/SMGCommunity/Petari/blob/6e9ae741a99bb32e6ffbb230a88c976f539dde70/src/RVLFaceLib/RFL_MakeTex.c#L817
pub fn quad(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rot_z: f32,
    origin: ImageOrigin,
    resolution: f32,
) -> (Vec<Vertex>, Vec<u32>, nalgebra::Matrix4<f32>) {
    //     Mtx rot;
    //     Mtx pos;
    //     f32 baseX;
    //     s16 s0;
    //     s16 s1;
    let base_x: f32;
    let s0: f32;
    let s1: f32;

    let mv_mtx = model_view_matrix(
        vec3(x, resolution - y, 0.0).into(),
        vec3(width, height, 1.0).into(),
        rot_z,
    );

    let p_mtx = Matrix4::new_orthographic(0.0, resolution, 0.0, resolution, -200.0, 200.0);
    let mut mvp_mtx = p_mtx * mv_mtx;

    *mvp_mtx.get_mut((1, 1)).unwrap() *= -1.0;

    match origin {
        ImageOrigin::Center => {
            base_x = -0.5;
            s0 = 1.0;
            s1 = 0.0;
        }
        ImageOrigin::Right => {
            base_x = -1.0;
            s0 = 1.0;
            s1 = 0.0;
        }
        ImageOrigin::Left => {
            base_x = 0.0;
            s0 = 0.0;
            s1 = 1.0;
        }
        ImageOrigin::Ignore => {
            base_x = 0.0;
            s0 = 0.0;
            s1 = 1.0;
        }
    }

    (
        vec![
            Vertex {
                position: v2(1.0 + base_x, -0.5),
                tex_coords: [s0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: v2(1.0 + base_x, 0.5),
                tex_coords: [s0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: v2(base_x, 0.5),
                tex_coords: [s1, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: v2(base_x, -0.5),
                tex_coords: [s1, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
        ],
        vec![0, 1, 2, 0, 2, 3],
        mvp_mtx,
    )
}

pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    camera_bgl: wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,
    surface_fmt: wgpu::TextureFormat,
    depth_texture: texture::Texture,
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

    fn depth_texture(&self) -> &texture::Texture {
        &self.depth_texture
    }
}

impl Default for HeadlessRenderer {
    fn default() -> HeadlessRenderer {
        HeadlessRenderer::new()
    }
}
impl HeadlessRenderer {
    pub fn new() -> HeadlessRenderer {
        pollster::block_on(HeadlessRenderer::async_new())
    }
    async fn async_new() -> HeadlessRenderer {
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

        let depth_texture = texture::Texture::create_depth_texture(&device, &SIZE, "depth_texture");

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

        HeadlessRenderer {
            device,
            queue,
            camera_bgl,
            camera_bg,
            surface_fmt,
            depth_texture,
        }
    }

    pub fn output_texture(
        &mut self,
        texture: &texture::Texture,
        mut encoder: CommandEncoder,
    ) -> DynamicImage {
        pollster::block_on(async {
            let u32_size = std::mem::size_of::<u32>() as u32;
            let output_buffer_size = wgpu::BufferAddress::from(
                u32_size * texture.texture.size().width * texture.texture.size().height,
            );
            let output_buffer_desc = wgpu::BufferDescriptor {
                size: output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST
                        // this tells wpgu that we want to read this buffer from the cpu
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

            let mut image = None;
            // We need to scope the mapping variables so that we can
            // unmap the buffer
            {
                let buffer_slice = output_buffer.slice(..);

                // NOTE: We have to create the mapping THEN device.poll() before await
                // the future. Otherwise the application will freeze.
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

#[deprecated]
#[allow(clippy::too_many_lines)]
pub async fn render_context_wgpu(render_context: RenderContext) -> DynamicImage {
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

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: render_context.size.x,
            height: render_context.size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());
    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer_size =
        wgpu::BufferAddress::from(u32_size * render_context.size.x * render_context.size.y);
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    for shape in render_context.shape {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&shape.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&shape.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let shape_texture_rgba = shape.tex.to_rgba8();
        let shape_texture_dimensions = shape_texture_rgba.dimensions();
        let shape_texture_size = wgpu::Extent3d {
            width: shape_texture_dimensions.0,
            height: shape_texture_dimensions.1,
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };
        let shape_diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: shape_texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB, so we need to reflect that here.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &shape_diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &shape_texture_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * shape_texture_dimensions.0),
                rows_per_image: Some(shape_texture_dimensions.1),
            },
            shape_texture_size,
        );

        let shape_diffuse_texture_view =
            shape_diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shape_diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shape_diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shape_diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shape_diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let mvp_matrix = shape.mvp_matrix.into();
        let mvp_uniform = TextureTransformUniform {
            mvp_matrix,
            channel_replacements_r: shape.modulation.channels[0],
            channel_replacements_g: shape.modulation.channels[1],
            channel_replacements_b: shape.modulation.channels[2],
            texture_type: (Into::<u8>::into(shape.modulation.mode)).into(),
            pad: Default::default(),
        };

        let mvp_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("MvpMatrix Buffer"),
            contents: bytemuck::cast_slice(&[mvp_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mvp_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("mvp_bind_group_layout"),
            });

        let mvp_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mvp_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_buffer.as_entire_binding(),
            }],
            label: Some("mvp_bind_group"),
        });

        let shader = wgpu::ShaderSource::Wgsl(SHADER.into());
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: shader,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &mvp_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_desc.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &shape_diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &mvp_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..shape.indices.len() as u32, 0, 0..1);
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(u32_size * render_context.size.x),
                rows_per_image: Some(render_context.size.x),
            },
        },
        texture_desc.size,
    );

    queue.submit(Some(encoder.finish()));

    let mut image = None;
    // We need to scope the mapping variables so that we can
    // unmap the buffer
    {
        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::PollType::Wait).unwrap();
        rx.receive().await.unwrap().unwrap();

        let data = &buffer_slice.get_mapped_range()[..];

        use image::{ImageBuffer, Rgba};
        let buffer: RgbaImage = ImageBuffer::<Rgba<u8>, _>::from_raw(
            render_context.size.x,
            render_context.size.y,
            data.to_owned(),
        )
        .unwrap();
        // buffer.save("image.png").unwrap();
        image = Some(DynamicImage::ImageRgba8(buffer));
    }
    output_buffer.unmap();

    image.unwrap()
}

pub mod texture {
    use std::error::Error;

    use glam::UVec2;
    use image::GenericImageView;
    use wgpu::TextureFormat;

    pub struct Texture {
        #[allow(unused)]
        pub texture: wgpu::Texture,
        pub view: wgpu::TextureView,
        pub sampler: wgpu::Sampler,
    }

    #[allow(unused)]
    impl Texture {
        pub fn from_bytes(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            bytes: &[u8],
            label: &str,
        ) -> Result<Self, Box<dyn Error>> {
            let img = image::load_from_memory(bytes)?;
            Self::from_image(device, queue, &img, Some(label))
        }

        pub fn from_image(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            img: &image::DynamicImage,
            label: Option<&str>,
        ) -> Result<Self, Box<dyn Error>> {
            let rgba = img.to_rgba8();
            let dimensions = img.dimensions();

            let size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            Ok(Self {
                texture,
                view,
                sampler,
            })
        }

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

        pub fn create_depth_texture(device: &wgpu::Device, size: &UVec2, label: &str) -> Self {
            let size = wgpu::Extent3d {
                // 2.
                width: size.x.max(1),
                height: size.y.max(1),
                depth_or_array_layers: 1,
            };
            let desc = wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                       | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };
            let texture = device.create_texture(&desc);

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

            Self {
                texture,
                view,
                sampler,
            }
        }

        pub fn create_texture(device: &wgpu::Device, size: &UVec2, label: &str) -> Self {
            let size = wgpu::Extent3d {
                // 2.
                width: size.x.max(1),
                height: size.y.max(1),
                depth_or_array_layers: 1,
            };
            let desc = wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                       | wgpu::TextureUsages::TEXTURE_BINDING
                       | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            };
            let texture = device.create_texture(&desc);

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                // 4.
                address_mode_u: wgpu::AddressMode::MirrorRepeat,
                address_mode_v: wgpu::AddressMode::MirrorRepeat,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

            Self {
                texture,
                view,
                sampler,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::draw::mask::MaskFaceParts;
    use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    use crate::res::tex::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT};
    use binrw::BinRead;
    use glam::uvec2;
    use image_compare::Algorithm;

    use super::*;
    use std::{error::Error, fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    // #[test]
    // #[allow(clippy::too_many_lines)]
    // fn test_render() -> R {
    //     let mut tex_file = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);
    //     let mut tex_shape = BufReader::new(File::open(SHAPE_MID_DAT)?);

    //     let mut char =
    //         File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Jasmine.charinfo")).unwrap();
    //     let char = NxCharInfo::read(&mut char).unwrap();

    //     let image = pollster::block_on(render_context_wgpu(RenderContext::new(
    //         // &FaceParts::init(&char, 256.0),
    //         &char,
    //         (&mut tex_shape, &mut tex_file),
    //     )?));
    //     let image = image.flipv();

    //     image.save(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/test_output/mask-rendered.png"
    //     ))?;

    //     let reference_image = image::open(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/test_data/jasmine-mask.png"
    //     ))
    //     .unwrap();

    //     let similarity = image_compare::rgb_hybrid_compare(
    //         &image.clone().into_rgb8(),
    //         &reference_image.clone().into_rgb8(),
    //     )
    //     .expect("wrong size!");

    //     similarity
    //         .image
    //         .to_color_map()
    //         .save(concat!(
    //             env!("CARGO_MANIFEST_DIR"),
    //             "/test_output/mask-similarity.png"
    //         ))
    //         .unwrap();

    //     let similarity = image_compare::gray_similarity_structure(
    //         &Algorithm::MSSIMSimple,
    //         &image.into_luma8(),
    //         &reference_image.into_luma8(),
    //     )
    //     .expect("wrong size!");

    //     similarity
    //         .image
    //         .to_color_map()
    //         .save(concat!(
    //             env!("CARGO_MANIFEST_DIR"),
    //             "/test_output/mask-similarity-grey.png"
    //         ))
    //         .unwrap();

    //     Ok(())
    // }
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
