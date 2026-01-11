//! Representing models, and other related structures.

use crate::building::trivial_quad;
use glam::{Mat4, Vec3, Vec4, vec3};
use image::DynamicImage;
use vee_resources::color::nx::ModulationIntent;
use vee_resources::packing::Float16;

type Color = [f32; 4];

/// A point in 3D space.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [Float16; 3],
    pub _pad: u16, // We need to stay within the bounds of 32 bit chunks
    pub tex_coords: [Float16; 2],
    pub normal: [f32; 3],
}

/// A texture prepared to be drawn onto a quad.
pub struct DrawableTexture {
    pub rendered_texture: DynamicImage,
    pub modulation: ModulationIntent,
    pub opaque: Option<[f32; 4]>,
}

impl DrawableTexture {
    #[must_use]
    pub fn model_2d(self) -> Model2d {
        let (vertices, indices) = trivial_quad();

        let mvp_matrix = Mat4::from_scale(vec3(-2.0, -2.0, 1.0));

        Model2d {
            vertices,
            indices,
            tex: self.rendered_texture,
            mvp_matrix,
            modulation: self.modulation,
            opaque: self.opaque,
            label: None,
        }
    }
}

/// Flat 2D model (always a quad.) Holds all data needed for rendering.
/// The texture held within this model is a set of bytes, not a handle
/// to an image in the GPU's Vram.
#[derive(Debug)]
pub struct Model2d {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tex: DynamicImage,
    pub mvp_matrix: Mat4,
    pub modulation: ModulationIntent,
    pub opaque: Option<Color>,
    pub label: Option<String>,
}

/// 3D model. Holds a handle to an image in Vram, or whatever you need instead of that.
#[derive(Default, Debug)]
pub struct GenericModel3d<Tex> {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub color: Vec4,
    pub texture: Option<Tex>,
    pub position: Vec3,
    pub scale: Vec3,
}
