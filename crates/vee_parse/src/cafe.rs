use crate::FixedLengthWideString;
use std::{fmt::Display, fs::read, io::Cursor};

use bilge::prelude::*;
use binrw::{BinRead, BinWrite, NullWideString, binrw};
use vee_parse_macros::bitfield;

#[bitfield(32)]
pub struct CharDataField {
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
pub struct PersonalInfoField {
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
    pub beard_scale: u4,
    pub beard_y: u5,
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
pub struct CafeAuthorId {
    pub data: [u8; 8],
}

#[binrw]
#[derive(Debug)]
pub struct CafeCreateId {
    pub data: [u8; 10],
}

/// A packed character info format.
/// This structure has a lot of bitfields.
/// These fields have been given speculative names.
///
/// This format is commonly known as `.ffsd`.
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct CafeCharStoreData {
    pub char_data: CharDataField,
    pub author_id: CafeAuthorId,
    pub create_id: CafeCreateId,
    pub reserved: [u8; 2],
    pub personal_info: PersonalInfoField,
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
