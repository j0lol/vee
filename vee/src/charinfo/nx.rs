use binrw::{NullWideString, binrw};
#[derive(Debug)]
#[binrw]
#[brw(little)]

pub struct UuidVer4 {
    idc: [u8; 16],
}
#[derive(Debug)]
#[binrw]
#[brw(little, assert(nickname.len() <= 22))]
pub struct NxCharInfo {
    create_info: UuidVer4,
    #[brw(pad_size_to = 0x16)]
    nickname: NullWideString,
    font_region: u8,
    favorite_color: u8,
    gender: u8,
    height: u8,
    build: u8,
    is_special: u8,
    region_move: u8,
    faceline_type: u8,
    faceline_color: u8,
    faceline_wrinkle: u8,
    faceline_make: u8,
    hair_type: u8,
    hair_color: u8,
    hair_flip: u8,
    eye_type: u8,
    eye_color: u8,
    eye_scale: u8,
    eye_aspect: u8,
    eye_rotate: u8,
    eye_x: u8,
    eye_y: u8,
    eyebrow_type: u8,
    eyebrow_color: u8,
    eyebrow_scale: u8,
    eyebrow_aspect: u8,
    eyebrow_rotate: u8,
    eyebrow_x: u8,
    eyebrow_y: u8,
    nose_type: u8,
    nose_scale: u8,
    nose_y: u8,
    mouth_type: u8,
    mouth_color: u8,
    mouth_scale: u8,
    mouth_aspect: u8,
    mouth_y: u8,
    beard_color: u8,
    beard_type: u8,
    mustache_type: u8,
    mustache_scale: u8,
    mustache_y: u8,
    glass_type: u8,
    glass_color: u8,
    glass_scale: u8,
    glass_y: u8,
    mole_type: u8,
    mole_scale: u8,
    mole_x: u8,
    mole_y: u8,
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
        let mut mii = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../j0.charinfo"))?;

        let mii = NxCharInfo::read(&mut mii)?;

        assert_eq!(mii.glass_color, 17);
        assert_eq!(mii.reserved, 0);

        Ok(())
    }
}
