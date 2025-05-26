use super::TEX_SCALE_X;
use super::TEX_SCALE_Y;
use super::Vertex;
use super::positioning::{FacePart, ImageOrigin, MaskFaceParts};
use super::render_2d::Model2d;
use crate::{
    charinfo::nx::NxCharInfo,
    color::nx::{ColorModulated, modulate},
    res::tex::nx::{ResourceTexture, TextureElement},
};
use glam::vec3;
use nalgebra::Matrix4;

pub const FACE_OUTPUT_SIZE: u16 = 512;
pub use bytemuck::cast_slice;

const NON_REPLACEMENT: [f32; 4] = [f32::NAN, f32::NAN, f32::NAN, f32::NAN];

/// Returns the models needed for the mask texture.
/// # Panics
/// - Panics if image loading fails.
pub fn mask_texture_meshes(
    char: &NxCharInfo,
    res_texture: &ResourceTexture,
    file_texture: &[u8],
) -> Vec<Model2d> {
    let mask = MaskFaceParts::init(char, 256.0);

    let make_shape = |part: FacePart, modulated: ColorModulated, tex_data: TextureElement| {
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

        Model2d {
            vertices,
            indices,
            tex: image::DynamicImage::ImageRgba8(tex),
            mvp_matrix: mtx,
            modulation: modulate(modulated, char),
            opaque: None,
            label: Some(format!("{modulated:?}")),
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

    vec![left_eye, right_eye, left_brow, right_brow, mouth]
}

pub fn model_view_matrix(
    translation: mint::Vector3<f32>,
    scale: mint::Vector3<f32>,
    rot_z: f32,
) -> nalgebra::Matrix4<f32> {
    let scale = nalgebra::Vector3::<f32>::from(scale);
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
/// # Panics
/// Shouldn't panic!
pub fn quad(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rot_z: f32,
    origin: ImageOrigin,
    resolution: f32,
) -> (Vec<Vertex>, Vec<u32>, nalgebra::Matrix4<f32>) {
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

    *mvp_mtx
        .get_mut((1, 1))
        .expect("That index is never going to be out of bounds") *= -1.0;

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
        ImageOrigin::Left | ImageOrigin::Ignore => {
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

pub fn trivial_quad() -> (Vec<Vertex>, Vec<u32>) {
    (
        vec![
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
        ],
        vec![0, 1, 2, 0, 2, 3],
    )
}
pub fn bgr_to_rgb([b, g, r, a]: [f32; 4]) -> [f32; 4] {
    [r, g, b, a]
}

#[cfg(test)]
mod tests {
    // use crate::charinfo;
    // use crate::charinfo::nx::NxCharInfo;
    // use crate::color::nx::modulate;
    // use crate::draw::faceline::{bgr_to_rgb, trivial_quad};
    // use crate::draw::render_2d::Model2d;
    // use crate::draw::render_3d::ProgramState;
    // use crate::draw::wgpu_render::{HeadlessRenderer, Vertex, model_view_matrix, quad, texture};
    // use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    // use crate::res::tex::nx::{ResourceTexture, ResourceTextureFormat, TEXTURE_MID_SRGB_DAT};
    // use binrw::BinRead;
    // use glam::{uvec2, vec3};
    // use nalgebra::Matrix4;
    // use std::error::Error;
    // use std::{fs::File, io::BufReader};
    // use wgpu::CommandEncoder;

    // type R = Result<(), Box<dyn Error>>;

    // #[test]
    // fn faceline_makeup() -> R {
    //     let mut headless_renderer = HeadlessRenderer::new();
    //     let mut encoder: CommandEncoder = headless_renderer
    //         .device()
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    //     let char = NxCharInfo::read(&mut File::open("../charline.charinfo").unwrap()).unwrap();
    //     let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

    //     let res_texture = ResourceTexture::read(&mut bin)?;

    //     let tex = res_texture.makeup[char.faceline_make as usize]
    //         .get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;
    //     let tex = image::DynamicImage::ImageRgba8(tex.unwrap());

    //     let target_texture =
    //         texture::Texture::create_texture(&headless_renderer.device(), &uvec2(256, 512), "");

    //     Rendered2dShape::render_texture_trivial(
    //         tex,
    //         modulate(crate::color::nx::ColorModulated::FacelineMakeup, &char),
    //         Some(bgr_to_rgb(
    //             crate::color::nx::srgb::FACELINE_COLOR[usize::from(char.faceline_color)],
    //         )),
    //         &mut headless_renderer,
    //         &target_texture.view,
    //         &mut encoder,
    //     );

    //     let image = headless_renderer.output_texture(&target_texture, encoder);

    //     println!("Done!");
    //     image.save(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/test_output/faceline_makeup.png"
    //     ))?;

    //     Ok(())
    // }

    // #[test]
    // fn faceline_beard() -> R {
    //     let mut headless_renderer = HeadlessRenderer::new();
    //     let mut encoder: CommandEncoder = headless_renderer
    //         .device()
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    //     let char = NxCharInfo::read(&mut File::open("../testguy.charinfo").unwrap()).unwrap();
    //     let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

    //     let res_texture = ResourceTexture::read(&mut bin)?;

    //     let tex = res_texture.beard[0]
    //         .get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;
    //     let tex = image::DynamicImage::ImageRgba8(tex.unwrap());

    //     let target_texture =
    //         texture::Texture::create_texture(&headless_renderer.device(), &uvec2(256, 512), "");

    //     Rendered2dShape::render_texture_trivial(
    //         tex,
    //         modulate(crate::color::nx::ColorModulated::FacelineMakeup, &char),
    //         None,
    //         &mut headless_renderer,
    //         &target_texture.view,
    //         &mut encoder,
    //     );

    //     let image = headless_renderer.output_texture(&target_texture, encoder);

    //     println!("Done!");
    //     image.save(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/test_output/faceline_beard.png"
    //     ))?;

    //     Ok(())
    // }

    // use crate::draw::mask::MaskFaceParts;
    // use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    // use crate::res::tex::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT};
    // use binrw::BinRead;
    // use glam::uvec2;
    // use image_compare::Algorithm;

    // use super::*;
    // use std::{error::Error, fs::File, io::BufReader};

    // type R = Result<(), Box<dyn Error>>;

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
