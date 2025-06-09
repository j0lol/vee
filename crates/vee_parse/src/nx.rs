use binrw::NullWideString;
use binrw::binrw;

use crate::FixedLengthWideString;

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
