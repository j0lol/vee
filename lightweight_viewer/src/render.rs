use crate::OVERLAY_REBECCA_PURPLE;
use crate::wgpu_color_to_vec4;
use glam::{Vec3, vec4};
use vfl::draw::render_3d::Rendered3dShape;
use vfl::draw::wgpu_render::texture;
use vfl::{
    color::nx::linear::FACELINE_COLOR,
    draw::wgpu_render::Vertex,
    res::shape::nx::{Shape, ShapeData},
};

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
            _ => vec4(0.0, 0.0, 0.0, 0.0),
            // _ => wgpu_color_to_vec4(OVERLAY_REBECCA_PURPLE),
        },
        texture: projected_texture,
        position,
    }
}
