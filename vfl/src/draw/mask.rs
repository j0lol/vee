use crate::charinfo::nx::NxCharInfo;

use super::{TEX_SCALE_X, TEX_SCALE_Y};

const fn tex_scale2dim(scale: f32) -> f32 {
    1.0 + 0.4 * scale
}

// i16 to not lose precision
const fn tex_rotate2ang(rotate: i16) -> f32 {
    (360.0 / 32.0) * (rotate % 32) as f32
}

const fn tex_unit(x: f32) -> f32 {
    x / 64.0
}

const TEX_EYE_BASE_X: f32 = tex_unit(0.0);
const TEX_EYE_BASE_Y: f32 = 18.451_525;
const TEX_EYE_BASE_W: f32 = tex_unit(342.0);
const TEX_EYE_BASE_H: f32 = tex_unit(288.0);

const TEX_EYEBROW_BASE_X: f32 = tex_unit(0.0);
const TEX_EYEBROW_BASE_Y: f32 = 16.549_807;
const TEX_EYEBROW_BASE_W: f32 = tex_unit(324.0);
const TEX_EYEBROW_BASE_H: f32 = tex_unit(288.0);

const TEX_MOUTH_BASE_Y: f32 = 29.25885;
const TEX_MOUTH_BASE_W: f32 = tex_unit(396.0);
const TEX_MOUTH_BASE_H: f32 = tex_unit(288.0);

const TEX_MUSTACHE_BASE_Y: f32 = 31.763_554;
const TEX_MUSTACHE_BASE_W: f32 = tex_unit(288.0);
const TEX_MUSTACHE_BASE_H: f32 = tex_unit(576.0);

const TEX_MOLE_BASE_X: f32 = 17.766_165;
const TEX_MOLE_BASE_Y: f32 = 17.95986;
const TEX_MOLE_BASE_W: f32 = tex_unit(0.0);
const TEX_MOLE_BASE_H: f32 = tex_unit(0.0);

const EYE_ROT_OFFSET: [u8; 50] = [
    29, 28, 28, 28, 29, 28, 28, 28, 29, 28, 28, 28, 28, 29, 29, 28, 28, 28, 29, 29, 28, 29, 28, 29,
    29, 28, 29, 28, 28, 29, 28, 28, 28, 29, 29, 29, 28, 28, 29, 29, 29, 28, 28, 29, 29, 29, 29, 29,
    28, 28,
];

const EYEBROW_ROT_OFFSET: [u8; 24] = [
    26, 26, 27, 25, 26, 25, 26, 25, 28, 25, 26, 24, 27, 27, 26, 26, 25, 25, 26, 26, 27, 26, 25, 27,
];

// Found in RFL, no idea what it is
const RFL_MAGIC_Y_OFFSET: f32 = 1.160_000_1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOrigin {
    Center,
    Left,
    Right,
    Ignore,
}

#[derive(Clone, Copy, Debug)]
pub struct FacePart {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub angle_deg: f32,
    pub origin: ImageOrigin,
}

/// The positioning of all parts of the face on the mask.
#[derive(Clone, Copy, Debug)]
pub struct MaskFaceParts {
    pub eye: [FacePart; 2],
    pub eyebrow: [FacePart; 2],
    pub mouth: FacePart,
    pub mustache: [FacePart; 2],
    pub mole: FacePart,
}

impl MaskFaceParts {
    pub fn init_glasses(
        info: &NxCharInfo,
        resolution: f32,
        nose_translate: [f32; 3],
    ) -> [FacePart; 2] {
        let resolution = resolution / 64.0;

        let glass_y = TEX_EYE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.glass_y);
        let glass_w = TEX_EYE_BASE_W * tex_scale2dim(info.glass_scale.into());
        let glass_h = TEX_EYE_BASE_H * tex_scale2dim(info.glass_scale.into());
        // let eye_a =
        //     tex_rotate2ang((info + EYE_ROT_OFFSET[info.glass_type as usize]).into());

        let eye_l = FacePart {
            x: resolution * (32.0),
            y: glass_y * resolution,
            width: glass_w * resolution,
            height: glass_h * resolution,
            angle_deg: 360.0 - 0.0,
            origin: ImageOrigin::Left,
        };
        let eye_r = FacePart {
            x: resolution * (32.0),
            y: glass_y * resolution,
            width: glass_w * resolution,
            height: glass_h * resolution,
            angle_deg: 0.0,
            origin: ImageOrigin::Right,
        };

        [eye_l, eye_r]
    }
    pub fn init(info: &NxCharInfo, resolution: f32) -> MaskFaceParts {
        // RFLi_TEX_UNIT
        let resolution = resolution / 64.0;

        let eye_spacing_x = TEX_EYE_BASE_X + TEX_SCALE_X * f32::from(info.eye_x);
        let eye_y = TEX_EYE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.eye_y);
        let eye_w = TEX_EYE_BASE_W * tex_scale2dim(info.eye_scale.into());
        let eye_h = TEX_EYE_BASE_H * tex_scale2dim(info.eye_scale.into());
        let eye_a =
            tex_rotate2ang((info.eye_rotate + EYE_ROT_OFFSET[info.eye_type as usize]).into());

        let eye_l = FacePart {
            x: resolution * (32.0 + eye_spacing_x),
            y: eye_y * resolution,
            width: eye_w * resolution,
            height: eye_h * resolution,
            angle_deg: 360.0 - eye_a,
            origin: ImageOrigin::Left,
        };
        let eye_r = FacePart {
            x: resolution * (32.0 - eye_spacing_x),
            y: eye_y * resolution,
            width: eye_w * resolution,
            height: eye_h * resolution,
            angle_deg: eye_a,
            origin: ImageOrigin::Right,
        };

        let eb_spacing_x = TEX_EYEBROW_BASE_X + TEX_SCALE_X * f32::from(info.eyebrow_x);
        let eb_y =
            TEX_EYEBROW_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.eyebrow_y);
        let eb_w = TEX_EYEBROW_BASE_W * tex_scale2dim(info.eyebrow_scale.into());
        let eb_h = TEX_EYEBROW_BASE_H * tex_scale2dim(info.eyebrow_scale.into());
        let eb_a = tex_rotate2ang(
            (info.eyebrow_rotate + EYEBROW_ROT_OFFSET[info.eyebrow_type as usize]).into(),
        );
        let eb_l = FacePart {
            x: resolution * (32.0 + eb_spacing_x),
            y: eb_y * resolution,
            width: eb_w * resolution,
            height: eb_h * resolution,
            angle_deg: 360.0 - eb_a,
            origin: ImageOrigin::Left,
        };
        let eb_r = FacePart {
            x: resolution * (32.0 - eb_spacing_x),
            y: eb_y * resolution,
            width: eb_w * resolution,
            height: eb_h * resolution,
            angle_deg: eb_a,
            origin: ImageOrigin::Right,
        };

        let mouth_y = TEX_MOUTH_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mouth_y);
        let mouth_w = TEX_MOUTH_BASE_W * tex_scale2dim(info.mouth_scale.into());
        let mouth_h = TEX_MOUTH_BASE_H * tex_scale2dim(info.mouth_scale.into());

        let mouth = FacePart {
            x: resolution * 32.0,
            y: mouth_y * resolution,
            width: mouth_w * resolution,
            height: mouth_h * resolution,
            angle_deg: 0.0,
            origin: ImageOrigin::Center,
        };

        let mus_y =
            TEX_MUSTACHE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mustache_y);
        let mus_w = TEX_MUSTACHE_BASE_W * tex_scale2dim(info.mustache_scale.into());
        let mus_h = TEX_MUSTACHE_BASE_H * tex_scale2dim(info.mustache_scale.into());

        let mus_l = FacePart {
            x: resolution * 32.0,
            y: mus_y * resolution,
            width: mus_w * resolution,
            height: mus_h * resolution,
            angle_deg: 0.0,
            origin: ImageOrigin::Left,
        };
        let mus_r = FacePart {
            x: resolution * 32.0,
            y: mus_y * resolution,
            width: mus_w * resolution,
            height: mus_h * resolution,
            angle_deg: 0.0,
            origin: ImageOrigin::Right,
        };

        let mole_x = TEX_MOLE_BASE_X + 2.0 * TEX_SCALE_X * f32::from(info.mole_x);
        let mole_y = TEX_MOLE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mole_y);
        let mole_w = TEX_MOLE_BASE_W * tex_scale2dim(info.mole_scale.into());
        let mole_h = TEX_MOLE_BASE_H * tex_scale2dim(info.mole_scale.into());

        let mole = FacePart {
            x: mole_x * resolution,
            y: mole_y * resolution,
            width: mole_w * resolution,
            height: mole_h * resolution,
            angle_deg: 0.0,
            origin: ImageOrigin::Center,
        };

        MaskFaceParts {
            eye: [eye_l, eye_r],
            eyebrow: [eb_l, eb_r],
            mouth,
            mustache: [mus_l, mus_r],
            mole,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    use binrw::BinRead;
    use std::error::Error;
    use std::{fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    // #[test]
    // fn mask_shape() -> R {
    //     let mut bin = BufReader::new(File::open(SHAPE_MID_DAT)?);

    //     let res = ResourceShape::read(&mut bin)?;

    //     let mut shape = res.mask[1];
    //     let mut shape = shape.shape_data(&mut bin)?;

    //     // let mut file = File::open(SHAPE_MID_DAT)?;

    //     // let gltf = shape.gltf(&mut file)?;
    //     // gltf.export_glb("jas.glb")?;
    //     //
    //     Ok(())
    // }
}
