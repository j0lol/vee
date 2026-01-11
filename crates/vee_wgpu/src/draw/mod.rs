//! Drawing models and textures.

use crate::draw::model::{beard, face_line, forehead, glasses, hair, hat, mask, nose, nose_line};
use crate::{Model3d, ProgramState};
use vee_parse::NxCharInfo;
use vee_resources::packing::Float16;
use wgpu::{CommandEncoder, TextureView};

pub(crate) mod body;
pub(crate) mod model;
pub(crate) mod texture;

type Model = Model3d;
type ModelOpt = Option<Model3d>;

/// A bundle of models that in totality represent a `Char`.
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
    pub hat: ModelOpt,
    pub extras: Vec<Model3d>,
    pub head_transform: glam::Mat4,
}

impl CharModel {
    pub fn new(
        st: &mut impl ProgramState,
        char_info: &NxCharInfo,
        encoder: &mut CommandEncoder,
    ) -> CharModel {
        // TODO: Don't hardcode this path
        let (extras, head_transform) =
            body::load_body(char_info, "resources_here/miibodymiddle female test.glb")
                .unwrap_or_else(|e| {
                    panic!("Failed to load GLTF: {}", e);
                });

        // Helper to transform a model
        let transform_model = |mut model: Model3d| {
            // Apply scale 10.0 to match the body
            let scale_matrix = glam::Mat4::from_scale(glam::Vec3::splat(10.0));
            // Scale head to match body proportion: 0.1 * (10 / 7) = 1/7
            let head_scale_correction = glam::Mat4::from_scale(glam::Vec3::splat(1.0 / 7.0));
            // Apply head transform (already in global space relative to scene root)
            let final_transform = scale_matrix * head_transform * head_scale_correction;

            // We need to bake the model's local scale and position into the vertices
            // because the head transform must apply to the *final* model-space coordinate.
            // Shader logic is: (vertex * scale) + position.
            // We want: Matrix * ((vertex * scale) + position).
            let model_scale = model.scale;
            let model_offset = model.position;

            // Reset uniforms so shader acts as pass-through for transform
            model.position = glam::Vec3::ZERO;
            model.scale = glam::Vec3::ONE;

            for vertex in &mut model.vertices {
                let pos = glam::Vec3::from([
                    vertex.position[0].as_f32(),
                    vertex.position[1].as_f32(),
                    vertex.position[2].as_f32(),
                ]);
                let normal = glam::Vec3::from(vertex.normal);

                // 1. Apply local scale
                // 2. Apply local offset
                let pos_local = (pos * model_scale) + model_offset;

                // 3. Apply global head transform
                let new_pos = final_transform.transform_point3(pos_local);

                // For normals:
                // Normal matrix is (M^-1)^T.
                // Here M = final_transform.
                // If M includes uniform scale (which it does), transform_vector3 works fine if we normalize.
                // We do NOT apply model_offset to normals (translation invariant).
                // We DO apply model_scale to normals if it's non-uniform, but Mii parts are usually uniform scale.
                // If uniform scale, normal direction doesn't change by scale.
                // However, `final_transform` has rotation, so we must rotate the normal.
                // Using transform_vector3 applies rotation and scale. Normalizing removes scale.
                let new_normal = final_transform.transform_vector3(normal).normalize();

                vertex.position = [
                    Float16::from_f32(new_pos.x),
                    Float16::from_f32(new_pos.y),
                    Float16::from_f32(new_pos.z),
                ];
                vertex.normal = new_normal.to_array();
            }
            model
        };

        // Set up required face parts
        let face_line = face_line(st, char_info, encoder)
            .map(transform_model)
            .expect("Required face part");
        let mask = mask(st, char_info, encoder)
            .map(transform_model)
            .expect("Required face part");
        let nose_line = nose_line(st, char_info, encoder)
            .map(transform_model)
            .expect("Required face part");

        // Set up "optional" face parts
        let forehead = forehead(st, char_info, encoder).map(transform_model);
        let hair = hair(st, char_info, encoder).map(transform_model);
        let nose = nose(st, char_info, encoder).map(transform_model);
        let glasses = glasses(st, char_info, encoder).map(transform_model);
        let beard = beard(st, char_info, encoder).map(transform_model);
        let hat = hat(st, char_info, encoder).map(transform_model);

        CharModel {
            face_line,
            forehead,
            mask,
            hair,
            nose,
            glasses,
            nose_line,
            beard,
            hat,
            extras,
            head_transform,
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
        if let Some(hat) = self.hat.as_mut() {
            st.draw_model_3d(hat, texture_view, encoder);
        }

        for extra in &mut self.extras {
            st.draw_model_3d(extra, texture_view, encoder);
        }
    }
}
