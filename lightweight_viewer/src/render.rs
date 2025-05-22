use crate::wgpu_color_to_vec4;
use std::{fs::File, io::BufReader, sync::Arc};

use glam::{Vec2, Vec3, Vec4, vec2, vec4};
use image::DynamicImage;
use vfl::{
    charinfo::nx::{BinRead, NxCharInfo},
    color::nx::linear::FACELINE_COLOR,
    draw::{
        mask::ImageOrigin,
        wgpu_render::{
            RenderContext, RenderShape as Rendered2dShape, SHADER, TextureTransformUniform, Vertex,
            cast_slice, quad, render_context_wgpu,
        },
    },
    res::{
        shape::nx::{
            GenericResourceShape, ResourceShape, SHAPE_MID_DAT, Shape, ShapeData, ShapeElement,
        },
        tex::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT},
    },
};
use wgpu::{
    Backends, BlendState, CommandEncoder, TexelCopyTextureInfo, Texture, TextureFormat,
    TextureView, include_wgsl, util::DeviceExt,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{OVERLAY_REBECCA_PURPLE, State, texture};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CharShapeUniform {
    diffuse_color: [f32; 4],
    position: [f32; 3],
    _pad: f32,
}

#[deprecated]
pub trait RenderThatContext {
    fn render(self, st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder);
    fn render_2d_shape(
        shape: Rendered2dShape,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    );
    // fn render_texture(
    //     tex: DynamicImage,
    //     size: Vec2,
    //     st: &mut State,
    //     texture_view: &TextureView,
    //     encoder: &mut CommandEncoder,
    // );
    fn render_3d_shape(
        shape: Rendered3dShape,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    );
}
impl RenderThatContext for RenderContext {
    fn render(self, st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
        for shape in self.shape {
            RenderContext::render_2d_shape(shape, st, texture_view, encoder)
        }
    }

    fn render_2d_shape(
        shape: Rendered2dShape,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let vertex_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&shape.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&shape.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let shape_texture_rgba = shape.tex.to_rgba8();
        let shape_texture_dimensions = shape_texture_rgba.dimensions();
        let shape_texture_size = wgpu::Extent3d {
            width: shape_texture_dimensions.0,
            height: shape_texture_dimensions.1,
            depth_or_array_layers: 1,
        };
        let shape_diffuse_texture = st.device.create_texture(&wgpu::TextureDescriptor {
            size: shape_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: st.surface_format.add_srgb_suffix(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[st.surface_format],
        });

        st.queue.write_texture(
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
        let shape_diffuse_sampler = st.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            st.device
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

        let shape_diffuse_bind_group = st.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let mvp_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MvpMatrix Buffer"),
                contents: cast_slice(&[mvp_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let mvp_bind_group_layout =
            st.device
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

        let mvp_bind_group = st.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mvp_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_buffer.as_entire_binding(),
            }],
            label: Some("mvp_bind_group"),
        });

        let shader = wgpu::ShaderSource::Wgsl(SHADER.into());
        let shader_module = st
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: shader,
            });
        let render_pipeline_layout =
            st.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &mvp_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline = st
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: st.surface_format.add_srgb_suffix(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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

    fn render_3d_shape(
        shape: Rendered3dShape,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let vertex_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&shape.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&shape.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let char_shape_uniform = CharShapeUniform {
            diffuse_color: shape.color.into(),
            position: shape.position.into(),
            _pad: 0.0,
        };
        let char_shape_buffer = st
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cs Buffer"),
                contents: cast_slice(&[char_shape_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let char_shape_bind_group_layout =
            st.device
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

        let char_shape_bind_group = st.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &char_shape_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: char_shape_buffer.as_entire_binding(),
            }],
            label: Some("cs_bind_group"),
        });

        let mut bind_group_layouts =
            vec![&st.camera_bind_group_layout, &char_shape_bind_group_layout];

        let projected_texture_bind_group_layout =
            st.device
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
        if shape.texture.is_some() {
            bind_group_layouts.push(&projected_texture_bind_group_layout);
        }

        let shader_module = if shape.texture.is_some() {
            st.device
                .create_shader_module(include_wgsl!("shader-texture.wgsl"))
        } else {
            st.device.create_shader_module(include_wgsl!("shader.wgsl"))
        };

        // Optional projected texture
        let projected_texture_bind_group = shape.texture.map(|texture| {
            st.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            st.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let render_pipeline = st
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: st.surface_format.add_srgb_suffix(),
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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
                    format: texture::Texture::DEPTH_FORMAT,
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
                    view: &st.depth_texture.view,
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
            render_pass.set_bind_group(0, &st.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &char_shape_bind_group, &[]);
            render_pass.set_bind_group(2, &projected_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..shape.indices.len() as u32, 0, 0..1);
        }
    }
}

#[deprecated]
pub struct Rendered3dShape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub color: Vec4,
    pub texture: Option<texture::Texture>,
    position: Vec3,
}

// I'm in a fucking horror of my own design
pub fn shape_data_to_render_3d_shape(
    d: ShapeData,
    shape: Shape,
    color: usize,
    position: Vec3,
    projected_texture: Option<texture::Texture>,
) -> Rendered3dShape {
    let mut vertices: Vec<Vertex> = vec![];
    let tex_coords = d
        .uvs
        .unwrap_or(vec![[f32::NAN, f32::NAN]; d.positions.len()]); // Go on, return NULL. See if I care.
    let normals = d.normals.unwrap();

    for i in 0..d.positions.len() {
        vertices.push(Vertex {
            position: d.positions[i],
            tex_coords: tex_coords[i],
            normal: normals[i],
        })
    }

    let indices = d.indices.iter().map(|x| u32::from(*x)).collect();

    Rendered3dShape {
        vertices,
        indices,
        color: match shape {
            Shape::HairNormal => vfl::color::nx::linear::COMMON_COLOR[color].into(),
            Shape::FaceLine | Shape::ForeheadNormal | Shape::Nose => FACELINE_COLOR[color].into(),
            Shape::Glasses => vec4(1.0, 0.0, 0.0, 0.0),
            Shape::NoseLine => vec4(0.0, 1.0, 0.0, 0.7),
            Shape::Mask => vec4(0.0, 0.0, 0.0, 0.0),
            _ => wgpu_color_to_vec4(OVERLAY_REBECCA_PURPLE),
        },
        texture: projected_texture,
        position,
    }
}

fn simple_quad(x: f32, y: f32, width: f32, height: f32) -> (Vec<Vertex>, Vec<u32>) {
    // let base_x: f32;
    // let s0: f32;
    // let s1: f32;

    // let mv_mtx = model_view_matrix(
    //     vec3(x, y, 0.0).into(),
    //     vec3(width, height, 1.0).into(),
    //     rot_z,
    // );

    // let p_mtx = Matrix4::new_orthographic(0.0, resolution, 0.0, resolution, -200.0, 200.0);
    // let mut mvp_mtx = p_mtx * mv_mtx;

    // *mvp_mtx.get_mut((1, 1)).unwrap() *= -1.0;

    // match origin {
    //     ImageOrigin::Center => {
    //         base_x = -0.5;
    //         s0 = 1.0;
    //         s1 = 0.0;
    //     }
    //     ImageOrigin::Right => {
    //         base_x = -1.0;
    //         s0 = 1.0;
    //         s1 = 0.0;
    //     }
    //     ImageOrigin::Left => {
    //         base_x = 0.0;
    //         s0 = 0.0;
    //         s1 = 1.0;
    //     }
    // }

    (
        vec![
            Vertex {
                position: [1.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
        ],
        vec![0, 1, 2, 0, 2, 3],
    )
}
