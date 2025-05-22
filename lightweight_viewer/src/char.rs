use crate::{
    render::{RenderThatContext, Rendered3dShape, shape_data_to_render_3d_shape},
    wgpu_color_to_vec4,
};
use std::{fs::File, io::BufReader, sync::Arc};

use glam::{Vec2, Vec3, Vec4, vec2, vec3};
use image::DynamicImage;
use vfl::{
    charinfo::nx::{BinRead, NxCharInfo},
    color::nx::{ColorModulated, linear::FACELINE_COLOR, modulate},
    draw::{
        mask::ImageOrigin,
        wgpu_render::{
            RenderContext, RenderShape as Rendered2dShape, SHADER, TextureTransformUniform, Vertex,
            cast_slice, model_view_matrix, quad, render_context_wgpu,
        },
    },
    res::{
        shape::nx::{
            GenericResourceShape, ResourceShape, SHAPE_MID_DAT, Shape, ShapeData, ShapeElement,
        },
        tex::nx::{ResourceTexture, ResourceTextureFormat, TEXTURE_MID_SRGB_DAT, TextureElement},
    },
};
use wgpu::{
    Backends, BlendState, CommandEncoder, TexelCopyTextureInfo, Texture, TextureFormat,
    TextureView, include_wgsl, util::DeviceExt,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{OVERLAY_REBECCA_PURPLE, State, texture};

pub fn draw_noseline(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let res_texture: ResourceTexture = ResourceTexture::read(&mut st.texture()).unwrap();
    let noseline_num = usize::from(st.char_info.nose_type);

    let tex: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = res_texture.noseline[1]
        .get_image(&mut st.texture())
        .unwrap()
        .unwrap();
    let tex = DynamicImage::ImageRgba8(tex);

    let (vertices, indices, mvp_matrix) =
        quad(0.0, 0.0, 128.0, 128.0, 0.0, ImageOrigin::Center, 256.0);

    //
    let render_shape: Rendered2dShape = Rendered2dShape {
        vertices,
        indices,
        tex,
        mvp_matrix,
        texture_type: ResourceTextureFormat::try_from(
            // res_texture.noseline[noseline_num].texture.format,
            5,
        )
        .unwrap(),
        channel_replacements: modulate(ColorModulated::NoseLine, &st.char_info),
    };

    RenderContext::render_2d_shape(render_shape, st, texture_view, encoder);
}

pub fn draw_mask(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let render_context =
        RenderContext::new(&st.char_info.clone(), (&mut st.shape(), &mut st.texture())).unwrap();

    render_context.render(st, texture_view, encoder);
}

fn draw_faceline(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    // let render_context =
    //     RenderContext::new(&st.char_info.clone(), (&mut st.shape(), &mut st.texture())).unwrap();

    let wrinkle_num = st.char_info.faceline_make;
    let makeup_num = st.char_info.faceline_make;
    let beard_num = st.char_info.beard_type;

    let res_texture: ResourceTexture = ResourceTexture::read(&mut st.texture()).unwrap();

    // let makeup = res_texture.makeup[1]
    //     .get_image(&mut st.texture())
    //     .unwrap()
    //     .unwrap();

    // RenderContext::render_texture(
    //     DynamicImage::ImageRgba8(makeup),
    //     vec2(256.0, 512.0),
    //     st,
    //     texture_view,
    //     encoder,
    // );

    let mut func = |tex_element: TextureElement, modulation: ColorModulated| {
        let beard = tex_element.get_image(&mut st.texture()).unwrap().unwrap();

        let (vertices, indices, mvp_matrix) =
            quad(100.0, 20.0, 20.0, 20.0, 0.0, ImageOrigin::Center, 256.0);

        let mvp_mtx = {
            let mv_mtx = model_view_matrix(
                vec3(0.0 + 256.0, 0.0 + 256.0, 0.0).into(),
                vec3(512.0, 256.0, 1.0).into(),
                0.0,
            );

            let p_mtx = nalgebra::Matrix4::new_orthographic(0.0, 512.0, 0.0, 512.0, -200.0, 200.0);
            let mut mvp_mtx: nalgebra::Matrix4<f32> = p_mtx * mv_mtx;

            *mvp_mtx.get_mut((1, 1)).unwrap() *= -1.0;
            mvp_mtx
        };

        // shape.mvp_matrix = mvp_mtx;

        let render_2d_shape = Rendered2dShape {
            vertices,
            indices,
            tex: DynamicImage::ImageRgba8(beard),
            mvp_matrix: mvp_mtx,
            texture_type: ResourceTextureFormat::try_from(2).unwrap(),
            channel_replacements: modulate(modulation, &st.char_info),
        };

        RenderContext::render_2d_shape(render_2d_shape, st, texture_view, encoder);
    };

    func(res_texture.beard[1], ColorModulated::Beard);

    let render_context =
        RenderContext::new(&st.char_info.clone(), (&mut st.shape(), &mut st.texture())).unwrap();

    // for shape in render_context.shape {
    //     // dbg!(&shape.vertices);
    //     let mut shape = shape;
    //     shape.tex = DynamicImage::ImageRgba8(
    //         res_texture.beard[1]
    //             .get_image(&mut st.texture())
    //             .unwrap()
    //             .unwrap(),
    //     );
    //     shape.texture_type = ResourceTextureFormat::try_from(2).unwrap();

    //     let mvp_mtx = {
    //         let mv_mtx = model_view_matrix(
    //             vec3(0.0 + 256.0, 0.0 + 256.0, 0.0).into(),
    //             vec3(512.0, 256.0, 1.0).into(),
    //             0.0,
    //         );

    //         let p_mtx = nalgebra::Matrix4::new_orthographic(0.0, 512.0, 0.0, 512.0, -200.0, 200.0);
    //         let mut mvp_mtx: nalgebra::Matrix4<f32> = p_mtx * mv_mtx;

    //         *mvp_mtx.get_mut((1, 1)).unwrap() *= -1.0;
    //         mvp_mtx
    //     };
    //     shape.mvp_matrix = mvp_mtx;

    //     RenderContext::render_2d_shape(shape, st, texture_view, encoder);
    // }
}

pub fn draw_char(st: &mut State, texture_view: &TextureView, encoder: &mut CommandEncoder) {
    let shapes = get_char_shapes(st, encoder);

    for shape in shapes {
        RenderContext::render_3d_shape(shape, st, texture_view, encoder);
    }
    // let res_shape: ResourceShape = ResourceShape::read(&mut st.shape()).unwrap();
    // let GenericResourceShape::Element(mut shape_faceline) = res_shape
    //     .fetch_shape(
    //         vfl::res::shape::nx::Shape::FaceLine,
    //         usize::from(st.char_info.faceline_type),
    //     )
    //     .unwrap()
    // else {
    //     panic!()
    // };
    // let GenericResourceShape::Element(mut shape_hair) = res_shape
    //     .fetch_shape(
    //         vfl::res::shape::nx::Shape::HairNormal,
    //         usize::from(st.char_info.hair_type),
    //     )
    //     .unwrap()
    // else {
    //     panic!()
    // };

    // let GenericResourceShape::Element(mut shape_mask) = res_shape
    //     .fetch_shape(
    //         vfl::res::shape::nx::Shape::Mask,
    //         usize::from(st.char_info.faceline_type),
    //     )
    //     .unwrap()
    // else {
    //     panic!()
    // };

    // let GenericResourceShape::Element(shape_nose) = res_shape
    //     .fetch_shape(
    //         vfl::res::shape::nx::Shape::Nose,
    //         usize::from(st.char_info.nose_type),
    //     )
    //     .unwrap()
    // else {
    //     panic!()
    // };

    // let GenericResourceShape::Element(shape_noseline) = res_shape
    //     .fetch_shape(
    //         vfl::res::shape::nx::Shape::NoseLine,
    //         usize::from(st.char_info.nose_type),
    //     )
    //     .unwrap()
    // else {
    //     panic!()
    // };

    // // Sometimes noses are texture-only.
    // let shape_nose = (shape_nose.common.size != 0).then_some(shape_nose);
    // let shape_noseline = (shape_noseline.common.size != 0).then_some(shape_noseline);

    // // Glass shape needs MV matrix applied
    // let shape_glass = (st.char_info.glass_type != 0)
    //     .then(|| res_shape.fetch_shape(vfl::res::shape::nx::Shape::Glasses, 0))
    //     .flatten();

    // let mask_texture = crate::texture::Texture::create_texture(
    //     &st.device,
    //     &PhysicalSize::<u32>::new(512, 512),
    //     "masktex",
    // );
    // let faceline_texture = crate::texture::Texture::create_texture(
    //     &st.device,
    //     &PhysicalSize::<u32>::new(512, 512),
    //     "facelinetex",
    // );

    // draw_mask(st, &mask_texture.view, encoder);
    // draw_faceline(st, &faceline_texture.view, encoder);

    // let mut render = move |mut element: ShapeElement,
    //                        shape_class: Shape,
    //                        color: u8,
    //                        texture: Option<texture::Texture>| {
    //     let shape_data = element.shape_data(&mut st.shape()).unwrap();

    //     let shape =
    //         shape_data_to_render_3d_shape(shape_data, shape_class, usize::from(color), texture);

    //     shape
    // };

    // // RenderContext::render_3d_shape(
    // //     render(
    // //         shape_faceline,
    // //         Shape::FaceLine,
    // //         st.char_info.faceline_color,
    // //         None,
    // //     ),
    // //     st,
    // //     texture_view,
    // //     encoder,
    // // );

    // // render(
    // //     shape_faceline,
    // //     Shape::FaceLine,
    // //     st.char_info.faceline_color,
    // //     None,
    // // );
    // RenderContext::render_3d_shape(
    //     shape_data_to_render_3d_shape(
    //         shape_faceline.shape_data(&mut st.shape()).unwrap(),
    //         Shape::FaceLine,
    //         usize::from(st.char_info.faceline_color),
    //         None,
    //     ),
    //     st,
    //     texture_view,
    //     encoder,
    // );
    // RenderContext::render_3d_shape(
    //     shape_data_to_render_3d_shape(
    //         shape_hair.shape_data(&mut st.shape()).unwrap(),
    //         Shape::HairNormal,
    //         usize::from(st.char_info.hair_color),
    //         None,
    //     ),
    //     st,
    //     texture_view,
    //     encoder,
    // );

    // if let Some(mut shape_nose) = shape_nose {
    //     RenderContext::render_3d_shape(
    //         shape_data_to_render_3d_shape(
    //             shape_nose.shape_data(&mut st.shape()).unwrap(),
    //             Shape::Nose,
    //             usize::from(st.char_info.faceline_color),
    //             None,
    //         ),
    //         st,
    //         texture_view,
    //         encoder,
    //     );
    // }

    // // if let Some(mut shape_noseline) = shape_noseline {
    // //     RenderContext::render_3d_shape(
    // //         shape_data_to_render_3d_shape(
    // //             shape_noseline.shape_data(&mut st.shape()).unwrap(),
    // //             Shape::NoseLine,
    // //             0,
    // //             None,
    // //         ),
    // //         st,
    // //         texture_view,
    // //         encoder,
    // //     );
    // // }
    // if let Some(GenericResourceShape::Element(mut shape_glass)) = shape_glass {
    //     RenderContext::render_3d_shape(
    //         shape_data_to_render_3d_shape(
    //             shape_glass.shape_data(&mut st.shape()).unwrap(),
    //             Shape::Glasses,
    //             usize::from(st.char_info.glass_color),
    //             None,
    //         ),
    //         st,
    //         texture_view,
    //         encoder,
    //     );
    // }

    // RenderContext::render_3d_shape(
    //     shape_data_to_render_3d_shape(
    //         shape_mask.shape_data(&mut st.shape()).unwrap(),
    //         Shape::Mask,
    //         0,
    //         Some(mask_texture),
    //     ),
    //     st,
    //     texture_view,
    //     encoder,
    // );
}

fn load_shape(
    shape_kind: Shape,
    shape_index: u8,
    shape_color: u8,
    st: &mut State,
    encoder: &mut CommandEncoder,
) -> Option<Rendered3dShape> {
    // println!("Loading shp {shape_kind:?}[{shape_index:?}] col#{shape_color:?}");
    let res_shape: ResourceShape = ResourceShape::read(&mut st.shape()).unwrap();
    let res_texture = ResourceTexture::read(&mut st.texture()).unwrap();
    let mut file_shape = st.shape();

    let GenericResourceShape::FaceLineTransform(faceline_transform) = res_shape
        .fetch_shape(
            Shape::FaceLineTransform,
            usize::from(st.char_info.faceline_type),
        )
        .unwrap()
    else {
        panic!()
    };

    let GenericResourceShape::Element(mut shape_element) = res_shape
        .fetch_shape(shape_kind, usize::from(shape_index))
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
            let tex = res_texture.noseline[1]
                .get_image(&mut BufReader::new(
                    File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
                ))
                .unwrap()
                .unwrap();

            let tex = DynamicImage::ImageRgba8(tex);

            let noseline_texture =
                texture::Texture::from_image(&st.device, &st.queue, &tex, None).unwrap();
            // let noseline_texture = crate::texture::Texture::create_texture(
            //     &st.device,
            //     &PhysicalSize::<u32>::new(128, 128),
            //     "noselinetex",
            // );

            // draw_noseline(st, &noseline_texture.view, encoder);

            Some(noseline_texture)
        }
        Shape::Mask => {
            let mask_texture = crate::texture::Texture::create_texture(
                &st.device,
                &PhysicalSize::<u32>::new(512, 512),
                "masktex",
            );

            draw_mask(st, &mask_texture.view, encoder);

            Some(mask_texture)
        }
        Shape::FaceLine => {
            let faceline_texture = crate::texture::Texture::create_texture(
                &st.device,
                &PhysicalSize::<u32>::new(512, 512),
                "facelinetex",
            );

            draw_faceline(st, &faceline_texture.view, encoder);

            Some(faceline_texture)
        }
        _ => None,
    };

    Some(shape_data_to_render_3d_shape(
        shape_element.shape_data(&mut file_shape).unwrap(),
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
