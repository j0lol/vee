use crate::draw::model::{beard, cap, face_line, forehead, glasses, hair, mask, nose, nose_line};
use crate::{Model3d, ProgramState};
use vfl::charinfo::nx::NxCharInfo;
use wgpu::{CommandEncoder, TextureView};

pub(crate) mod model;
pub(crate) mod texture;

type Model = Model3d;
type ModelOpt = Option<Model3d>;

/// A bundle of models that in totality represent a character.
#[derive(Debug)]
pub struct CharModel {
    pub face_line: Model,
    pub forehead: ModelOpt,
    pub mask: Model,
    pub hair: ModelOpt,
    pub nose: ModelOpt,
    pub glasses: ModelOpt,
    pub nose_line: Model,
    pub beard: ModelOpt,
    pub cap: ModelOpt,
}

impl CharModel {
    pub fn new(
        st: &mut impl ProgramState,
        char_info: &NxCharInfo,
        encoder: &mut CommandEncoder,
    ) -> CharModel {
        CharModel {
            face_line: face_line(st, char_info, encoder).unwrap(),
            forehead: forehead(st, char_info, encoder),
            mask: mask(st, char_info, encoder).unwrap(),
            hair: hair(st, char_info, encoder),
            nose: nose(st, char_info, encoder),
            glasses: glasses(st, char_info, encoder),
            nose_line: nose_line(st, char_info, encoder).unwrap(),
            beard: beard(st, char_info, encoder),
            cap: cap(st, char_info, encoder),
        }
    }

    pub fn render(
        &mut self,
        st: &mut impl ProgramState,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        st.draw_model_3d(&mut self.face_line, texture_view, encoder);

        if let Some(forehead) = self.forehead.as_mut() {
            st.draw_model_3d(forehead, texture_view, encoder);
        }

        if let Some(hair) = self.hair.as_mut() {
            st.draw_model_3d(hair, texture_view, encoder);
        }

        st.draw_model_3d(&mut self.mask, texture_view, encoder);

        if let Some(nose) = self.nose.as_mut() {
            st.draw_model_3d(nose, texture_view, encoder);
        }

        st.draw_model_3d(&mut self.nose_line, texture_view, encoder);

        if let Some(glasses) = self.glasses.as_mut() {
            st.draw_model_3d(glasses, texture_view, encoder);
        }
        if let Some(beard) = self.beard.as_mut() {
            st.draw_model_3d(beard, texture_view, encoder);
        }
        if let Some(cap) = self.cap.as_mut() {
            st.draw_model_3d(cap, texture_view, encoder);
        }
    }
}
