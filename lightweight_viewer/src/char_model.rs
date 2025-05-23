use vfl::{draw::render_3d::Rendered3dShape, res::shape::nx::Shape};
use wgpu::{CommandEncoder, TextureView};

use crate::{State, char::load_shape};

type Model = Rendered3dShape;
type ModelOpt = Option<Rendered3dShape>;

#[derive(Debug)]
pub struct CharModel {
    pub face_line: Model,
    pub mask: Model,
    pub hair: ModelOpt,
    pub nose: ModelOpt,
    pub nose_line: Model,
}
impl CharModel {
    pub fn new(st: &mut State, encoder: &mut CommandEncoder) -> CharModel {
        CharModel {
            face_line: face_line(st, encoder).unwrap(),
            mask: mask(st, encoder).unwrap(),
            hair: hair(st, encoder),
            nose: None,
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
        // if let Some(hair) = self.hair.as_mut() {
        //     hair.render(st, texture_view, encoder);
        // }
        // self.mask.render(st, texture_view, encoder);

        // self.nose_line.render(st, texture_view, encoder);
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

fn nose_line(st: &mut State, encoder: &mut CommandEncoder) -> ModelOpt {
    load_shape(Shape::NoseLine, st.char_info.nose_type, 0, st, encoder)
}
