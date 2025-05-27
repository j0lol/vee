use crate::{res::shape::nx::ResourceCommonAttribute, utils::inflate_bytes};
use binrw::BinRead;
use image::{ImageBuffer, Rgba, RgbaImage};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::error::Error;
use tegra_swizzle::{block_height_mip0, div_round_up, swizzle::deswizzle_block_linear};

pub const TEXTURE_MID_SRGB_DAT: &str = concat!(
    env!("CARGO_WORKSPACE_DIR"),
    "/resources_here/NXTextureMidSRGB.dat"
);

#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceTextureAttribute {
    alignment: u32,
    pub width: u16,
    pub height: u16,
    pub format: ResourceTextureFormat,
    mip_count: u8,
    tile_mode: u8,
    pad: [u8; 1],
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct TextureElement {
    pub common: ResourceCommonAttribute,
    pub texture: ResourceTextureAttribute,
}

// Format rundown:
// https://www.reedbeta.com/blog/understanding-bcn-texture-compression-formats/#comparison-table

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy, BinRead)]
#[br(repr = u8)]
#[repr(u8)]
pub enum ResourceTextureFormat {
    R = 0,       // R8Unorm (Ffl Name)
    Rb = 1,      // R8B8Unorm
    Rgba = 2,    // R8B8G8A8Unorm
    Bc4 = 3,     // Bc4Unorm (Compressed R)
    Bc5 = 4,     // Bc5Unorm (Compressed Rb)
    Bc7 = 5,     // Bc7Unorm (Compressed Rgba)
    Astc4x4 = 6, // Astc4x4Unorm (Compressed Rgba)
}

impl TextureElement {
    /// Gets the raw bytes of a texture. Takes an argument of the resource file.
    /// # Errors
    /// - Encounters texture data that isn't Zlib deflated
    /// - Deswizzling texture data fails
    pub fn get_texture_bytes(&self, file: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        use ResourceTextureFormat as Rtf;

        let start: usize = self.common.offset as usize;
        let end: usize = self.common.offset as usize + self.common.size_compressed as usize;

        let range = start..end;

        let tex_data = inflate_bytes(&file[range])?;

        let needs_swizzling = self.texture.tile_mode == 0;

        let tex_data = if needs_swizzling {
            let block_size = match self.texture.format {
                Rtf::R | Rtf::Rb | Rtf::Rgba => 1,
                Rtf::Bc4 | Rtf::Bc5 | Rtf::Bc7 | Rtf::Astc4x4 => 4,
            };

            let bytes_per_pixel = match self.texture.format {
                Rtf::R | Rtf::Rb | Rtf::Rgba => 1,
                Rtf::Bc4 => 8,
                Rtf::Bc5 | Rtf::Bc7 | Rtf::Astc4x4 => 16,
            };

            let height = self.texture.height.into();
            let block_height = block_height_mip0(div_round_up(height, block_size));

            deswizzle_block_linear(
                div_round_up(self.texture.width.into(), block_size),
                div_round_up(self.texture.height.into(), block_size),
                1,
                &tex_data,
                block_height,
                bytes_per_pixel,
            )?
        } else {
            tex_data
        };

        Ok(tex_data)
    }

    /// Gets the raw bytes of a texture. Takes an argument of the resource file.
    /// # Errors
    /// - Encounters texture data that isn't Zlib deflated
    /// - Deswizzling texture data fails
    /// - Texture decompression fails
    /// # Panics
    /// - If texture data is empty
    #[cfg(feature = "draw")]
    pub fn get_uncompressed_bytes(&self, file: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let normalize_textures = false;

        if self.texture.width == 0 || self.texture.height == 0 {
            return Ok(None);
        }

        let tex_data = self.get_texture_bytes(file)?;
        assert!(!tex_data.is_empty());

        let mut tex_data_decoded =
            vec![0; (u32::from(self.texture.width) * u32::from(self.texture.height)) as usize];
        match self.texture.format {
            ResourceTextureFormat::Bc7 => {
                texture2ddecoder::decode_bc7(
                    &tex_data,
                    self.texture.width.into(),
                    self.texture.height.into(),
                    &mut tex_data_decoded,
                )?;
            }
            ResourceTextureFormat::Bc4 => {
                texture2ddecoder::decode_bc4(
                    &tex_data,
                    self.texture.width.into(),
                    self.texture.height.into(),
                    &mut tex_data_decoded,
                )?;

                // Convert R to Rgba
                if normalize_textures {
                    tex_data_decoded = tex_data_decoded
                        .iter()
                        .map(|x| {
                            let [_, _, w, _] = x.to_le_bytes();
                            u32::from_le_bytes([w, w, w, w])
                        })
                        .collect();
                }
            }
            ResourceTextureFormat::Bc5 => {
                texture2ddecoder::decode_bc5(
                    &tex_data,
                    self.texture.width.into(),
                    self.texture.height.into(),
                    &mut tex_data_decoded,
                )?;

                // Convert Rb to Rgba
                if normalize_textures {
                    tex_data_decoded = tex_data_decoded
                        .iter()
                        .map(|x| {
                            let [w, a, _, _] = x.to_le_bytes();
                            u32::from_le_bytes([w, w, w, a])
                        })
                        .collect();
                }
            }
            ResourceTextureFormat::R => {
                tex_data_decoded = tex_data
                    .iter()
                    .map(|x| u32::from_le_bytes([*x, *x, *x, 255]))
                    .collect();
            }
            ResourceTextureFormat::Astc4x4 => {
                texture2ddecoder::decode_astc_4_4(
                    &tex_data,
                    self.texture.width.into(),
                    self.texture.height.into(),
                    &mut tex_data_decoded,
                )?;
            }
            e => unimplemented!("{e:?} is not decoded yet"),
        }

        // U32 to 4x U8
        let tex_data_decoded: Vec<u8> = tex_data_decoded
            .into_iter()
            .flat_map(<u32>::to_le_bytes)
            // .flat_map(|x| {
            //     let [b, g, r, a] = x.to_le_bytes();
            //     [r, g, b, a]
            // })
            .collect();

        Ok(Some(tex_data_decoded))
    }

    /// Gets the raw bytes of a texture. Takes an argument of the resource file.
    /// # Errors
    /// - Encounters texture data that isn't Zlib deflated
    /// - Deswizzling texture data fails
    /// - Texture decompression fails
    /// # Panics
    /// - If texture data is empty
    /// - If calculated image container is too small
    #[cfg(feature = "draw")]
    pub fn get_image(&self, bytes: &[u8]) -> Result<Option<RgbaImage>, Box<dyn Error>> {
        let bytes = match self.get_uncompressed_bytes(bytes) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = RgbaImage::from_raw(
            self.texture.width.into(),
            self.texture.height.into(),
            bytes,
        )
        .unwrap();

        Ok(Some(img))
    }
}

#[derive(BinRead, Clone, Copy)]
#[br(little, magic = b"NFTR")]
pub struct ResourceTexture {
    ver: u32,
    file_size: u32,
    max_size: [u32; 11],
    max_alignment: [u32; 11],
    pub hat: [TextureElement; 132],
    pub eye: [TextureElement; 62],
    pub eyebrow: [TextureElement; 24],
    pub beard: [TextureElement; 2],
    pub wrinkle: [TextureElement; 12],
    pub makeup: [TextureElement; 12],
    pub glass: [TextureElement; 20],
    pub mole: [TextureElement; 2],
    pub mouth: [TextureElement; 37],
    pub mustache: [TextureElement; 6],
    pub noseline: [TextureElement; 18],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn read() -> R {
        let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

        let _ = ResourceTexture::read(&mut bin)?;

        Ok(())
    }

    // #[test]
    // #[cfg(feature = "draw")]
    // fn eye_tex() -> R {
    //     let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

    //     let res = ResourceTexture::read(&mut bin)?;

    //     let res = res.eye[0];

    //     let tex = res.get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;

    //     if let Some(tex) = tex {
    //         tex.save(concat!(env!("CARGO_MANIFEST_DIR"), "/test_output/eye.png"))?;
    //     }
    //     Ok(())
    // }

    // #[test]
    // #[cfg(feature = "draw")]
    // fn face_tex() -> R {
    //     let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

    //     let res = ResourceTexture::read(&mut bin)?;

    //     // let res = res.makeup[1];

    //     let tex =
    //         res.makeup[1].get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;

    //     if let Some(tex) = tex {
    //         tex.save(concat!(
    //             env!("CARGO_MANIFEST_DIR"),
    //             "/test_output/makeup.png"
    //         ))?;
    //         // tex.save("./tex.png")?;
    //     }

    //     // let res = ResourceTexture::read(&mut bin)?;
    //     let tex =
    //         res.noseline[1].get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;

    //     if let Some(tex) = tex {
    //         tex.save(concat!(
    //             env!("CARGO_MANIFEST_DIR"),
    //             "/test_output/noseline.png"
    //         ))?;
    //     }
    //     Ok(())
    // }
}
