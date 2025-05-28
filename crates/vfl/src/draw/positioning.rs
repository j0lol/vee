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

// FFLiCharInfo fn FFLiiGetEyeRotateOffset
const fn eye_rot_offset(i: usize) -> u8 {
    const OFFSETS: [u8; 80] = [
        3, 4, 4, 4, 3, 4, 4, 4, 3, 4, 4, 4, 4, 3, 3, 4, 4, 4, 3, 3, 4, 3, 4, 3, 3, 4, 3, 4, 4, 3,
        4, 4, 4, 3, 3, 3, 4, 4, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 3, 4, 4, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    ];

    32 - OFFSETS[i]
}

// todo use above
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
    pub fn init(info: &NxCharInfo, resolution: f32) -> MaskFaceParts {
        // RFLi_TEX_UNIT
        let base_scale = tex_unit(resolution);

        let eye_base_scale = tex_scale2dim(info.eye_scale.into());
        let eye_base_scale_y = 0.12 * f32::from(info.eye_aspect) + 0.64;

        let eye_spacing_x = TEX_EYE_BASE_X + TEX_SCALE_X * f32::from(info.eye_x);
        let eye_y = TEX_EYE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.eye_y);
        let eye_w = TEX_EYE_BASE_W * eye_base_scale;
        let eye_h = TEX_EYE_BASE_H * eye_base_scale * eye_base_scale_y;
        let eye_a = tex_rotate2ang(i16::from(
            info.eye_rotate + eye_rot_offset(info.eye_type as usize),
        ));

        let eye_l = FacePart {
            x: base_scale * (32.0 + eye_spacing_x),
            y: eye_y * base_scale,
            width: eye_w * base_scale,
            height: eye_h * base_scale,
            angle_deg: 360.0 - eye_a,
            origin: ImageOrigin::Left,
        };
        let eye_r = FacePart {
            x: base_scale * (32.0 - eye_spacing_x),
            y: eye_y * base_scale,
            width: eye_w * base_scale,
            height: eye_h * base_scale,
            angle_deg: eye_a,
            origin: ImageOrigin::Right,
        };

        let eb_base_scale = tex_scale2dim(info.eyebrow_scale.into());
        let eb_base_scale_y = 0.12 * f32::from(info.eyebrow_aspect) + 0.64;

        let eb_spacing_x = TEX_EYEBROW_BASE_X + TEX_SCALE_X * f32::from(info.eyebrow_x);
        let eb_y =
            TEX_EYEBROW_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.eyebrow_y);
        let eb_w = TEX_EYEBROW_BASE_W * eb_base_scale;
        let eb_h = TEX_EYEBROW_BASE_H * eb_base_scale * eb_base_scale_y;
        let eb_a = tex_rotate2ang(
            (info.eyebrow_rotate + EYEBROW_ROT_OFFSET[info.eyebrow_type as usize]).into(),
        );
        let eb_l = FacePart {
            x: base_scale * (32.0 + eb_spacing_x),
            y: eb_y * base_scale,
            width: eb_w * base_scale,
            height: eb_h * base_scale,
            angle_deg: 360.0 - eb_a,
            origin: ImageOrigin::Left,
        };
        let eb_r = FacePart {
            x: base_scale * (32.0 - eb_spacing_x),
            y: eb_y * base_scale,
            width: eb_w * base_scale,
            height: eb_h * base_scale,
            angle_deg: eb_a,
            origin: ImageOrigin::Right,
        };

        let mouth_base_scale = tex_scale2dim(info.mouth_scale.into());
        let mouth_base_scale_y = 0.12 * f32::from(info.mouth_aspect) + 0.64;
        let mouth_y = TEX_MOUTH_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mouth_y);
        let mouth_w = TEX_MOUTH_BASE_W * mouth_base_scale;
        let mouth_h = TEX_MOUTH_BASE_H * mouth_base_scale * mouth_base_scale_y;

        let mouth = FacePart {
            x: base_scale * 32.0,
            y: mouth_y * base_scale,
            width: mouth_w * base_scale,
            height: mouth_h * base_scale,
            angle_deg: 0.0,
            origin: ImageOrigin::Center,
        };

        let mus_y =
            TEX_MUSTACHE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mustache_y);
        let mus_w = TEX_MUSTACHE_BASE_W * tex_scale2dim(info.mustache_scale.into());
        let mus_h = TEX_MUSTACHE_BASE_H * tex_scale2dim(info.mustache_scale.into());

        let mus_l = FacePart {
            x: base_scale * 32.0,
            y: mus_y * base_scale,
            width: mus_w * base_scale,
            height: mus_h * base_scale,
            angle_deg: 0.0,
            origin: ImageOrigin::Left,
        };
        let mus_r = FacePart {
            x: base_scale * 32.0,
            y: mus_y * base_scale,
            width: mus_w * base_scale,
            height: mus_h * base_scale,
            angle_deg: 0.0,
            origin: ImageOrigin::Right,
        };

        let mole_x = TEX_MOLE_BASE_X + 2.0 * TEX_SCALE_X * f32::from(info.mole_x);
        let mole_y = TEX_MOLE_BASE_Y + RFL_MAGIC_Y_OFFSET * TEX_SCALE_Y * f32::from(info.mole_y);
        let mole_w = tex_scale2dim(info.mole_scale.into());
        let mole_h = tex_scale2dim(info.mole_scale.into());

        let mole = FacePart {
            x: mole_x * base_scale,
            y: mole_y * base_scale,
            width: mole_w * base_scale,
            height: mole_h * base_scale,
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
    // use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    // use binrw::BinRead;
    // use std::error::Error;
    // use std::{fs::File, io::BufReader};

    // type R = Result<(), Box<dyn Error>>;

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
