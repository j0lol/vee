pub use binrw::{BinRead, NullWideString, binrw};

#[derive(Debug, Copy, Clone)]
#[binrw]
#[brw(little)]
pub struct UuidVer4 {
    idc: [u8; 16],
}
#[derive(Debug, Clone)]
#[binrw]
#[brw(little, assert(nickname.len() <= 22))]
pub struct NxCharInfo {
    create_info: UuidVer4,
    #[brw(pad_size_to = 0x16)]
    pub nickname: NullWideString,
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
    reserved: u8, /* always zero */
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::{error::Error, fs::File};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn mii_deser() -> R {
        let mut mii = File::open(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/resources_here/j0.charinfo"
        ))?;

        let mii = NxCharInfo::read(&mut mii)?;

        assert_eq!(mii.glass_color, 17);
        assert_eq!(mii.reserved, 0);

        Ok(())
    }
}
