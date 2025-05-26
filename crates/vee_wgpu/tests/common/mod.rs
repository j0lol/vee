use glam::uvec2;
use std::fs::File;
use vee_wgpu::{ProgramState, headless::HeadlessRenderer, texture::TextureBundle};
use vfl::{
    charinfo::nx::{BinRead, NxCharInfo},
    res::{
        shape::nx::{ResourceShape, SHAPE_MID_DAT},
        tex::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT},
    },
};
use wgpu::CommandEncoder;

#[allow(unused)]
pub struct Everything {
    pub render: HeadlessRenderer,
    pub encoder: CommandEncoder,
    pub texture: TextureBundle,
    pub char: NxCharInfo,
    pub shape_header: ResourceShape,
    pub texture_header: ResourceTexture,
    pub shape_data: Vec<u8>,
    pub texture_data: Vec<u8>,
}

pub fn setup_renderer_linear_color() -> Everything {
    let renderer = HeadlessRenderer::new();
    let encoder = renderer
        .device()
        .create_command_encoder(&Default::default());

    let texture =
        TextureBundle::create_texture_linear_color(&renderer.device(), &uvec2(512, 512), "tex");

    let mut char_info = File::open(format!(
        "{}/resources_here/jasmine.charinfo",
        env!("CARGO_WORKSPACE_DIR"),
    ))
    .unwrap();
    let char = NxCharInfo::read(&mut char_info).unwrap();

    let shape_header = ResourceShape::read(&mut File::open(SHAPE_MID_DAT).unwrap()).unwrap();
    let texture_header =
        ResourceTexture::read(&mut File::open(TEXTURE_MID_SRGB_DAT).unwrap()).unwrap();
    let shape_data = std::fs::read(SHAPE_MID_DAT).unwrap();
    let texture_data = std::fs::read(TEXTURE_MID_SRGB_DAT).unwrap();

    Everything {
        render: renderer,
        encoder,
        texture,
        char,
        shape_header,
        texture_header,
        shape_data,
        texture_data,
    }
}

#[allow(dead_code)]
pub fn setup_renderer() -> Everything {
    let renderer = HeadlessRenderer::new();
    let encoder = renderer
        .device()
        .create_command_encoder(&Default::default());

    let texture = TextureBundle::create_texture(&renderer.device(), &uvec2(512, 512), "tex");

    let mut char_info = File::open(format!(
        "{}/resources_here/jasmine.charinfo",
        env!("CARGO_WORKSPACE_DIR"),
    ))
    .unwrap();
    let char = NxCharInfo::read(&mut char_info).unwrap();

    let shape_header = ResourceShape::read(&mut File::open(SHAPE_MID_DAT).unwrap()).unwrap();
    let texture_header =
        ResourceTexture::read(&mut File::open(TEXTURE_MID_SRGB_DAT).unwrap()).unwrap();
    let shape_data = std::fs::read(SHAPE_MID_DAT).unwrap();
    let texture_data = std::fs::read(TEXTURE_MID_SRGB_DAT).unwrap();

    Everything {
        render: renderer,
        encoder,
        texture,
        char,
        shape_header,
        texture_header,
        shape_data,
        texture_data,
    }
}
