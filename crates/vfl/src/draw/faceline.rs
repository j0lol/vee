use super::Vertex;

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
}
