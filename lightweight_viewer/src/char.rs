use crate::State;
use crate::render::shape_data_to_render_3d_shape;
use glam::{Vec3, uvec2};
use image::{DynamicImage, RgbaImage};
use vfl::color::nx::ModulationIntent;
use vfl::draw::render_3d::Rendered3dShape;
use vfl::res::tex::nx::{ResourceTexture, TextureElement};
use vfl::{
    color::nx::{ColorModulated, modulate},
    draw::{
        render_2d::Rendered2dShape,
        wgpu_render::{RenderContext, texture},
    },
    res::shape::nx::{GenericResourceShape, Shape},
};
use wgpu::{CommandEncoder, TextureView};

pub fn draw_noseline(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture = &st.resources.texture_header;
    let file_texture = &st.resources.texture_data;

    let noseline_num = usize::from(st.char_info.nose_type);

    let tex: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = res_texture.noseline[noseline_num]
        .get_image(file_texture)
        .unwrap()
        .unwrap();
    let tex = DynamicImage::ImageRgba8(tex);

    Rendered2dShape::render_texture_trivial(
        tex,
        modulate(ColorModulated::NoseLineShape, &st.char_info),
        None,
        st,
        texture_view,
        encoder,
    );
}

pub fn draw_mask(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_shape = &st.resources.shape_header;
    let res_texture = &st.resources.texture_header;
    let file_texture = &st.resources.texture_data;

    let render_context =
        RenderContext::new(&st.char_info.clone(), res_texture, res_shape, file_texture).unwrap();

    for shape in render_context.shape {
        shape.render(st, texture_view, encoder);
    }
}

// Looks up a TextureElement, and returns the texture with any modulation that needs to be done.
// Returns an Option<T> because the texture could not exist (e.g. CharInfo w/o Beard)
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
            // We need to do a smarter check here.
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

    // let makeup_tex = res_texture.makeup[st.char_info.faceline_make as usize]
    //     .get_image(&st.resources.texture_data)
    //     .unwrap();
    // let Some(makeup_tex) = makeup_tex else {
    //     return;
    // };
    // let makeup_tex = image::DynamicImage::ImageRgba8(tex);

    let textures = get_faceline_textures(st, &res_texture);

    for (i, (rendered_texture, modulation)) in textures.iter().enumerate() {
        // Check if we are the first to be rendered out, then add an opaque background.
        // We don't want an opaque redraw happening over our other faceline textures.
        let opaque = (i == 0).then_some(
            vfl::color::nx::srgb::FACELINE_COLOR[usize::from(st.char_info.faceline_color)],
        );

        Rendered2dShape::render_texture_trivial(
            rendered_texture.to_owned(),
            modulation.to_owned(),
            opaque,
            st,
            texture_view,
            encoder,
        );
    }
}

fn draw_glasses(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    println!("bleh!");
    let res_texture = st.resources.texture_header;

    let Some((rendered_texture, modulation)) = load_faceline_texture(
        st,
        res_texture.glass[st.char_info.glass_type as usize],
        ColorModulated::Glass,
    ) else {
        return;
    };

    Rendered2dShape::render_texture_trivial(
        rendered_texture,
        modulation,
        None,
        st,
        texture_view,
        encoder,
    );
}

pub fn draw_char(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let shapes = get_char_shapes(st, encoder);

    for mut shape in shapes {
        shape.render(st, texture_view, encoder);
    }
}

pub(crate) fn load_shape(
    shape_kind: Shape,
    shape_index: u8,
    shape_color: u8,
    st: &mut State,
    encoder: &mut CommandEncoder,
) -> Option<Rendered3dShape> {
    let res_shape = &st.resources.shape_header;

    // println!("Loading shp {shape_kind:?}[{shape_index:?}] col#{shape_color:?}");

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
    let position = match shape_kind {
        Shape::Nose | Shape::NoseLine | Shape::Glasses => {
            Vec3::from_array(faceline_transform.nose_translate)
        }
        _ => Vec3::ZERO,
    };

    // Draw out any textures we need.
    let projected_texture = match shape_kind {
        Shape::NoseLine => {
            // let tex = res_texture.noseline[1]
            //     .get_image(&mut BufReader::new(
            //         File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
            //     ))
            //     .unwrap()
            //     .unwrap();

            // let tex = DynamicImage::ImageRgba8(tex);

            // let noseline_texture =
            //     texture::Texture::from_image(&st.device, &st.queue, &tex, None).unwrap();
            // // let noseline_texture = crate::texture::Texture::create_texture(
            // //     &st.device,
            // //     &PhysicalSize::<u32>::new(128, 128),
            // //     "noselinetex",
            // // );

            let noseline_texture =
                texture::Texture::create_texture(&st.device, &uvec2(256, 256), "noselinetex");

            draw_noseline(st, &noseline_texture.view, encoder);

            Some(noseline_texture)
        }
        Shape::Mask => {
            let mask_texture =
                texture::Texture::create_texture(&st.device, &uvec2(512, 512), "masktex");

            draw_mask(st, &mask_texture.view, encoder);

            Some(mask_texture)
        }
        Shape::FaceLine => {
            let faceline_texture =
                texture::Texture::create_texture(&st.device, &uvec2(512, 512), "facelinetex");

            draw_faceline(st, &faceline_texture.view, encoder);

            Some(faceline_texture)
        }
        Shape::Glasses => {
            let glasses_texture =
                texture::Texture::create_texture(&st.device, &uvec2(512, 512), "glassestex");

            draw_glasses(st, &glasses_texture.view, encoder);

            Some(glasses_texture)
        }
        _ => None,
    };

    let file_shape = &st.resources.shape_data[..];

    Some(shape_data_to_render_3d_shape(
        shape_element.shape_data(file_shape).unwrap(),
        shape_kind,
        usize::from(shape_color),
        position,
        projected_texture,
    ))
}

fn get_char_shapes(st: &mut State, encoder: &mut CommandEncoder) -> Vec<Rendered3dShape> {
    // Order DOES matter for back-to-front sorting. It's not a perfect science, though.
    vec![
        load_shape(
            Shape::FaceLine,
            st.char_info.faceline_type,
            st.char_info.faceline_color,
            st,
            encoder,
        ),
        load_shape(
            Shape::HairNormal,
            st.char_info.hair_type,
            st.char_info.hair_color,
            st,
            encoder,
        ),
        load_shape(
            Shape::Nose,
            st.char_info.nose_type,
            st.char_info.faceline_color,
            st,
            encoder,
        ),
        load_shape(Shape::NoseLine, st.char_info.nose_type, 0, st, encoder),
        {
            if st.char_info.glass_type != 0 {
                load_shape(Shape::Glasses, 0, st.char_info.glass_color, st, encoder)
            } else {
                None
            }
        },
        load_shape(Shape::Mask, st.char_info.faceline_type, 0, st, encoder),
    ]
    .into_iter()
    .flatten()
    .collect()
}
