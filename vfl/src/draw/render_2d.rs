use bytemuck::cast_slice;
use glam::vec3;
use image::DynamicImage;
use nalgebra::Matrix4;
use wgpu::{
    CommandEncoder, PipelineCompilationOptions, TexelCopyTextureInfo, TextureFormat, TextureView,
    include_wgsl, util::DeviceExt,
};

use crate::{
    color::nx::{ModulationIntent, modulate},
    res::tex::nx::{RawTexture, ResourceTexture, ResourceTextureFormat, TextureElement},
};

use super::{
    faceline::trivial_quad,
    render_3d::ProgramState,
    wgpu_render::{TextureTransformUniform, Vertex},
};

type Color = [f32; 4];

const NON_REPLACEMENT: Color = [f32::NAN, f32::NAN, f32::NAN, f32::NAN];
#[derive(Debug)]
pub struct Rendered2dShape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tex: RawTexture,
    pub mvp_matrix: Matrix4<f32>,
    pub modulation: ModulationIntent,
    pub opaque: Option<Color>,
}

pub fn texture_format_to_wgpu(tex: TextureElement) -> TextureFormat {
    use TextureFormat as WgpuTextureFormat;
    let format = ResourceTextureFormat::try_from(tex.texture.format).unwrap();
    match format {
        ResourceTextureFormat::R => WgpuTextureFormat::R8Unorm,
        ResourceTextureFormat::Rg => WgpuTextureFormat::Rg8Unorm,
        ResourceTextureFormat::Rgba => WgpuTextureFormat::Rgba8Unorm,
        ResourceTextureFormat::Bc4 => WgpuTextureFormat::Bc4RUnorm,
        ResourceTextureFormat::Bc5 => WgpuTextureFormat::Bc5RgUnorm,
        ResourceTextureFormat::Bc7 => WgpuTextureFormat::Bc7RgbaUnorm,
        ResourceTextureFormat::Astc4x4 => WgpuTextureFormat::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::Unorm,
        },
    }
}

pub fn texture_format_buffer_layout(tex: TextureElement) -> wgpu::TexelCopyBufferLayout {
    use ResourceTextureFormat as Rtf;
    const BC3_BYTES_PER_BLOCK: u32 = 16;
    const BC3_PIXELS_PER_BLOCK: u32 = 4;
    const BC4_BYTES_PER_BLOCK: u32 = 8;

    let format = Rtf::try_from(tex.texture.format).unwrap();

    match format {
        Rtf::R => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(u32::from(tex.texture.width)),
            rows_per_image: None,
        },
        Rtf::Rg => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(2 * u32::from(tex.texture.width)),
            rows_per_image: None,
        },
        Rtf::Astc4x4 | Rtf::Bc7 | Rtf::Bc4 | Rtf::Rgba => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some((16 / 4) * u32::from(tex.texture.width)),
            rows_per_image: None,
        },
        Rtf::Bc5 => wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some((32 / 4) * u32::from(tex.texture.width)),
            rows_per_image: None,
        },
    }
}

impl Rendered2dShape {
    pub fn render_texture_trivial(
        rendered_texture: RawTexture,
        modulation: ModulationIntent,
        opaque: Option<Color>,
        st: &mut impl ProgramState,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let (vertices, indices) = trivial_quad();

        let mvp_matrix = {
            let scale = vec3(-2.0, -2.0, 1.0);
            let scale = nalgebra::Vector3::<f32>::from(Into::<mint::Vector3<f32>>::into(scale));

            nalgebra::Matrix4::new_nonuniform_scaling(&scale)
        };

        let rendered_2d_shape = Rendered2dShape {
            vertices,
            indices,
            tex: rendered_texture,
            mvp_matrix,
            modulation,
            opaque,
        };

        rendered_2d_shape.render(st, texture_view, encoder);
    }

    #[allow(clippy::too_many_lines)]
    pub fn render(
        self,
        st: &mut impl ProgramState,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let vertex_buffer = st
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = st
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let shape_texture_size = wgpu::Extent3d {
            width: u32::from(self.tex.metadata.texture.width),
            height: u32::from(self.tex.metadata.texture.height),
            depth_or_array_layers: 1,
        };
        let shape_diffuse_texture = st.device().create_texture(&wgpu::TextureDescriptor {
            size: shape_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format_to_wgpu(self.tex.metadata),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[
                texture_format_to_wgpu(self.tex.metadata),
                texture_format_to_wgpu(self.tex.metadata).add_srgb_suffix(),
            ],
        });

        st.queue().write_texture(
            TexelCopyTextureInfo {
                texture: &shape_diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.tex.bytes,
            texture_format_buffer_layout(self.tex.metadata),
            shape_texture_size,
        );

        let shape_diffuse_texture_view =
            shape_diffuse_texture.create_view(&wgpu::TextureViewDescriptor {
                format: Some(texture_format_to_wgpu(self.tex.metadata)),
                ..Default::default()
            });
        let shape_diffuse_sampler = st.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            st.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let shape_diffuse_bind_group = st.device().create_bind_group(&wgpu::BindGroupDescriptor {
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

        let mvp_matrix = self.mvp_matrix.into();
        let mvp_uniform = TextureTransformUniform {
            mvp_matrix,
            channel_replacements_r: self.modulation.channels[0],
            channel_replacements_g: self.modulation.channels[1],
            channel_replacements_b: self.modulation.channels[2],
            texture_type: (Into::<u8>::into(self.modulation.mode)).into(),
            pad: Default::default(),
        };

        let mvp_buffer = st
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MvpMatrix Buffer"),
                contents: cast_slice(&[mvp_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let mvp_bind_group_layout =
            st.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let mvp_bind_group = st.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mvp_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_buffer.as_entire_binding(),
            }],
            label: Some("mvp_bind_group"),
        });

        let shader_module = st
            .device()
            .create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            st.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &mvp_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline = st
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: st.surface_fmt().add_srgb_suffix(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
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
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match self.opaque {
                            Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: r.into(),
                                g: g.into(),
                                b: b.into(),
                                a: a.into(),
                            }),
                            None => wgpu::LoadOp::Load,
                        },
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

            render_pass.draw_indexed(0..self.indices.len().try_into().unwrap(), 0, 0..1);
        }
    }
}
