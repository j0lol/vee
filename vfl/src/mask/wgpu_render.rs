use super::{FacePart, FaceParts, ImageOrigin, TEX_SCALE_X, TEX_SCALE_Y};
use crate::{
    charinfo::nx::NxCharInfo,
    color::{
        cafe::{
            EYE_COLOR_B, EYE_COLOR_G, EYE_COLOR_R, GLASS_COLOR_R, HAIR_COLOR, MOUTH_COLOR_B,
            MOUTH_COLOR_G, MOUTH_COLOR_R,
        },
        nx::{ColorModulated, modulate},
    },
    shape_load::nx::ResourceShape,
    tex_load::nx::{ResourceTexture, ResourceTextureFormat, TextureElement},
};
use binrw::BinRead;
use glam::{UVec2, uvec2, vec3};
use image::{DynamicImage, RgbaImage};
use nalgebra::Matrix4;
use std::{error::Error, fs::File, io::BufReader};
use wgpu::{DeviceDescriptor, TexelCopyTextureInfo, util::DeviceExt};

pub const FACE_OUTPUT_SIZE: u16 = 1024;
const SHADER: &str = include_str!("./shader.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
struct TextureTransformUniform {
    mvp_matrix: [[f32; 4]; 4],
    channel_replacements_r: [f32; 4],
    channel_replacements_g: [f32; 4],
    channel_replacements_b: [f32; 4],
    texture_type: u32,
    pad: [u32; 3],
}

const NON_REPLACEMENT: [f32; 4] = [f32::NAN, f32::NAN, f32::NAN, f32::NAN];
struct RenderShape {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    tex: DynamicImage,
    mvp_matrix: Matrix4<f32>,
    texture_type: crate::tex_load::nx::ResourceTextureFormat,
    channel_replacements: [[f32; 4]; 3],
}

pub struct RenderContext {
    size: UVec2,
    shape: Vec<RenderShape>,
}

impl RenderContext {
    #[allow(clippy::too_many_lines)]
    pub fn new(
        char: &NxCharInfo,
        (file_shape, file_texture): (&mut BufReader<File>, &mut BufReader<File>),
    ) -> Result<Self, Box<dyn Error>> {
        let res_texture = ResourceTexture::read(file_texture)?;

        let mask = FaceParts::init(char, 256.0);

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

                RenderShape {
                    vertices,
                    indices,
                    tex: image::DynamicImage::ImageRgba8(tex),
                    mvp_matrix: mtx,
                    texture_type: ResourceTextureFormat::try_from(tex_data.texture.format).unwrap(),
                    channel_replacements: modulate(modulated, char),
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

    pub fn new_glasses(
        char: &NxCharInfo,
        (file_shape, file_texture): (&mut BufReader<File>, &mut BufReader<File>),
    ) -> Result<Self, Box<dyn Error>> {
        let res_shape = ResourceShape::read(file_shape)?;
        let res_texture = ResourceTexture::read(file_texture)?;

        let nose_translate =
            res_shape.face_line_transform[char.faceline_type as usize].nose_translate;

        let glasses = dbg!(FaceParts::init_glasses(char, 256.0, nose_translate));

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

        let glass_l_render_shape = RenderShape {
            vertices,
            indices,
            tex: image::DynamicImage::ImageRgba8(tex.clone()),
            mvp_matrix: mtx,
            texture_type: ResourceTextureFormat::try_from(tex_data.texture.format).unwrap(),
            channel_replacements: [
                GLASS_COLOR_R[char.glass_color as usize],
                NON_REPLACEMENT,
                NON_REPLACEMENT,
            ],
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

        let glass_r_render_shape = RenderShape {
            vertices,
            indices,
            tex: image::DynamicImage::ImageRgba8(tex),
            mvp_matrix: mtx,
            texture_type: ResourceTextureFormat::try_from(tex_data.texture.format).unwrap(),
            channel_replacements: [
                GLASS_COLOR_R[char.glass_color as usize],
                NON_REPLACEMENT,
                NON_REPLACEMENT,
            ],
        };

        Ok(RenderContext {
            size: uvec2(512, 256),
            shape: vec![glass_l_render_shape, glass_r_render_shape],
        })
    }
}

fn model_view_matrix(
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
fn quad(
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
        vec3(x, y, 0.0).into(),
        vec3(width, height, 1.0).into(),
        rot_z,
    );

    let p_mtx = Matrix4::new_orthographic(0.0, resolution, 0.0, resolution, -200.0, 200.0);
    let mvp_mtx = p_mtx * mv_mtx;

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
    }

    (
        vec![
            Vertex {
                position: v2(1.0 + base_x, -0.5),
                tex_coords: [s0, 0.0],
            },
            Vertex {
                position: v2(1.0 + base_x, 0.5),
                tex_coords: [s0, 1.0],
            },
            Vertex {
                position: v2(base_x, 0.5),
                tex_coords: [s1, 1.0],
            },
            Vertex {
                position: v2(base_x, -0.5),
                tex_coords: [s1, 0.0],
            },
        ],
        vec![0, 1, 2, 0, 2, 3],
        mvp_mtx,
    )
}

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
            channel_replacements_r: shape.channel_replacements[0],
            channel_replacements_g: shape.channel_replacements[1],
            channel_replacements_b: shape.channel_replacements[2],
            texture_type: (Into::<u8>::into(shape.texture_type)).into(),
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

#[cfg(test)]
mod tests {
    use crate::mask::{FaceParts, NxCharInfo};
    use crate::shape_load::nx::{ResourceShape, SHAPE_MID_DAT};
    use crate::tex_load::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT};
    use binrw::BinRead;
    use glam::uvec2;

    use super::*;
    use std::{error::Error, fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_render() -> R {
        let mut tex_file = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);
        let mut tex_shape = BufReader::new(File::open(SHAPE_MID_DAT)?);

        let mut char =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Jasmine.charinfo")).unwrap();
        let char = NxCharInfo::read(&mut char).unwrap();

        let image = pollster::block_on(render_context_wgpu(RenderContext::new(
            // &FaceParts::init(&char, 256.0),
            &char,
            (&mut tex_shape, &mut tex_file),
        )?));

        image.save("image.png")?;

        Ok(())
    }
}
