use crate::{
    FixedLengthWideString, GenericChar, NxCharInfo,
    error::CharConversionError,
    generic::{
        AsGenericChar, Beard, Body, CreationData, Eye, Eyebrow, Faceline, FromGenericChar, Gender,
        GenericColor, Glass, Hair, MetaData, Mole, Mouth, Mustache, Nose, Position, PositionY,
        Rotation, Scale, ScaleX, UniformScale,
    },
    nx::UuidVer4,
    seal::Sealant,
};
use binrw::binrw;

/// Length of obfuscated studio data
const STUDIO_DATA_ENCODED_LENGTH: usize = 47;

/// Studio character information format.
/// This format is used in the Mii studio on the Nintendo website
/// and stored in browser localStorage.
#[derive(Debug, Clone)]
#[binrw]
#[brw(little)]
pub struct StudioCharInfo {
    pub beard_color: u8,
    pub beard_type: u8,
    pub build: u8,
    pub eye_aspect: u8,
    pub eye_color: u8,
    pub eye_rotate: u8,
    pub eye_scale: u8,
    pub eye_type: u8,
    pub eye_x: u8,
    pub eye_y: u8,
    pub eyebrow_aspect: u8,
    pub eyebrow_color: u8,
    pub eyebrow_rotate: u8,
    pub eyebrow_scale: u8,
    pub eyebrow_type: u8,
    pub eyebrow_x: u8,
    pub eyebrow_y: u8,
    pub faceline_color: u8,
    pub faceline_make: u8,
    pub faceline_type: u8,
    pub faceline_wrinkle: u8,
    pub favorite_color: u8,
    pub gender: u8,
    pub glass_color: u8,
    pub glass_scale: u8,
    pub glass_type: u8,
    pub glass_y: u8,
    pub hair_color: u8,
    pub hair_flip: u8,
    pub hair_type: u8,
    pub height: u8,
    pub mole_scale: u8,
    pub mole_type: u8,
    pub mole_x: u8,
    pub mole_y: u8,
    pub mouth_aspect: u8,
    pub mouth_color: u8,
    pub mouth_scale: u8,
    pub mouth_type: u8,
    pub mouth_y: u8,
    pub mustache_scale: u8,
    pub mustache_type: u8,
    pub mustache_y: u8,
    pub nose_scale: u8,
    pub nose_type: u8,
    pub nose_y: u8,
}

/// Decode obfuscated studio URL data
/// This reverses the encoding process used in studio URLs
pub fn studio_url_obfuscation_decode(data: &mut [u8]) {
    let mut previous = data[0];

    // Reverse the encoding process
    for i in 1..STUDIO_DATA_ENCODED_LENGTH {
        let encoded_byte = data[i];
        let mut original = encoded_byte.wrapping_sub(7); // reverse the addition of 7
        original ^= previous; // reverse the XOR with the previous encoded byte
        data[i - 1] = original;
        previous = encoded_byte; // update previous to the current encoded byte for next iteration
    }

    // The last byte will be overwritten, but that's fine as we only care about the first 46 bytes
}

impl StudioCharInfo {
    pub fn to_nxcharinfo(&self) -> NxCharInfo {
        NxCharInfo {
            create_info: UuidVer4 { idc: [0; 16] },
            nickname: FixedLengthWideString::from_string("Unknown".to_string()),
            font_region: 0,
            favorite_color: self.favorite_color,
            gender: self.gender,
            height: self.height,
            build: self.build,
            is_special: 0,
            region_move: 0,
            faceline_type: self.faceline_type,
            faceline_color: self.faceline_color,
            faceline_wrinkle: self.faceline_wrinkle,
            faceline_make: self.faceline_make,
            hair_type: self.hair_type,
            hair_color: self.hair_color,
            hair_flip: self.hair_flip,
            eye_type: self.eye_type,
            eye_color: self.eye_color,
            eye_scale: self.eye_scale,
            eye_aspect: self.eye_aspect,
            eye_rotate: self.eye_rotate,
            eye_x: self.eye_x,
            eye_y: self.eye_y,
            eyebrow_type: self.eyebrow_type,
            eyebrow_color: self.eyebrow_color,
            eyebrow_scale: self.eyebrow_scale,
            eyebrow_aspect: self.eyebrow_aspect,
            eyebrow_rotate: self.eyebrow_rotate,
            eyebrow_x: self.eyebrow_x,
            eyebrow_y: self.eyebrow_y,
            nose_type: self.nose_type,
            nose_scale: self.nose_scale,
            nose_y: self.nose_y,
            mouth_type: self.mouth_type,
            mouth_color: self.mouth_color,
            mouth_scale: self.mouth_scale,
            mouth_aspect: self.mouth_aspect,
            mouth_y: self.mouth_y,
            beard_color: self.beard_color,
            beard_type: self.beard_type,
            mustache_type: self.mustache_type,
            mustache_scale: self.mustache_scale,
            mustache_y: self.mustache_y,
            glass_type: self.glass_type,
            glass_color: self.glass_color,
            glass_scale: self.glass_scale,
            glass_y: self.glass_y,
            mole_type: self.mole_type,
            mole_scale: self.mole_scale,
            mole_x: self.mole_x,
            mole_y: self.mole_y,
            reserved: 0,
        }
    }
}

impl Sealant for StudioCharInfo {}

impl AsGenericChar for StudioCharInfo {
    fn as_generic(&self) -> Result<GenericChar, CharConversionError> {
        Ok(GenericChar {
            name: String::new(), // Studio format doesn't include name
            body: Body {
                gender: Gender::from_u8(self.gender)?,
                height: self.height,
                build: self.build,
            },
            faceline: Faceline {
                ty: self.faceline_type,
                color: GenericColor::nx_faceline(self.faceline_color),
                wrinkle_ty: self.faceline_wrinkle,
                makeup_ty: self.faceline_make,
            },
            hair: Hair {
                ty: self.hair_type,
                color: GenericColor::nx_common(self.hair_color),
                //flip: u8_to_bool(self.hair_flip, "hair::flip".to_string())?,
                flip: false,
            },
            eye: Eye {
                ty: self.eye_type,
                color: GenericColor::nx_common(self.eye_color),
                pos: Position {
                    x: self.eye_x,
                    y: self.eye_y,
                },
                scale: Scale {
                    w: self.eye_scale,
                    h: self.eye_aspect,
                },
                rotation: Rotation {
                    ang: self.eye_rotate,
                },
            },
            eyebrow: Eyebrow {
                ty: self.eyebrow_type,
                color: GenericColor::nx_common(self.eyebrow_color),
                pos: Position {
                    x: self.eyebrow_x,
                    y: self.eyebrow_y,
                },
                scale: Scale {
                    w: self.eyebrow_scale,
                    h: self.eyebrow_aspect,
                },
                rotation: Rotation {
                    ang: self.eyebrow_rotate,
                },
            },
            nose: Nose {
                ty: self.nose_type,
                pos: PositionY { y: self.nose_y },
                scale: UniformScale {
                    amount: self.nose_scale,
                },
            },
            mouth: Mouth {
                ty: self.mouth_type,
                color: GenericColor::nx_common(self.mouth_color),
                pos: PositionY { y: self.mouth_y },
                scale: Scale {
                    w: self.mouth_scale,
                    h: self.mouth_aspect,
                },
            },
            beard: Beard {
                ty: self.beard_type,
                color: GenericColor::nx_common(self.beard_color),
            },
            mustache: Mustache {
                ty: self.mustache_type,
                pos: PositionY { y: self.mustache_y },
                scale: ScaleX {
                    w: self.mustache_scale,
                },
            },
            glass: Glass {
                ty: self.glass_type,
                color: GenericColor::nx_common(self.glass_color),
                pos: PositionY { y: self.glass_y },
                scale: ScaleX {
                    w: self.glass_scale,
                },
            },
            mole: Mole {
                ty: self.mole_type,
                pos: Position {
                    x: self.mole_x,
                    y: self.mole_y,
                },
                scale: ScaleX { w: self.mole_scale },
            },
            meta_data: MetaData {
                special: false, // Studio format doesn't have special flag
                favorite_color: GenericColor::favorite_color(self.favorite_color),
            },
            creation_data: CreationData::None, // Studio format doesn't have creation data
        })
    }
}

impl FromGenericChar for StudioCharInfo {
    type Output = StudioCharInfo;

    fn from_generic(char: GenericChar) -> Self::Output {
        StudioCharInfo {
            beard_color: char.beard.color.to_nx().raw_index(),
            mustache_type: char.mustache.ty,
            build: char.body.build,
            eye_aspect: char.eye.scale.h,
            eye_color: char.eye.color.to_nx().raw_index(),
            eye_rotate: char.eye.rotation.ang,
            eye_scale: char.eye.scale.w,
            eye_type: char.eye.ty,
            eye_x: char.eye.pos.x,
            eye_y: char.eye.pos.y,
            eyebrow_aspect: char.eyebrow.scale.h,
            eyebrow_color: char.eyebrow.color.to_nx().raw_index(),
            eyebrow_rotate: char.eyebrow.rotation.ang,
            eyebrow_scale: char.eyebrow.scale.w,
            eyebrow_type: char.eyebrow.ty,
            eyebrow_x: char.eyebrow.pos.x,
            eyebrow_y: char.eyebrow.pos.y,
            faceline_color: char.faceline.color.to_nx().raw_index(),
            faceline_make: char.faceline.makeup_ty,
            faceline_type: char.faceline.ty,
            faceline_wrinkle: char.faceline.wrinkle_ty,
            favorite_color: char.meta_data.favorite_color.raw_index(),
            gender: char.body.gender.as_u8(),
            glass_color: char.glass.color.to_cafe_glass_or_default().raw_index(),
            glass_scale: char.glass.scale.w,
            glass_type: char.glass.ty,
            glass_y: char.glass.pos.y,
            hair_color: char.hair.color.to_cafe_hair_or_default().raw_index(),
            hair_flip: char.hair.flip as u8,
            hair_type: char.hair.ty,
            height: char.body.height,
            mole_scale: char.mole.scale.w,
            mole_type: char.mole.ty,
            mole_x: char.mole.pos.x,
            mole_y: char.mole.pos.y,
            mouth_aspect: char.mouth.scale.h,
            mouth_color: char.mouth.color.to_cafe_mouth_or_default().raw_index(),
            mouth_scale: char.mouth.scale.w,
            mouth_type: char.mouth.ty,
            mouth_y: char.mouth.pos.y,
            mustache_scale: char.mustache.scale.w,
            beard_type: char.beard.ty,
            mustache_y: char.mustache.pos.y,
            nose_scale: char.nose.scale.amount,
            nose_type: char.nose.ty,
            nose_y: char.nose.pos.y,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_studio_to_generic() {
        let studio = StudioCharInfo {
            beard_color: 0,
            mustache_type: 0,
            build: 64,
            eye_aspect: 64,
            eye_color: 0,
            eye_rotate: 0,
            eye_scale: 64,
            eye_type: 0,
            eye_x: 32,
            eye_y: 32,
            eyebrow_aspect: 64,
            eyebrow_color: 0,
            eyebrow_rotate: 0,
            eyebrow_scale: 64,
            eyebrow_type: 0,
            eyebrow_x: 32,
            eyebrow_y: 48,
            faceline_color: 0,
            faceline_make: 0,
            faceline_type: 0,
            faceline_wrinkle: 0,
            favorite_color: 0,
            gender: 0,
            glass_color: 0,
            glass_scale: 0,
            glass_type: 0,
            glass_y: 20,
            hair_color: 0,
            hair_flip: 0,
            hair_type: 0,
            height: 64,
            mole_scale: 64,
            mole_type: 0,
            mole_x: 32,
            mole_y: 64,
            mouth_aspect: 64,
            mouth_color: 0,
            mouth_scale: 64,
            mouth_type: 0,
            mouth_y: 48,
            mustache_scale: 64,
            beard_type: 0,
            mustache_y: 32,
            nose_scale: 64,
            nose_type: 0,
            nose_y: 64,
        };

        let generic = studio.as_generic().unwrap();
        assert_eq!(generic.body.height, 64);
        assert_eq!(generic.body.build, 64);
        assert_eq!(generic.body.gender, Gender::Male);
    }

    #[test]
    fn test_studio_obfuscation_decode() {
        // This is more of a sanity check - the actual test would need real obfuscated data
        let mut data = [0u8; STUDIO_DATA_ENCODED_LENGTH];
        let original = data;

        // Apply encoding (simplified version)
        for i in 1..STUDIO_DATA_ENCODED_LENGTH {
            data[i] = data[i].wrapping_add(7) ^ data[i - 1];
        }

        // Now decode it
        studio_url_obfuscation_decode(&mut data);

        // Should match original (except last byte which gets overwritten)
        for i in 0..STUDIO_DATA_ENCODED_LENGTH - 1 {
            assert_eq!(data[i], original[i]);
        }
    }
}
