use crate::draw::texture::{draw_cap, draw_faceline, draw_glasses, draw_mask, draw_noseline};
use crate::draw::ModelOpt;
use crate::texture::TextureBundle;
use crate::{Model3d, ProgramState};
use glam::{uvec2, vec3, vec4, UVec2, Vec3};
use vee_models::model::{GenericModel3d, Vertex};
use vee_parse::NxCharInfo;
use vee_resources::color;
use vee_resources::packing::Float16;
use vee_resources::shape::{GenericResourceShape, Shape, ShapeMesh};
use wgpu::{CommandEncoder, TextureView};

pub(crate) fn load_shape(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    shape_kind: Shape,
    shape_index: u8,
    shape_color: u8,
    encoder: &mut CommandEncoder,
) -> Option<Model3d> {
    let shape_header = &st.shape_header();

    let GenericResourceShape::FaceLineTransform(faceline_transform) = shape_header.index_by_shape(
        Shape::FaceLineTransform,
        usize::from(char_info.faceline_type),
    )?
    else {
        panic!()
    };

    let GenericResourceShape::Element(mut shape_element) =
        shape_header.index_by_shape(shape_kind, usize::from(shape_index))?
    else {
        panic!()
    };

    // For some reason there are just empty gaps in the shape data.
    // To validate this you just have to check that the size is 0? Who knows why.
    if shape_element.common.size == 0 {
        return None;
    }

    // Some meshes need positioning.
    let position = match shape_kind {
        Shape::HairNormal | Shape::ForeheadNormal => {
            // FFLiCharModelCreator.cpp :638
            Vec3::from_array(faceline_transform.hair_translate)
        }
        Shape::Beard => Vec3::from_array(faceline_transform.beard_translate),
        Shape::Nose | Shape::NoseLine => {
            let nose = Vec3::from_array(faceline_transform.nose_translate);
            let nose_y = f32::from(char_info.nose_y);

            // FFLiCharModelCreator.cpp :638
            vec3(nose.x, nose.y + (nose_y - 8.0) * -1.5, nose.z)
        }
        Shape::Glasses => {
            let nose = Vec3::from_array(faceline_transform.nose_translate);
            let glass_y = f32::from(char_info.glass_y);

            // FFLiCharModelCreator.cpp fn:InitShapes
            vec3(nose.x, nose.y + (glass_y - 11.0) * -1.5 + 5.0, nose.z + 2.0)
        }
        _ => Vec3::ZERO,
    };

    let scale = match shape_kind {
        // RFL_Model.c :784
        Shape::Glasses => Vec3::splat(0.15 * f32::from(char_info.glass_scale) + 0.4),
        Shape::HairCap | Shape::ForeheadCap | Shape::HatCap => {
            dbg!(shape_kind);
            faceline_transform.hair_translate.into()
        }
        // RFL_Model.c :705
        Shape::Nose | Shape::NoseLine => Vec3::splat(0.175 * f32::from(char_info.nose_scale) + 0.4),
        Shape::HairNormal => {
            if char_info.hair_flip != 0 {
                vec3(-1.0, 1.0, 1.0)
            } else {
                Vec3::ONE
            }
        }
        _ => Vec3::ONE,
    };

    // Closure to reduce boilerplate for writing out textures.
    let mut draw_tex =
        |func: fn(&mut dyn ProgramState, &NxCharInfo, &TextureView, &mut CommandEncoder),
         size: UVec2| {
            let texture = TextureBundle::create_texture(
                &st.device(),
                &size,
                &format!("projected texture {func:?}"),
            );

            func(st, &char_info, &texture.view, encoder);

            Some(texture)
        };

    // Draw out any textures we need.
    // This stupid match requires that each fn pointer uses `dyn` instead of `impl`, meaning there
    // is a runtime cost associated here. Blegh. FIXME.
    let projected_texture = match shape_kind {
        Shape::NoseLine => draw_tex(draw_noseline, uvec2(256, 256)),
        Shape::Mask => draw_tex(draw_mask, uvec2(512, 512)),
        Shape::FaceLine => draw_tex(draw_faceline, uvec2(512, 512)),
        Shape::Glasses => draw_tex(draw_glasses, uvec2(512, 512)),
        Shape::HatCap | Shape::HairCap | Shape::ForeheadCap => draw_tex(draw_cap, uvec2(512, 512)),
        _ => None,
    };

    let file_shape = &st.shape_data()[..];

    Some(mesh_to_model(
        shape_element.mesh(file_shape).unwrap(),
        shape_kind,
        usize::from(shape_color),
        position,
        scale,
        projected_texture,
    ))
}

// I'm in a fucking horror of my own design

/// Converts a `ShapeMesh` into a `Model3d`.
pub(crate) fn mesh_to_model(
    d: ShapeMesh,
    shape: Shape,
    color: usize,
    position: Vec3,
    scale: Vec3,
    projected_texture: Option<TextureBundle>,
) -> Model3d {
    let mut vertices: Vec<Vertex> = vec![];
    let tex_coords = d
        .uvs // Go on, return NULL. See if I care.
        .unwrap_or(vec![
            [f32::NAN, f32::NAN].map(Float16::from_f32);
            d.positions.len()
        ]);
    let normals = d.normals.unwrap();

    for i in 0..d.positions.len() {
        let [px, py, pz, _] = d.positions[i];
        vertices.push(Vertex {
            position: [px, py, pz],
            _pad: 0,
            tex_coords: tex_coords[i],
            normal: normals[i].unpack(),
        })
    }

    let indices = d.indices.iter().map(|x| u32::from(*x)).collect();

    GenericModel3d {
        vertices,
        indices,
        color: match shape {
            Shape::HairNormal | Shape::Beard => color::nx::linear::COMMON_COLOR[color].into(),
            Shape::FaceLine | Shape::ForeheadNormal | Shape::Nose => {
                color::nx::linear::FACELINE_COLOR[color].into()
            }
            _ => vec4(0.0, 0.0, 0.0, 0.0),
        },
        texture: projected_texture,
        position,
        scale,
    }
}

pub(super) fn face_line(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::FaceLine,
        char_info.faceline_type,
        char_info.faceline_color.0,
        encoder,
    )
}

pub(super) fn forehead(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::ForeheadNormal,
        char_info.hair_type,
        char_info.faceline_color.0,
        encoder,
    )
}

pub(super) fn hair(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::HairNormal,
        char_info.hair_type,
        char_info.hair_color.0,
        encoder,
    )
}

pub(super) fn mask(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::Mask,
        char_info.faceline_type,
        0,
        encoder,
    )
}

pub(super) fn nose(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::Nose,
        char_info.nose_type,
        char_info.faceline_color.0,
        encoder,
    )
}

pub(super) fn nose_line(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::NoseLine,
        char_info.nose_type,
        0,
        encoder,
    )
}

pub(super) fn glasses(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    if char_info.glass_type != 0 {
        load_shape(
            st,
            char_info,
            Shape::Glasses,
            0,
            char_info.glass_color.0,
            encoder,
        )
    } else {
        None
    }
}

pub(super) fn beard(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    if char_info.beard_type < 4 && char_info.beard_type != 0 {
        load_shape(
            st,
            char_info,
            Shape::Beard,
            char_info.beard_type,
            char_info.beard_color.0,
            encoder,
        )
    } else {
        None
    }
}

pub(super) fn cap(
    st: &mut impl ProgramState,
    char_info: &NxCharInfo,
    encoder: &mut CommandEncoder,
) -> ModelOpt {
    load_shape(
        st,
        char_info,
        Shape::HatCap,
        char_info.hair_type,
        char_info.favorite_color.0,
        encoder,
    )
}
