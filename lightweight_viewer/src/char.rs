use std::{fs::File, io::BufReader, sync::Arc};

use glam::{Vec3, Vec4};
use vfl::{
    charinfo::nx::{BinRead, NxCharInfo},
    color::nx::linear::FACELINE_COLOR,
    draw::wgpu_render::{
        RenderContext, RenderShape as Rendered2dShape, SHADER, TextureTransformUniform, Vertex,
        cast_slice, render_context_wgpu,
    },
    res::{
        shape::nx::{
            GenericResourceShape, ResourceShape, SHAPE_MID_DAT, Shape, ShapeData, ShapeElement,
        },
        tex::nx::TEXTURE_MID_SRGB_DAT,
    },
};
use wgpu::{
    Backends, CommandEncoder, TexelCopyTextureInfo, Texture, TextureFormat, TextureView,
    include_wgsl, util::DeviceExt,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{State, texture};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CharShapeUniform {
    diffuse_color: [f32; 4],
}

trait RenderThatContext {
    fn render(self, st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder);
    fn render_2d_shape(
        shape: Rendered2dShape,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    );
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
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };
        let shape_diffuse_texture = st.device.create_texture(&wgpu::TextureDescriptor {
            size: shape_texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB, so we need to reflect that here.
            format: st.surface_format.add_srgb_suffix(),
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
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
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

        let shader_module = st.device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            st.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &st.camera_bind_group_layout,
                        &char_shape_bind_group_layout,
                    ],
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
                        blend: None,
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
            // render_pass.set_bind_group(1, &mvp_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..shape.indices.len() as u32, 0, 0..1);
        }
    }
}

struct Rendered3dShape {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    color: Vec4,
    texture: Option<Texture>,
}

// I'm in a fucking horror of my own design
fn shape_data_to_render_3d_shape(d: ShapeData, shape: Shape, color: usize) -> Rendered3dShape {
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
            Shape::FaceLine | Shape::ForeheadNormal => FACELINE_COLOR[color].into(),
            _ => [1.0, 0.0, 1.0, 1.0].into(),
        },
        texture: None,
    }
}

pub fn draw_mask(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let render_context =
        RenderContext::new(&st.char_info.clone(), (&mut st.shape(), &mut st.texture())).unwrap();

    render_context.render(st, texture_view, encoder);
}

pub fn draw_char(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_shape: ResourceShape = ResourceShape::read(&mut st.shape()).unwrap();
    let GenericResourceShape::Element(mut shape_faceline) = res_shape
        .fetch_shape(
            vfl::res::shape::nx::Shape::FaceLine,
            usize::from(st.char_info.faceline_type),
        )
        .unwrap()
    else {
        panic!()
    };
    let GenericResourceShape::Element(mut shape_hair) = res_shape
        .fetch_shape(
            vfl::res::shape::nx::Shape::HairNormal,
            usize::from(st.char_info.hair_type),
        )
        .unwrap()
    else {
        panic!()
    };

    let mask_texture = crate::texture::Texture::create_texture(
        &st.device,
        &PhysicalSize::<u32>::new(512, 512),
        "mask",
    );

    draw_mask(st, &mask_texture.view, encoder);

    // TODO: add mask to mask model and whatever blablablablablablaballablabla
    RenderContext::render_3d_shape(
        shape_data_to_render_3d_shape(
            shape_faceline.shape_data(&mut st.shape()).unwrap(),
            Shape::FaceLine,
            usize::from(st.char_info.faceline_color),
        ),
        st,
        texture_view,
        encoder,
    );
    RenderContext::render_3d_shape(
        shape_data_to_render_3d_shape(
            shape_hair.shape_data(&mut st.shape()).unwrap(),
            Shape::HairNormal,
            usize::from(st.char_info.hair_color),
        ),
        st,
        texture_view,
        encoder,
    );
}
