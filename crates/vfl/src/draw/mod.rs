use crate::color::nx::ModulationIntent;
use image::DynamicImage;
use mesh_building::trivial_quad;
use nalgebra as na;
use render_2d::Model2d;

#[cfg(feature = "draw")]
pub mod mesh_building;
pub mod positioning;
#[cfg(feature = "draw")]
pub mod render_2d;
#[cfg(feature = "draw")]
pub mod render_3d;

pub const TEX_SCALE_X: f32 = 0.889_614_64;
pub const TEX_SCALE_Y: f32 = 0.927_667_5;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct DrawableTexture {
    pub rendered_texture: DynamicImage,
    pub modulation: ModulationIntent,
    pub opaque: Option<[f32; 4]>,
}

impl DrawableTexture {
    #[must_use]
    pub fn model_2d(self) -> Model2d {
        let (vertices, indices) = trivial_quad();

        let mvp_matrix = {
            let scale = na::Vector3::<f32>::new(-2.0, -2.0, 1.0); // IDK

            na::Matrix4::new_nonuniform_scaling(&scale)
        };

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
