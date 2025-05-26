use crate::state::State;
use glam::{UVec2, Vec3, uvec2, vec3, vec4};
use image::DynamicImage;
use vee_wgpu::texture::TextureBundle;
use vee_wgpu::{Model3d, ProgramState};
use vfl::color::nx::ModulationIntent;
use vfl::draw::mesh_building::mask_texture_meshes;
use vfl::draw::render_3d::GenericModel3d;
use vfl::draw::{DrawableTexture, Vertex};
use vfl::res::shape::nx::ShapeData;
use vfl::res::tex::nx::{ResourceTexture, TextureElement};
use vfl::{
    color::nx::{ColorModulated, modulate},
    res::shape::nx::{GenericResourceShape, Shape},
};
use wgpu::{CommandEncoder, TextureView};

pub(crate) fn draw_noseline(
    st: &mut State,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let res_texture = &st.resources.texture_header;
    let file_texture = &st.resources.texture_data;

    let noseline_num = usize::from(st.char_info.nose_type);

    let tex: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = res_texture.noseline[noseline_num]
        .get_image(file_texture)
        .unwrap()
        .unwrap();
    let tex = DynamicImage::ImageRgba8(tex);

    st.draw_texture(
        DrawableTexture {
            rendered_texture: tex,
            modulation: modulate(ColorModulated::NoseLineShape, &st.char_info),
            opaque: None,
        },
        texture_view,
        encoder,
    );
}

pub(crate) fn draw_mask(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture = &st.resources.texture_header;
    let file_texture = &st.resources.texture_data;

    let shapes = mask_texture_meshes(&st.char_info, res_texture, file_texture);

    for mut shape in shapes {
        st.draw_model_2d(&mut shape, texture_view, encoder);
    }
}

// Looks up a TextureElement, and returns the texture with any modulation that needs to be done.
// Returns an `Option<T>` because the texture could not exist (e.g. CharInfo w/o Beard)
fn load_faceline_texture(
    st: &mut State,
    texture_element: TextureElement,
    modulated: ColorModulated,
) -> Option<(DynamicImage, ModulationIntent)> {
    texture_element
        .get_image(&st.resources.texture_data)
        .unwrap()
        .map(|tex| {
            (
                DynamicImage::ImageRgba8(tex),
                modulate(modulated, &st.char_info),
            )
        })
}

// Load faceline textures in order [wrinkle, makeup, beard], and removes any that don't exist
fn get_faceline_textures(
    st: &mut State,
    res_texture: &ResourceTexture,
) -> Vec<(DynamicImage, ModulationIntent)> {
    vec![
        load_faceline_texture(
            st,
            res_texture.wrinkle[st.char_info.faceline_wrinkle as usize],
            ColorModulated::FacelineWrinkle,
        ),
        load_faceline_texture(
            st,
            res_texture.makeup[st.char_info.faceline_make as usize],
            ColorModulated::FacelineMakeup,
        ),
        {
            // We need to do a "smarter" check here.
            if st.char_info.beard_type >= 4 {
                load_faceline_texture(st, res_texture.beard[1], ColorModulated::BeardTexture)
            } else {
                None
            }
        },
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn draw_faceline(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture = st.resources.texture_header;

    let textures = get_faceline_textures(st, &res_texture);

    for (i, (rendered_texture, modulation)) in textures.into_iter().enumerate() {
        // Check if we are the first to be rendered out, then add an opaque background.
        // We don't want an opaque redraw happening over our other faceline textures.
        let opaque = (i == 0).then_some(
            vfl::color::nx::linear::FACELINE_COLOR[usize::from(st.char_info.faceline_color)],
        );

        st.draw_texture(
            DrawableTexture {
                rendered_texture,
                modulation,
                opaque,
            },
            texture_view,
            encoder,
        );
    }
}

fn draw_glasses(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture = st.resources.texture_header;

    let texture = load_faceline_texture(
        st,
        res_texture.glass[st.char_info.glass_type as usize],
        ColorModulated::Glass,
    );

    let Some((rendered_texture, modulation)) = texture else {
        return;
    };

    st.draw_texture(
        DrawableTexture {
            rendered_texture,
            modulation,
            opaque: None,
        },
        texture_view,
        encoder,
    );
}

pub(crate) fn load_shape(
    shape_kind: Shape,
    shape_index: u8,
    shape_color: u8,
    st: &mut State,
    encoder: &mut CommandEncoder,
) -> Option<Model3d> {
    let res_shape = &st.resources.shape_header;

    let GenericResourceShape::FaceLineTransform(faceline_transform) = res_shape.index_by_shape(
        Shape::FaceLineTransform,
        usize::from(st.char_info.faceline_type),
    )?
    else {
        panic!()
    };

    let GenericResourceShape::Element(mut shape_element) =
        res_shape.index_by_shape(shape_kind, usize::from(shape_index))?
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
        Shape::Nose | Shape::NoseLine => {
            let nose = Vec3::from_array(faceline_transform.nose_translate);
            let nose_y = f32::from(st.char_info.nose_y);

            // FFLiCharModelCreator.cpp :638
            vec3(nose.x, nose.y + (nose_y - 8.0) * -1.5, nose.z)
        }
        Shape::Glasses => {
            let nose = Vec3::from_array(faceline_transform.nose_translate);
            let glass_y = f32::from(st.char_info.glass_y);

            // FFLiCharModelCreator.cpp :691
            vec3(nose.x, nose.y + (glass_y - 11.0) * -1.5 + 5.0, nose.z + 2.0)
        }
        _ => Vec3::ZERO,
    };

    let scale = match shape_kind {
        Shape::Glasses => Vec3::splat(0.15 * f32::from(st.char_info.glass_scale) + 0.4), // RFL_Model.c :784
        Shape::Nose => Vec3::splat(0.175 * f32::from(st.char_info.nose_scale) + 0.4), // RFL_Model.c :705
        _ => Vec3::ONE,
    };

    // Closure to reduce boilerplate for writing out textures.
    let mut draw_tex = |func: fn(&mut State, &TextureView, &mut CommandEncoder), size: UVec2| {
        let texture = TextureBundle::create_texture(
            &st.device,
            &size,
            &format!("projected texture {func:?}"),
        );

        func(st, &texture.view, encoder);

        Some(texture)
    };

    // Draw out any textures we need.
    let projected_texture = match shape_kind {
        Shape::NoseLine => draw_tex(draw_noseline, uvec2(256, 256)),
        Shape::Mask => draw_tex(draw_mask, uvec2(512, 512)),
        Shape::FaceLine => draw_tex(draw_faceline, uvec2(512, 512)),
        Shape::Glasses => draw_tex(draw_glasses, uvec2(512, 512)),
        _ => None,
    };

    let file_shape = &st.resources.shape_data[..];

    Some(mesh_to_model(
        shape_element.shape_data(file_shape).unwrap(),
        shape_kind,
        usize::from(shape_color),
        position,
        scale,
        projected_texture,
    ))
}

// I'm in a fucking horror of my own design
pub(crate) fn mesh_to_model(
    d: ShapeData,
    shape: Shape,
    color: usize,
    position: Vec3,
    scale: Vec3,
    projected_texture: Option<TextureBundle>,
) -> Model3d {
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

    GenericModel3d {
        vertices,
        indices,
        color: match shape {
            Shape::HairNormal => vfl::color::nx::linear::COMMON_COLOR[color].into(),
            Shape::FaceLine | Shape::ForeheadNormal | Shape::Nose => {
                vfl::color::nx::linear::FACELINE_COLOR[color].into()
            }
            _ => vec4(0.0, 0.0, 0.0, 0.0),
        },
        texture: projected_texture,
        position,
        scale,
    }
}
