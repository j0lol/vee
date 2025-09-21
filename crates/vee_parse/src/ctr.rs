use crate::error::CharConversionError;
use crate::generic::{
    AsGenericChar, Beard, Body, CreationData, CtrCreationData, Eye, Eyebrow, Faceline,
    FavoriteColor, Gender, GenericColor, Glass, Hair, MetaData, Mole, Mouth, Mustache, Nose,
    Position, PositionY, Rotation, Scale, ScaleX, ScaleY, UniformScale,
};
use crate::nx::NxColor;
use crate::seal::Sealant;
use crate::{FixedLengthWideString, GenericChar, u8_to_bool};
use bilge::prelude::*;
use binrw::{BinRead, BinWrite, binrw};
use vee_parse_macros::bitfield;

#[bitfield(32)]
pub struct PersonalInfo1Field {
    pub char_version: u8,
    pub copyable: u1,
    pub ng_word: u1,
    pub region_move: u2,
    pub font_region: u2,
    pub reserved_0: u2,
    pub room_index: u4,
    pub position_in_room: u4,
    pub author_type: u4,
    pub birth_platform: u3,
    pub reserved_1: u1,
}

#[bitfield(16)]
pub struct PersonalInfo2Field {
    pub gender: u1,
    pub birth_month: u4,
    pub birth_day: u5,
    pub favorite_color: u4,
    pub favorite: u1,
    pub padding: u1,
}

#[bitfield(16)]
pub struct FaceField {
    pub local_only: u1,
    pub face_type: u4,
    pub face_color: u3,
    pub face_texture: u4,
    pub face_makeup: u4,
}

#[bitfield(16)]
pub struct HairField {
    pub hair_type: u8,
    pub hair_color: u3,
    pub hair_flip: u1,
    pub padding: u4,
}

#[bitfield(16)]
pub struct EyeField {
    pub eye_type: u6,
    pub eye_color: u3,
    pub eye_scale: u4,
    pub eye_aspect: u3,
}

#[bitfield(16)]
pub struct EyePositionField {
    pub eye_rotate: u5,
    pub eye_x: u4,
    pub eye_y: u5,
    pub padding: u2,
}

#[bitfield(16)]
pub struct EyebrowField {
    pub eyebrow_type: u5,
    pub eyebrow_color: u3,
    pub eyebrow_scale: u4,
    pub eyebrow_aspect: u3,
    pub padding_3: u1,
}

#[bitfield(16)]
pub struct EyebrowPositionField {
    pub eyebrow_rotate: u5,
    pub eyebrow_x: u4,
    pub eyebrow_y: u5,
    pub padding: u2,
}

#[bitfield(16)]
pub struct NoseField {
    pub nose_type: u5,
    pub nose_scale: u4,
    pub nose_y: u5,
    pub padding: u2,
}

#[bitfield(16)]
pub struct MouthField {
    pub mouth_type: u6,
    pub mouth_color: u3,
    pub mouth_scale: u4,
    pub mouth_aspect: u3,
}

#[bitfield(16)]
pub struct MouthPositionField {
    pub mouth_y: u5,
    pub mustache_type: u3,
    pub padding: u8,
}

#[bitfield(16)]
pub struct BeardField {
    pub beard_type: u3,
    pub beard_color: u3,
    pub mustache_scale: u4,
    pub mustache_y: u5,
    pub padding_7: u1,
}

#[bitfield(16)]
pub struct GlassField {
    pub glass_type: u4,
    pub glass_color: u3,
    pub glass_scale: u4,
    pub glass_y: u5,
}

#[bitfield(16)]
pub struct MoleField {
    pub mole_type: u1,
    pub mole_scale: u4,
    pub mole_x: u5,
    pub mole_y: u5,
    pub padding_8: u1,
}

#[binrw]
#[derive(Debug)]
pub struct CtrAuthorId {
    pub data: [u8; 8],
}

// FFLiCreateID
#[binrw]
#[derive(Debug)]
pub struct CtrCreateId {
    pub data: [u8; 10],
}

/// A packed character info format.
/// This structure has a lot of bitfields.
/// These fields have been given speculative names.
///
/// This format is known as `.ffsd`.
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct CtrStoreData {
    pub personal_info_1: PersonalInfo1Field,
    pub author_id: CtrAuthorId,
    pub create_id: CtrCreateId,
    pub reserved: [u8; 2],
    pub personal_info_2: PersonalInfo2Field,
    pub name: FixedLengthWideString<10>,
    pub height: u8,
    pub build: u8,
    pub face: FaceField,
    pub hair: HairField,
    pub eye: EyeField,
    pub eye_position: EyePositionField,
    pub eyebrow: EyebrowField,
    pub eyebrow_position: EyebrowPositionField,
    pub nose: NoseField,
    pub mouth: MouthField,
    pub mouth_position: MouthPositionField,
    pub beard: BeardField,
    pub glass: GlassField,
    pub mole: MoleField,
    pub creator_name: FixedLengthWideString<10>,
    pub padding: u16,
    pub crc: u16,
}

fn cafe_color_generic(col: u8) -> GenericColor {
    GenericColor::CafeTable(col.into())
}

impl Sealant for CtrStoreData {}

impl AsGenericChar for CtrStoreData {
    fn as_generic(&self) -> Result<GenericChar, CharConversionError> {
        let char = GenericChar {
            name: self.name.to_string(),

            creation_data: CreationData::Ctr(CtrCreationData {}),
            body: Body {
                gender: Gender::from_u8(self.personal_info_2.gender().as_u8())?,
                height: self.height,
                build: self.build,
            },
            faceline: Faceline {
                ty: self.face.face_type().as_u8(),
                color: cafe_color_generic(self.face.face_color().as_u8()),
                wrinkle_ty: self.face.face_texture().as_u8(),
                makeup_ty: self.face.face_makeup().as_u8(),
            },
            hair: Hair {
                ty: self.hair.hair_type(),
                color: cafe_color_generic(self.hair.hair_color().as_u8()),
                flip: u8_to_bool(self.hair.hair_flip().as_u8(), "hair::flip".to_string())?,
            },
            eye: Eye {
                ty: self.eye.eye_type().as_u8(),
                color: cafe_color_generic(self.eye.eye_color().as_u8()),
                pos: Position {
                    x: self.eye_position.eye_x().as_u8(),
                    y: self.eye_position.eye_y().as_u8(),
                },
                scale: Scale {
                    w: self.eye.eye_scale().as_u8(),
                    h: self.eye.eye_aspect().as_u8(),
                },
                rotation: Rotation {
                    ang: self.eye_position.eye_rotate().as_u8(),
                },
            },
            eyebrow: Eyebrow {
                ty: self.eyebrow.eyebrow_type().as_u8(),
                color: cafe_color_generic(self.eyebrow.eyebrow_color().as_u8()),
                pos: Position {
                    x: self.eyebrow_position.eyebrow_x().as_u8(),
                    y: self.eyebrow_position.eyebrow_y().as_u8(),
                },
                scale: Scale {
                    w: self.eyebrow.eyebrow_scale().as_u8(),
                    h: self.eyebrow.eyebrow_aspect().as_u8(),
                },
                rotation: Rotation {
                    ang: self.eyebrow_position.eyebrow_rotate().as_u8(),
                },
            },
            nose: Nose {
                ty: self.nose.nose_type().as_u8(),
                pos: PositionY {
                    y: self.nose.nose_y().as_u8(),
                },
                scale: UniformScale {
                    amount: self.nose.nose_scale().as_u8(),
                },
            },
            mouth: Mouth {
                ty: self.mouth.mouth_type().as_u8(),
                color: cafe_color_generic(self.mouth.mouth_color().as_u8()),
                pos: PositionY {
                    y: self.mouth_position.mouth_y().as_u8(),
                },
                scale: Scale {
                    w: self.mouth.mouth_scale().as_u8(),
                    h: self.mouth.mouth_aspect().as_u8(),
                },
            },
            beard: Beard {
                ty: self.beard.beard_type().as_u8(),
                color: cafe_color_generic(self.beard.beard_color().as_u8()),
            },
            mustache: Mustache {
                ty: self.mouth_position.mustache_type().as_u8(),
                pos: PositionY {
                    y: self.beard.mustache_y().as_u8(),
                },
                scale: ScaleX {
                    w: self.beard.mustache_scale().as_u8(),
                },
            },
            glass: Glass {
                ty: self.glass.glass_type().as_u8(),
                color: cafe_color_generic(self.glass.glass_color().as_u8()),
                pos: PositionY {
                    y: self.glass.glass_y().as_u8(),
                },
                scale: ScaleX {
                    w: self.glass.glass_scale().as_u8(),
                },
            },
            mole: Mole {
                ty: self.mole.mole_type().as_u8(),
                pos: Position {
                    x: self.mole.mole_x().as_u8(),
                    y: self.mole.mole_y().as_u8(),
                },
                scale: ScaleX {
                    w: self.mole.mole_scale().as_u8(),
                },
            },
            meta_data: MetaData {
                special: {
                    println!("Warn: Special flag is NOT being read. Placeholder value used.");
                    false
                },
                favorite_color: FavoriteColor(self.personal_info_2.favorite_color().as_usize()),
            },
        };

        Ok(char)
    }
}
