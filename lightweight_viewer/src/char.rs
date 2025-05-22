use crate::render::shape_data_to_render_3d_shape;
use vfl::draw::render_3d::Rendered3dShape;
use glam::{uvec2, Vec3};
use image::DynamicImage;
use vfl::{
    color::nx::{modulate, ColorModulated},
    draw::{
        render_2d::Rendered2dShape,
        wgpu_render::{
            texture
            , RenderContext,
        },
    },
    res::shape::nx::{
        GenericResourceShape, Shape,
    },
};
use wgpu::{
    CommandEncoder,
    TextureView,
};
use crate::State;

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

fn draw_faceline(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture = &st.resources.texture_header;

    let tex = res_texture.makeup[st.char_info.faceline_make as usize]
        .get_image(&st.resources.texture_data)
        .unwrap()
        .unwrap();
    let tex = image::DynamicImage::ImageRgba8(tex);

    Rendered2dShape::render_texture_trivial(
        tex,
        modulate(
            vfl::color::nx::ColorModulated::FacelineMakeup,
            &st.char_info,
        ),
        Some(vfl::color::nx::srgb::FACELINE_COLOR[usize::from(st.char_info.faceline_color)]),
        st,
        texture_view,
        encoder,
    );
}

pub fn draw_char(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let shapes = get_char_shapes(st, encoder);

    for shape in shapes {
        shape.render(st, texture_view, encoder);
    }
}

fn load_shape(
    shape_kind: Shape,
    shape_index: u8,
    shape_color: u8,
    st: &mut State,
    encoder: &mut CommandEncoder,
) -> Option<Rendered3dShape> {
    let res_shape = &st.resources.shape_header;

    // println!("Loading shp {shape_kind:?}[{shape_index:?}] col#{shape_color:?}");

    let GenericResourceShape::FaceLineTransform(faceline_transform) = res_shape
        .index_by_shape(
            Shape::FaceLineTransform,
            usize::from(st.char_info.faceline_type),
        )
        .unwrap()
    else {
        panic!()
    };

    let GenericResourceShape::Element(mut shape_element) = res_shape
        .index_by_shape(shape_kind, usize::from(shape_index))
        .unwrap()
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
