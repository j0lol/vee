use ffl_runner::FFlRunner;
use glam::{Mat4, uvec2};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    str::FromStr,
};
use vee_parse::{BinRead, NxCharInfo};
use vee_resources::shape::ResourceShape;
use vee_resources::tex::ResourceTexture;
use vee_wgpu::texture::TextureBundle;
use vee_wgpu::{ProgramState, headless::HeadlessRenderer};
use wgpu::CommandEncoder;

pub mod ffl_runner;

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
    let shape_file_path = format!(
        "{}resources_here/ShapeMid.dat",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    );
    let tex_file_path = format!(
        "{}resources_here/NXTextureMidSRGB.dat",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    );

    let renderer = HeadlessRenderer::new(&shape_file_path, &tex_file_path);
    let encoder = renderer
        .device()
        .create_command_encoder(&Default::default());

    let texture =
        TextureBundle::create_texture_linear_color(&renderer.device(), &uvec2(512, 512), "tex");

    let mut char_info = File::open(format!(
        "{}/resources_here/jasmine.charinfo",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    ))
    .unwrap();
    let char = NxCharInfo::read(&mut char_info).unwrap();

    let shape_header = ResourceShape::read(&mut File::open(&shape_file_path).unwrap()).unwrap();
    let texture_header = ResourceTexture::read(&mut File::open(&tex_file_path).unwrap()).unwrap();
    let shape_data = fs::read(&shape_file_path).unwrap();
    let texture_data = fs::read(&tex_file_path).unwrap();

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
    let shape_file_path = format!(
        "{}resources_here/ShapeMid.dat",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    );
    let tex_file_path = format!(
        "{}resources_here/NXTextureMidSRGB.dat",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    );

    let renderer = HeadlessRenderer::new(&shape_file_path, &tex_file_path);
    let encoder = renderer
        .device()
        .create_command_encoder(&Default::default());

    let texture = TextureBundle::create_texture(&renderer.device(), &uvec2(512, 512), "tex");

    let mut char_info = File::open(format!(
        "{}/resources_here/jasmine.charinfo",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    ))
    .unwrap();
    let char = NxCharInfo::read(&mut char_info).unwrap();

    let shape_header = ResourceShape::read(&mut File::open(&shape_file_path).unwrap()).unwrap();
    let texture_header = ResourceTexture::read(&mut File::open(&tex_file_path).unwrap()).unwrap();
    let shape_data = fs::read(&shape_file_path).unwrap();
    let texture_data = fs::read(&tex_file_path).unwrap();

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

#[derive(Serialize, Deserialize)]
pub struct MaskTestData {
    pub left_eye: Mat4,
    pub right_eye: Mat4,
    pub left_eyebrow: Mat4,
    pub right_eyebrow: Mat4,
    pub mouth: Mat4,
    pub mole: Option<Mat4>,
}

pub fn get_mask_data() -> MaskTestData {
    let file = PathBuf::from(format!(
        "{}/test_data/inputs/jasmine_mask_mtx.json",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
    ));

    if fs::exists(&file).unwrap() {
        serde_json::from_reader(BufReader::new(File::open(file).unwrap())).unwrap()
    } else {
        let mut ffl = FFlRunner {
            dir: PathBuf::from_str(&format!(
                "{}../FFL-Testing/",
                std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
            ))
            .unwrap(),
        };

        // This takes about two seconds.
        ffl.run_ffl_testing().unwrap();

        let matrices = [
            "eye0mtx.txt",
            "eye1mtx.txt",
            "brow0mtx.txt",
            "brow1mtx.txt",
            "mouthmtx.txt",
        ];
        let [left_eye, right_eye, left_eyebrow, right_eyebrow, mouth] =
            matrices.map(|file| ffl.get_resultant_mtx44(file).unwrap());

        let data = MaskTestData {
            left_eye,
            right_eye,
            left_eyebrow,
            right_eyebrow,
            mouth,
            mole: None,
        };

        fs::write(file, serde_json::to_string_pretty(&data).unwrap()).unwrap();

        data
    }
}
