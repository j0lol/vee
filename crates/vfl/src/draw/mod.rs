use crate::color::nx::ModulationIntent;
use faceline::trivial_quad;
use image::DynamicImage;
use nalgebra as na;
use render_2d::Model2d;
use render_3d::GenericModel3d;

#[cfg(feature = "draw")]
pub mod render_2d;
#[cfg(feature = "draw")]
pub mod render_3d;
#[cfg(feature = "draw")]
pub mod wgpu_render;

pub mod faceline;
pub mod mask;

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

pub trait AbstractRenderer {
    type DrawState;

    fn draw_texture<Rt, Cb>(
        &mut self,
        texture: DrawableTexture,
        render_target: Rt,
        command_buffer: Cb,
    ) {
        let (vertices, indices) = trivial_quad();

        let mvp_matrix = {
            let scale = na::Vector3::<f32>::new(-2.0, -2.0, 1.0); // IDK

            na::Matrix4::new_nonuniform_scaling(&scale)
        };

        let rendered_2d_shape = Model2d {
            vertices,
            indices,
            tex: texture.rendered_texture,
            mvp_matrix,
            modulation: texture.modulation,
            opaque: texture.opaque,
            label: None,
        };

        self.draw_model_2d(rendered_2d_shape, render_target, command_buffer);
    }
    fn draw_model_2d<Rt, Cb>(&mut self, mesh: Model2d, render_target: Rt, command_buffer: Cb);
    fn draw_model_3d<Tex, Rt, Cb>(
        &mut self,
        mesh: GenericModel3d<Tex>,
        render_target: Rt,
        command_buffer: Cb,
    );
}
