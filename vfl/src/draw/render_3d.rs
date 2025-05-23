use bytemuck::cast_slice;
use glam::{Vec3, Vec4};
use wgpu::{
    BlendState, CommandEncoder, PipelineCompilationOptions, TextureView, include_wgsl,
    util::DeviceExt,
};

use super::wgpu_render::{Vertex, texture};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CharShapeUniform {
    diffuse_color: [f32; 4],
    position: [f32; 3],
    _pad: f32,
}

pub trait ProgramState {
    fn device(&self) -> wgpu::Device;
    fn queue(&self) -> wgpu::Queue;
    fn camera_bgl(&self) -> &wgpu::BindGroupLayout;
    fn camera_bg(&self) -> &wgpu::BindGroup;
    fn surface_fmt(&self) -> wgpu::TextureFormat;
    fn depth_texture(&self) -> &texture::TextureBundle;
}

#[derive(Default, Debug)]
pub struct Rendered3dShape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub color: Vec4,
    pub texture: Option<crate::draw::wgpu_render::texture::TextureBundle>,
    pub position: Vec3,
}

impl Rendered3dShape {
    #[allow(clippy::too_many_lines)]
    pub fn render(
        &mut self,
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

        let char_shape_uniform = CharShapeUniform {
            diffuse_color: self.color.into(),
            position: self.position.into(),
            _pad: 0.0,
        };
        let char_shape_buffer = st
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cs Buffer"),
                contents: cast_slice(&[char_shape_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let char_shape_bind_group_layout =
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
                    label: Some("cs_group_layout"),
                });

        let char_shape_bind_group = st.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &char_shape_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: char_shape_buffer.as_entire_binding(),
            }],
            label: Some("cs_bind_group"),
        });

        let mut bind_group_layouts = vec![st.camera_bgl(), &char_shape_bind_group_layout];

        let projected_texture_bind_group_layout =
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
        if self.texture.is_some() {
            bind_group_layouts.push(&projected_texture_bind_group_layout);
        }

        let shader_module = if self.texture.is_some() {
            st.device()
                .create_shader_module(include_wgsl!("shader-3d-texture.wgsl"))
        } else {
            st.device()
                .create_shader_module(include_wgsl!("shader-3d.wgsl"))
        };

        // Optional projected texture
        let projected_texture_bind_group = self.texture.as_ref().map(|texture| {
            st.device().create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &projected_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            })
        });

        let render_pipeline_layout =
            st.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
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
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None, // TODO toggle
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::TextureBundle::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                }),
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

        // Scoped lifetime to drop the render pass
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &st.depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, st.camera_bg(), &[]);
            render_pass.set_bind_group(1, &char_shape_bind_group, &[]);
            render_pass.set_bind_group(2, &projected_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..self.indices.len().try_into().unwrap(), 0, 0..1);
        }
    }
}
