use crate::{
    FixedLengthWideString, GenericChar,
    error::CharConversionError,
    generic::{
        AsGenericChar, Beard, Body, CreationData, Eye, Eyebrow, Faceline, FavoriteColor, Gender,
        GenericColor, Glass, Hair, MetaData, Mole, Mouth, Mustache, Nose, NxCreationData, Position,
        PositionY, Rotation, Scale, ScaleX, ScaleY, UniformScale,
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
    idc: [u8; 16],
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
    pub favorite_color: NxColor,
    pub gender: u8,
    pub height: u8,
    pub build: u8,
    pub is_special: u8,
    pub region_move: u8,
    pub faceline_type: u8,
    pub faceline_color: NxColor,
    pub faceline_wrinkle: u8,
    pub faceline_make: u8,
    pub hair_type: u8,
    pub hair_color: NxColor,
    pub hair_flip: u8,
    pub eye_type: u8,
    pub eye_color: NxColor,
    pub eye_scale: u8,
    pub eye_aspect: u8,
    pub eye_rotate: u8,
    pub eye_x: u8,
    pub eye_y: u8,
    pub eyebrow_type: u8,
    pub eyebrow_color: NxColor,
    pub eyebrow_scale: u8,
    pub eyebrow_aspect: u8,
    pub eyebrow_rotate: u8,
    pub eyebrow_x: u8,
    pub eyebrow_y: u8,
    pub nose_type: u8,
    pub nose_scale: u8,
    pub nose_y: u8,
    pub mouth_type: u8,
    pub mouth_color: NxColor,
    pub mouth_scale: u8,
    pub mouth_aspect: u8,
    pub mouth_y: u8,
    pub beard_color: NxColor,
    pub beard_type: u8,
    pub mustache_type: u8,
    pub mustache_scale: u8,
    pub mustache_y: u8,
    pub glass_type: u8,
    pub glass_color: NxColor,
    pub glass_scale: u8,
    pub glass_y: u8,
    pub mole_type: u8,
    pub mole_scale: u8,
    pub mole_x: u8,
    pub mole_y: u8,
    pub reserved: u8, /* always zero */
}

fn nx_color_generic(col: NxColor) -> GenericColor {
    GenericColor::NxTable(col.0.into())
}

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
                color: nx_color_generic(self.faceline_color),
                wrinkle_ty: self.faceline_wrinkle,
                makeup_ty: self.faceline_make,
            },
            hair: Hair {
                ty: self.hair_type,
                color: nx_color_generic(self.hair_color),
                flip: u8_to_bool(self.hair_flip, "hair::flip".to_string())?,
            },
            eye: Eye {
                ty: self.eye_type,
                color: nx_color_generic(self.eye_color),
                pos: Position {
                    x: self.eye_x,
                    y: self.eye_y,
                },
                scale: ScaleY { h: self.eye_aspect },
                rotation: Rotation {
                    ang: self.eye_rotate,
                },
            },
            eyebrow: Eyebrow {
                ty: self.eyebrow_type,
                color: nx_color_generic(self.eyebrow_color),
                pos: Position {
                    x: self.eyebrow_x,
                    y: self.eyebrow_y,
                },
                scale: Scale {
                    w: self.eye_scale,
                    h: self.eye_aspect,
                },
                rotation: Rotation {
                    ang: self.eye_rotate,
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
                color: nx_color_generic(self.mouth_color),
                pos: PositionY { y: self.mouth_y },
                scale: Scale {
                    w: self.mouth_scale,
                    h: self.mouth_aspect,
                },
            },
            beard: Beard {
                ty: self.beard_type,
                color: nx_color_generic(self.beard_color),
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
                color: nx_color_generic(self.glass_color),
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
                favorite_color: FavoriteColor(self.favorite_color.0.into()),
            },
            creation_data: CreationData::Nx(NxCreationData {
                create_info: self.create_info,
                font_region: self.font_region,
                region_move: self.region_move,
            }),
        })
    }
}
