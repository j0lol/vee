#[cfg(test)]
mod tests {
    use crate::charinfo;
    use crate::charinfo::nx::NxCharInfo;
    use crate::res::shape::nx::{ResourceShape, SHAPE_MID_DAT};
    use crate::res::tex::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT};
    use binrw::BinRead;
    use std::error::Error;
    use std::{fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn faceline_makeup() -> R {
        let char = NxCharInfo::read(&mut File::open("../charline.charinfo").unwrap()).unwrap();
        let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

        let res_texture = ResourceTexture::read(&mut bin)?;

        // let res = res.makeup[1];

        let tex = res_texture.makeup[char.faceline_make as usize]
            .get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;

        if let Some(tex) = tex {
            tex.save(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_output/faceline_makeup.png"
            ))?;
        }

        Ok(())
    }
}
