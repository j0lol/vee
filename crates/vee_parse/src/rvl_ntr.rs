use crate::{
    FixedLengthWideString, GenericChar,
    generic::{
        AsGenericChar, Beard, Body, CreationData, Eye, Eyebrow, Faceline, Glass, Hair, MetaData,
        Mole, Mouth, Mustache, Nose, PositionY, Rotation, Scale, UniformScale,
    },
    seal::Sealant,
};
use bilge::prelude::*;
use binrw::{BinRead, BinWrite, binrw};
use paste::paste;
use vee_parse_macros::bitfield;

macro_rules! rvl_and_ntr {
    ( $(#[$attr:meta]) * $vis:vis struct $struct_name:ident { $($field_vis:vis $field_name:ident: $field_type:ty),* $(,)? }) => {
        paste! {
            #[binrw]
            #[brw(big)]
            $(#[$attr])*
            $vis struct [<Rvl $struct_name>] {
                $($field_vis $field_name: $field_type),*
            }

            #[binrw]
            #[brw(little)]
            $(#[$attr])*
            $vis struct [<Ntr $struct_name>] {
                $($field_vis $field_name: $field_type),*
            }
        }
    };
}

#[bitsize(4)]
#[repr(u8)]
#[derive(FromBits, Debug, PartialEq)]
pub enum FavoriteColor {
    Red = 0,
    Orange = 1,
    Yellow = 2,
    YellowGreen = 3,
    Green = 4,
    Blue = 5,
    SkyBlue = 6,
    Pink = 7,
    Purple = 8,
    Brown = 9,
    White = 10,
    Black = 11,
    #[fallback]
    Invalid(u4),
}

impl FavoriteColor {
    fn as_u8(&self) -> u8 {
        match self {
            FavoriteColor::Red => 0,
            FavoriteColor::Orange => 1,
            FavoriteColor::Yellow => 2,
            FavoriteColor::YellowGreen => 3,
            FavoriteColor::Green => 4,
            FavoriteColor::Blue => 5,
            FavoriteColor::SkyBlue => 6,
            FavoriteColor::Pink => 7,
            FavoriteColor::Purple => 8,
            FavoriteColor::Brown => 9,
            FavoriteColor::White => 10,
            FavoriteColor::Black => 11,
            FavoriteColor::Invalid(n) => n.as_u8(),
        }
    }
}

#[bitsize(1)]
#[derive(FromBits, Debug, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

#[bitfield(16)]
pub struct PersonalInfoField {
    pub padding: u1,
    /// Originally named sex
    pub gender: Gender,
    pub birth_month: u4,

    /// 0 = unset, Counts from 1-31
    /// Birth month and day must be set together if set.
    pub birth_day: u5,
    pub favorite_color: FavoriteColor,
    pub favorite: bool,
}

#[bitfield(16)]
pub struct FaceField {
    pub face_type: u3,
    pub face_color: u3,
    pub face_tex: u4,
    pub padding2: u3,
    pub localonly: u1,

    /// Set to 1 if downloaded from "Check Mii Out Channel".
    pub type_: u2,
}

#[bitfield(16)]
pub struct HairField {
    pub hair_type: u7,
    pub hair_color: u3,
    pub hair_flip: u1,
    pub padding3: u5,
}

#[bitfield(32)]
pub struct EyebrowField {
    pub eyebrow_type: u5,
    pub eyebrow_rotate: u5,
    pub padding4: u6,
    pub eyebrow_color: u3,
    pub eyebrow_scale: u4,
    pub eyebrow_y: u5,
    pub eyebrow_x: u4,
}

#[bitfield(32)]
pub struct EyeField {
    pub eye_type: u6,
    pub eye_rotate: u5,
    pub eye_y: u5,
    pub eye_color: u3,
    pub eye_scale: u4,
    pub eye_x: u4,
    pub padding5: u5,
}

#[bitfield(16)]
pub struct NoseField {
    pub nose_type: u4,
    pub nose_scale: u4,
    pub nose_y: u5,
    pub padding6: u3,
}

#[bitfield(16)]
pub struct MouthField {
    pub mouth_type: u5,
    pub mouth_color: u2,
    pub mouth_scale: u4,
    pub mouth_y: u5,
}

#[bitfield(16)]
pub struct GlassField {
    pub glass_type: u4,
    pub glass_color: u3,
    pub glass_scale: u4,
    pub glass_y: u5,
}

#[bitfield(16)]
pub struct FaceHairField {
    pub mustache_type: u2,
    pub beard_type: u2,
    pub beard_color: u3,
    pub beard_scale: u4,
    pub beard_y: u5,
}

#[bitfield(16)]
pub struct MoleField {
    pub mole_type: u1,
    pub mole_scale: u4,
    pub mole_y: u5,
    pub mole_x: u5,
    pub padding8: u1,
}

/// `flags` contain information about where the Char was created,
/// and some other miscellaneous state. See [RvlCreateIdFlags::platform].
///
/// `create_date_offset` contains an offset timestamp of creation date.
/// See [Self::create_date_timestamp]'s implementation for more information.
///
/// `addr_low` is built from the Mac address of the Rvl/Ntr console.
/// It contains a checksum of the first three bytes, and the last three bytes.
///
/// ```rust
/// fn build_addr_low(mac: &[u8; 6]) -> [u8; 4] {
///     let checksum = mac.iter().take(3).fold(0u8, |sum, &b| sum.wrapping_add(b)) & 0x7F;
///     [checksum, mac[3], mac[4], mac[5]]
/// }
/// ```
///
/// It is unknown how the checksum byte is calculated on Ntr targets.
///
/// In order for the Char to be considered as created from the
/// same console on Rvl (RFLiIsMyHomeID), the CreateId has to
/// be non-null, not from Ntr, and the `addr_low` has to match.
///
#[bitfield(64)]
pub struct CreateId {
    pub flags: CreateIdFlags,

    pub create_date_offset: u28,

    pub addr_low: [u8; 4],
}

impl CreateId {
    /// Outputs the creation date timestamp from the offset encoded.
    /// It is not known if this implementation is accurate.
    pub fn create_date_timestamp(&self) -> u32 {
        const JAN_1_2006: u32 = 1136073600;
        let offset = self.create_date_offset().as_u32();

        (offset * 4) + JAN_1_2006
    }
}

/// `Etc` can be either targets: Cafe, Nx, Miitomo
#[bitsize(2)]
#[derive(FromBits, Debug, PartialEq)]
pub enum CreateIdPlatform {
    Rvl = 0b00,
    Ctr = 0b01,
    Ntr = 0b10,
    Etc = 0b11,
}

/// These flags can be used to derive the creation platform,
/// see [Self::platform].
///
#[bitfield(4)]
pub struct CreateIdFlags {
    /// Cleared = Special, Set = Normal
    pub normal: bool,

    /// Cleared on Wii and 3DS, set on DS and Wii U.
    pub field_1: bool,

    /// Given to random Miis and seen in some games' CPU Miis.
    pub temporary: bool,

    /// Cleared on Wii and DS, set on 3DS and Wii U.
    pub field_3: bool,
}

impl CreateIdFlags {
    /// Outputs the creation platform for the Char.
    /// Uses `field_1` and `field_3`.
    pub fn platform(&self) -> CreateIdPlatform {
        let bits = u2::from_u8(match (self.field_1(), self.field_3()) {
            (false, false) => 0b00_u8,
            (true, false) => 0b10,
            (false, true) => 0b01,
            (true, true) => 0b11,
        });

        CreateIdPlatform::from(bits)
    }
}

rvl_and_ntr! {
    /// A packed character info format.
    /// This structure has a lot of bitfields.
    /// These fields have been given speculative names.
    ///
    /// This format is commonly known as `.{r,n}cd`.
    /// Rvl and Ntr only differ by endian-ness.
    #[derive(Debug)]
    pub struct CharData {
        pub personal_info: PersonalInfoField,
        pub name: FixedLengthWideString<10>,
        pub height: u8,
        pub build: u8,
        pub create_id: CreateId,
        pub face: FaceField,
        pub hair: HairField,
        pub eyebrow: EyebrowField,
        pub eye: EyeField,
        pub nose: NoseField,
        pub mouth: MouthField,
        pub glass: GlassField,
        pub face_hair: FaceHairField,
        pub mole: MoleField,
        pub creator_name: FixedLengthWideString<10>,
    }
}

/// A packed character info format.
/// RvlCharData with an attached checksum.
///
/// This format is commonly known as `.rsd`.
/// This format is big endian.
#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct RvlStoreData {
    pub data: RvlCharData,
    pub checksum: u16,
}

/// A packed character info format.
/// NtrCharData with an attached checksum.
///
/// This format is commonly known as `.nsd`.
/// This format is little endian.
#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct NtrStoreData {
    pub data: NtrCharData,
    pub checksum: u16,
}
