use crate::{
    FixedLengthWideString, GenericChar,
    error::CharConversionError,
    generic::{
        AsGenericChar, Beard, Body, CreationData, Eye, Eyebrow, Faceline, FromGenericChar, Gender,
        GenericColor, Glass, Hair, MetaData, Mole, Mouth, Mustache, Nose, NxCreationData, Position,
        PositionY, Rotation, Scale, ScaleX, UniformScale,
    },
    seal::Sealant,
    u8_to_bool,
};
use binrw::binrw;

/// Wrapper for nn::mii color index.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
#[binrw]
#[repr(transparent)]
pub struct NxColor(pub u8);

/// Simple UuidV4. No logic provided as create info is not useful for anything... yet.
#[derive(Debug, Copy, Clone)]
#[binrw]
#[brw(little)]
pub struct UuidVer4 {
    pub idc: [u8; 16],
}

/// Bindings to the `.charinfo` format.
/// This format is not used in transmission,
/// so is not packed like other formats (e.g. Cafe, Revolution.)
///
/// For more information on the layout, check the ImHex struct pattern:
/// [/testbed/nx_charinfo.hexpat](https://github.com/j0lol/vee/blob/main/testbed/nx_charinfo.hexpat)
#[derive(Debug, Clone)]
#[binrw]
#[brw(little)]
pub struct NxCharInfo {
    pub create_info: UuidVer4,
    pub nickname: FixedLengthWideString<11>,
    pub font_region: u8,
    pub favorite_color: u8,
    pub gender: u8,
    pub height: u8,
    pub build: u8,
    pub is_special: u8,
    pub region_move: u8,
    pub faceline_type: u8,
    pub faceline_color: u8,
    pub faceline_wrinkle: u8,
    pub faceline_make: u8,
    pub hair_type: u8,
    pub hair_color: u8,
    pub hair_flip: u8,
    pub eye_type: u8,
    pub eye_color: u8,
    pub eye_scale: u8,
    pub eye_aspect: u8,
    pub eye_rotate: u8,
    pub eye_x: u8,
    pub eye_y: u8,
    pub eyebrow_type: u8,
    pub eyebrow_color: u8,
    pub eyebrow_scale: u8,
    pub eyebrow_aspect: u8,
    pub eyebrow_rotate: u8,
    pub eyebrow_x: u8,
    pub eyebrow_y: u8,
    pub nose_type: u8,
    pub nose_scale: u8,
    pub nose_y: u8,
    pub mouth_type: u8,
    pub mouth_color: u8,
    pub mouth_scale: u8,
    pub mouth_aspect: u8,
    pub mouth_y: u8,
    pub beard_color: u8,
    pub beard_type: u8,
    pub mustache_type: u8,
    pub mustache_scale: u8,
    pub mustache_y: u8,
    pub glass_type: u8,
    pub glass_color: u8,
    pub glass_scale: u8,
    pub glass_y: u8,
    pub mole_type: u8,
    pub mole_scale: u8,
    pub mole_x: u8,
    pub mole_y: u8,
    pub reserved: u8, /* always zero */
}

use GenericColor as Color;

impl Sealant for NxCharInfo {}

impl AsGenericChar for NxCharInfo {
    fn as_generic(&self) -> Result<GenericChar, CharConversionError> {
        Ok(GenericChar {
            name: self.nickname.to_string(),
            body: Body {
                gender: Gender::from_u8(self.gender)?,
                height: self.height,
                build: self.build,
            },
            faceline: Faceline {
                ty: self.faceline_type,
                color: Color::nx_faceline(self.faceline_color),
                wrinkle_ty: self.faceline_wrinkle,
                makeup_ty: self.faceline_make,
            },
            hair: Hair {
                ty: self.hair_type,
                color: Color::nx_common(self.hair_color),
                flip: u8_to_bool(self.hair_flip, "hair::flip".to_string())?,
            },
            eye: Eye {
                ty: self.eye_type,
                color: Color::nx_common(self.eye_color),
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
                color: Color::nx_common(self.eyebrow_color),
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
                color: Color::nx_common(self.mouth_color),
                pos: PositionY { y: self.mouth_y },
                scale: Scale {
                    w: self.mouth_scale,
                    h: self.mouth_aspect,
                },
            },
            beard: Beard {
                ty: self.beard_type,
                color: Color::nx_common(self.beard_color),
            },
            mustache: Mustache {
                ty: self.mustache_y,
                pos: PositionY { y: self.mustache_y },
                scale: ScaleX {
                    w: self.mustache_scale,
                },
            },
            glass: Glass {
                ty: self.glass_type,
                color: Color::nx_common(self.glass_color),
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
                special: u8_to_bool(self.is_special, "meta_data::special".to_string())?,
                favorite_color: Color::favorite_color(self.favorite_color),
            },
            creation_data: CreationData::Nx(NxCreationData {
                create_info: self.create_info,
                font_region: self.font_region,
                region_move: self.region_move,
            }),
        })
    }
}

impl FromGenericChar for NxCharInfo {
    type Output = NxCharInfo;

    fn from_generic(char: GenericChar) -> Self::Output {
        NxCharInfo {
            create_info: UuidVer4 { idc: [0; 16] },
            nickname: FixedLengthWideString::from_string(char.name),
            font_region: 0,
            favorite_color: char.meta_data.favorite_color.to_nx().raw_index(),
            gender: char.body.gender.as_u8(),
            height: char.body.height,
            build: char.body.build,
            is_special: char.meta_data.special as u8,
            region_move: 0,
            faceline_type: char.faceline.ty,
            faceline_color: char.faceline.color.to_nx().raw_index(),
            faceline_wrinkle: char.faceline.wrinkle_ty,
            faceline_make: char.faceline.makeup_ty,
            hair_type: char.hair.ty,
            hair_color: char.hair.color.to_nx().raw_index(),
            hair_flip: char.hair.flip as u8,
            eye_type: char.eye.ty,
            eye_color: char.eye.color.to_nx().raw_index(),
            eye_scale: char.eye.scale.w,
            eye_aspect: char.eye.scale.h,
            eye_rotate: char.eye.rotation.ang,
            eye_x: char.eye.pos.x,
            eye_y: char.eye.pos.y,
            eyebrow_type: char.eyebrow.ty,
            eyebrow_color: char.eyebrow.color.to_nx().raw_index(),
            eyebrow_scale: char.eyebrow.scale.w,
            eyebrow_aspect: char.eyebrow.scale.h,
            eyebrow_rotate: char.eyebrow.rotation.ang,
            eyebrow_x: char.eyebrow.pos.x,
            eyebrow_y: char.eyebrow.pos.y,
            nose_type: char.nose.ty,
            nose_scale: char.nose.scale.amount,
            nose_y: char.nose.pos.y,
            mouth_type: char.mouth.ty,
            mouth_color: char.mouth.color.to_nx().raw_index(),
            mouth_scale: char.mouth.scale.w,
            mouth_aspect: char.mouth.scale.h,
            mouth_y: char.mouth.pos.y,
            beard_color: char.beard.color.to_nx().raw_index(),
            beard_type: char.beard.ty,
            mustache_type: char.mustache.ty,
            mustache_scale: char.mustache.scale.w,
            mustache_y: char.mustache.pos.y,
            glass_type: char.glass.ty,
            glass_color: char.glass.color.to_nx().raw_index(),
            glass_scale: char.glass.scale.w,
            glass_y: char.glass.pos.y,
            mole_type: char.mole.ty,
            mole_scale: char.mole.scale.w,
            mole_x: char.mole.pos.x,
            mole_y: char.mole.pos.x,
            reserved: 0,
        }
    }
}
