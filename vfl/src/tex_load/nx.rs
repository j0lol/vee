use std::error::Error;

use crate::{
    shape_load::nx::ResourceCommonAttribute,
    utils::{ReadSeek, inflate_bytes, read_byte_slice},
};
use binrw::BinRead;
use image::{ImageBuffer, Rgba, RgbaImage};

pub const TEXTURE_MID_SRGB_DAT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/NXTextureMidSRGB.dat");

#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceTextureAttribute {
    alignment: u32,
    pub width: u16,
    pub height: u16,
    format: u8,
    mip_count: u8,
    tile_mode: u8,
    pad: [u8; 1],
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct TextureElement {
    pub common: ResourceCommonAttribute,
    pub texture: ResourceTextureAttribute,
}
use strum_macros::FromRepr;
use tegra_swizzle::{block_height_mip0, div_round_up, swizzle::deswizzle_block_linear};

// Format rundown:
// https://www.reedbeta.com/blog/understanding-bcn-texture-compression-formats/#comparison-table
#[derive(FromRepr, Debug)]
enum ResourceTextureFormat {
    R = 0,       // R8Unorm (Ffl Name)
    Rb = 1,      // R8B8Unorm
    Rgba = 2,    // R8B8G8A8Unorm
    Bc4 = 3,     // Bc4Unorm (Compressed R)
    Bc5 = 4,     // Bc5Unorm (Compressed Rb)
    Bc7 = 5,     // Bc7Unorm (Compressed Rgba)
    Astc4x4 = 6, // Astc4x4Unorm (Compressed Rgba)
}

impl TextureElement {
    pub fn get_texture_bytes(&self, file: &mut dyn ReadSeek) -> Result<Vec<u8>, Box<dyn Error>> {
        let tex_data = read_byte_slice(
            file,
            self.common.offset.into(),
            self.common.size_compressed.try_into()?,
        )?;

        let tex_data = inflate_bytes(&tex_data)?;

        let needs_swizzling = self.texture.tile_mode == 0;

        let tex_data = if needs_swizzling {
            let block_size =
                match ResourceTextureFormat::from_repr(self.texture.format as usize).unwrap() {
                    ResourceTextureFormat::R
                    | ResourceTextureFormat::Rb
                    | ResourceTextureFormat::Rgba => 1,
                    ResourceTextureFormat::Bc4
                    | ResourceTextureFormat::Bc5
                    | ResourceTextureFormat::Bc7
                    | ResourceTextureFormat::Astc4x4 => 4,
                };

            let bytes_per_pixel =
                match ResourceTextureFormat::from_repr(self.texture.format as usize).unwrap() {
                    ResourceTextureFormat::R
                    | ResourceTextureFormat::Rb
                    | ResourceTextureFormat::Rgba => 1,
                    ResourceTextureFormat::Bc4 => 8,
                    ResourceTextureFormat::Bc5
                    | ResourceTextureFormat::Bc7
                    | ResourceTextureFormat::Astc4x4 => 16,
                };

            let height = self.texture.height.into();
            let block_height = block_height_mip0(div_round_up(height, block_size));

            // panic!("{}", tex_data.len());
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

    pub fn get_uncompressed_bytes(
        &self,
        file: &mut dyn ReadSeek,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        if self.texture.width == 0 || self.texture.height == 0 {
            return Ok(None);
        }

        let tex_data = self.get_texture_bytes(file)?;
        assert!(!tex_data.is_empty());

        let mut tex_data_decoded =
            vec![0; (u32::from(self.texture.width) * u32::from(self.texture.height)) as usize];
        match ResourceTextureFormat::from_repr(self.texture.format as usize).unwrap() {
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
                tex_data_decoded = tex_data_decoded
                    .iter()
                    .map(|x| {
                        let [_, _, w, _] = x.to_le_bytes();
                        u32::from_le_bytes([w, w, w, w])
                    })
                    .collect();
            }
            ResourceTextureFormat::Bc5 => {
                texture2ddecoder::decode_bc5(
                    &tex_data,
                    self.texture.width.into(),
                    self.texture.height.into(),
                    &mut tex_data_decoded,
                )?;

                // Convert Rb to Rgba
                tex_data_decoded = tex_data_decoded
                    .iter()
                    .map(|x| {
                        let [w, a, _, _] = x.to_le_bytes();
                        u32::from_le_bytes([w, w, w, a])
                    })
                    .collect();
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
            .iter()
            .flat_map(|x| {
                let [b, g, r, a] = x.to_le_bytes();
                [r, g, b, a]
            })
            .collect();

        Ok(Some(tex_data_decoded))
    }
    pub fn get_image(&self, file: &mut dyn ReadSeek) -> Result<Option<RgbaImage>, Box<dyn Error>> {
        let bytes = match self.get_uncompressed_bytes(file) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = image::RgbaImage::from_raw(
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

    #[test]
    fn eye_tex() -> R {
        let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

        let res = ResourceTexture::read(&mut bin)?;

        let res = res.eye[0];

        let tex = res.get_image(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))?;

        if let Some(tex) = tex {
            tex.save("./tex.png")?;
        }
        Ok(())
    }
}
