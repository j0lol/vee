use bytemuck::cast_slice;
use texture::TextureBundle;
use vfl::draw::Vertex;
use wgpu::{BlendState, CommandEncoder, TextureView};
use wgpu::{PipelineCompilationOptions, TexelCopyTextureInfo, include_wgsl, util::DeviceExt};

pub mod headless;

pub type Model3d = vfl::draw::render_3d::GenericModel3d<TextureBundle>;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CharShapeUniform {
    diffuse_color: [f32; 4],
    position: [f32; 3],
    _pad: f32,
    scale: [f32; 3],
    _pad2: f32,
}

trait UniformBuffer {
    const ATTRIBS: [wgpu::VertexAttribute; 3];

    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
impl UniformBuffer for Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
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

pub trait ProgramState {
    fn device(&self) -> wgpu::Device;
    fn queue(&self) -> wgpu::Queue;
    fn camera_bgl(&self) -> &wgpu::BindGroupLayout;
    fn camera_bg(&self) -> &wgpu::BindGroup;
    fn surface_fmt(&self) -> wgpu::TextureFormat;
    fn depth_texture(&self) -> &TextureBundle;

    fn draw_texture(
        &mut self,
        tex: vfl::draw::DrawableTexture,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.draw_model_2d(&mut tex.model_2d(), view, encoder)
    }

    fn draw_model_2d(
        &mut self,
        mesh: &mut vfl::draw::render_2d::Model2d,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        if let Some(ref label) = mesh.label {
            encoder.push_debug_group(&format!("Texture {label}"));
        }

        let vertex_buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let shape_texture_rgba = mesh.tex.to_rgba8();
        let shape_texture_dimensions = shape_texture_rgba.dimensions();
        let shape_texture_size = wgpu::Extent3d {
            width: shape_texture_dimensions.0,
            height: shape_texture_dimensions.1,
            depth_or_array_layers: 1,
        };
        let shape_diffuse_texture = self.device().create_texture(&wgpu::TextureDescriptor {
            size: shape_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_fmt(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[self.surface_fmt()],
        });

        self.queue().write_texture(
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
        let shape_diffuse_sampler = self.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            self.device()
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

        let shape_diffuse_bind_group =
            self.device().create_bind_group(&wgpu::BindGroupDescriptor {
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

        let mvp_matrix = mesh.mvp_matrix.to_cols_array_2d();
        let mvp_uniform = TextureTransformUniform {
            mvp_matrix,
            channel_replacements_r: mesh.modulation.channels[0],
            channel_replacements_g: mesh.modulation.channels[1],
            channel_replacements_b: mesh.modulation.channels[2],
            texture_type: Into::<u8>::into(mesh.modulation.mode).into(),
            pad: Default::default(),
        };

        let mvp_buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MvpMatrix Buffer"),
                contents: cast_slice(&[mvp_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let mvp_bind_group_layout =
            self.device()
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

        let mvp_bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mvp_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_buffer.as_entire_binding(),
            }],
            label: Some("mvp_bind_group"),
        });

        let shader_module = self
            .device()
            .create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            self.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &mvp_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline =
            self.device()
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
                            format: self.surface_fmt(),
                            blend: Some(BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
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
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match mesh.opaque {
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

            render_pass.draw_indexed(0..mesh.indices.len().try_into().unwrap(), 0, 0..1);
        }

        if mesh.label.is_some() {
            encoder.pop_debug_group();
        }
    }

    fn draw_model_3d(
        &mut self,
        mesh: &mut Model3d,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let vertex_buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let char_shape_uniform = CharShapeUniform {
            diffuse_color: mesh.color.into(),
            position: mesh.position.into(),
            _pad: 0.0,
            scale: mesh.scale.into(),
            _pad2: 0.0,
        };
        let char_shape_buffer =
            self.device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Cs Buffer"),
                    contents: cast_slice(&[char_shape_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let char_shape_bind_group_layout =
            self.device()
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

        let char_shape_bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &char_shape_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: char_shape_buffer.as_entire_binding(),
            }],
            label: Some("cs_bind_group"),
        });

        let mut bind_group_layouts = vec![self.camera_bgl(), &char_shape_bind_group_layout];

        let projected_texture_bind_group_layout =
            self.device()
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
        if mesh.texture.is_some() {
            bind_group_layouts.push(&projected_texture_bind_group_layout);
        }

        let shader_module = self
            .device()
            .create_shader_module(include_wgsl!("shader_3d.wgsl"));

        // Optional projected texture
        let projected_texture_bind_group = mesh.texture.as_ref().map(|texture| {
            self.device().create_bind_group(&wgpu::BindGroupDescriptor {
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
            self.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let render_pipeline =
            self.device()
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
                        entry_point: if mesh.texture.is_some() {
                            Some("fs_main")
                        } else {
                            Some("fs_color_only")
                        },
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.surface_fmt().add_srgb_suffix(),
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
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: TextureBundle::DEPTH_FORMAT,
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
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture().view,
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
            render_pass.set_bind_group(0, self.camera_bg(), &[]);
            render_pass.set_bind_group(1, &char_shape_bind_group, &[]);
            render_pass.set_bind_group(2, &projected_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..mesh.indices.len().try_into().unwrap(), 0, 0..1);
        }
    }
}

pub mod texture {
    use std::error::Error;
    use std::fmt::Debug;

    use glam::UVec2;
    use image::GenericImageView;
    use wgpu::TextureFormat;

    pub struct TextureBundle {
        #[allow(unused)]
        pub texture: wgpu::Texture,
        pub view: wgpu::TextureView,
        pub sampler: wgpu::Sampler,
    }
    impl Debug for TextureBundle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Texture").finish()
        }
    }

    #[allow(unused)]
    impl TextureBundle {
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
                format: TextureFormat::Rgba8UnormSrgb,
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

        pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float; // 1.

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

        /// Outputs a linear color texture instead of an srgb color texture. Should replace.
        pub fn create_texture_linear_color(
            device: &wgpu::Device,
            size: &UVec2,
            label: &str,
        ) -> Self {
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
                format: TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                       | wgpu::TextureUsages::TEXTURE_BINDING
                       | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb],
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
                view_formats: &[TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb],
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
