use vfl::{
    draw::{faceline, render_3d::Rendered3dShape},
    res::shape::nx::{Shape, ShapeData},
};
use wgpu::{CommandEncoder, TextureView};

use crate::{State, char::load_shape};

type Tex = wgpu::Texture;
type TexOpt = Option<wgpu::Texture>;
type Model = Rendered3dShape;
type ModelOpt = Option<Rendered3dShape>;

#[derive(Debug)]
pub struct CharModel {
    pub face_line: Model,
    pub mask: Model,
    pub hair: ModelOpt,
    pub nose: ModelOpt,
    pub glasses: ModelOpt,
    pub nose_line: Model,
}
impl CharModel {
    pub fn new(st: &mut State, encoder: &mut CommandEncoder) -> CharModel {
        CharModel {
            face_line: face_line(st, encoder).unwrap(),
            mask: mask(st, encoder).unwrap(),
            hair: hair(st, encoder),
            nose: nose(st, encoder),
            glasses: glasses(st, encoder),
            nose_line: nose_line(st, encoder).unwrap(),
        }
    }

    pub fn render(
        &mut self,
        st: &mut State,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.face_line.render(st, texture_view, encoder);
        if let Some(hair) = self.hair.as_mut() {
            hair.render(st, texture_view, encoder);
        }
        self.mask.render(st, texture_view, encoder);

        if let Some(nose) = self.nose.as_mut() {
            nose.render(st, texture_view, encoder);
        }

        self.nose_line.render(st, texture_view, encoder);

        if let Some(glasses) = self.glasses.as_mut() {
            glasses.render(st, texture_view, encoder);
        }
    }
}

fn face_line(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(
        Shape::FaceLine,
        st.char_info.faceline_type,
        st.char_info.faceline_color,
        st,
        encoder,
    )
}

fn hair(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(
        Shape::HairNormal,
        st.char_info.hair_type,
        st.char_info.hair_color,
        st,
        encoder,
    )
}

fn mask(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(Shape::Mask, st.char_info.faceline_type, 0, st, encoder)
}

fn nose(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(
        Shape::Nose,
        st.char_info.nose_type,
        st.char_info.faceline_color,
        st,
        encoder,
    )
}

fn nose_line(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(Shape::NoseLine, st.char_info.nose_type, 0, st, encoder)
}

fn glasses(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    if st.char_info.glass_type != 0 {
        load_shape(Shape::Glasses, 0, st.char_info.glass_color, st, encoder)
    } else {
        None
    }
}
