//! Library to parse Mii resource data. Currently only supports Nx shape and texture files.
pub use half::f16 as half_f16;
use std::io;

pub mod color;
pub mod packing;
pub mod shape;
pub mod tex;

pub(crate) fn inflate_bytes(bytes: &[u8]) -> io::Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut z = ZlibDecoder::new(bytes);
    let mut vec = Vec::new();
    z.read_to_end(&mut vec)?;
    Ok(vec)
}

mod cafe {
    use vee_parse::BinRead;

    /// f_f_li_resource_header
    ///
    /// Shape/texture resource format for FFL.
    /// Each buffer is compressed using raw deflate.
    ///
    /// Texture formats are in f_f_li_texture_format (R/RG/RGBA),
    /// shape formats are in (comments next to)
    /// f_f_li_resource_shape_element_type (float/(u)int8/10_10_10_2).
    ///
    /// Shapes contain bounding boxes and textures contain mipmaps.
    /// Other than shapes/textures, this file also contains
    /// transform vectors for each faceline and hair shape.
    #[derive(BinRead, Clone, Copy, Debug)]
    #[br(big, magic = b"FFRA", assert(unknown[11] == 0x0))]
    pub struct CafeResourceHeader {
        version: u32,
        uncompress_buffer_size: u32,
        expanded_buffer_size: u32,
        is_expand: u32,

        texture_data: CafeResourceTextureHeader,
        shape_data: CafeResourceShapeHeader,

        unknown: [u32; 12],
    }

    /// Arch (Miitomo) Resources are notably different during parsing
    /// while being derivative of [CafeResourceHeader].
    #[derive(BinRead, Clone, Copy, Debug)]
    #[br(big, magic = b"FFRA", assert(unknown[11] == 0x0))]
    pub struct ArchResourceHeader {
        version: u32,
        uncompress_buffer_size: u32,
        expanded_buffer_size: u32,
        is_expand: u32,

        texture_data: ArchResourceTextureHeader,
        shape_data: CafeResourceShapeHeader,

        unknown: [u32; 12],
    }

    #[derive(BinRead, Clone, Copy, Debug)]
    pub struct CafeResourceTextureHeader {
        max_size: [u32; 11],
        /// AKA FaceT_beard
        beard: [CafeResourcePartsInfo; 3],
        cap: [CafeResourcePartsInfo; 132],
        eye: [CafeResourcePartsInfo; 62],     // 62/80 FFL/AFL
        eyebrow: [CafeResourcePartsInfo; 24], // 24/28 FFL/AFL
        /// FaceT_line, "wrinkle"
        face_t_line: [CafeResourcePartsInfo; 12],
        /// FaceT_make, make, makeup
        face_t_make: [CafeResourcePartsInfo; 12],
        glass: [CafeResourcePartsInfo; 9], // 9/20 FFL/AFL
        mole: [CafeResourcePartsInfo; 2],
        mouth: [CafeResourcePartsInfo; 37], // 37/52 FFL/AFL
        mustache: [CafeResourcePartsInfo; 6],
        /// Noseline
        nline: [CafeResourcePartsInfo; 18],
    }

    #[derive(BinRead, Clone, Copy, Debug)]
    pub struct ArchResourceTextureHeader {
        max_size: [u32; 11],
        /// AKA FaceT_beard
        beard: [CafeResourcePartsInfo; 3],
        cap: [CafeResourcePartsInfo; 132],
        eye: [CafeResourcePartsInfo; 80],     // 62/80 FFL/AFL
        eyebrow: [CafeResourcePartsInfo; 28], // 24/28 FFL/AFL
        /// FaceT_line, "wrinkle"
        face_t_line: [CafeResourcePartsInfo; 12],
        /// FaceT_make, make, makeup
        face_t_make: [CafeResourcePartsInfo; 12],
        glass: [CafeResourcePartsInfo; 20], // 9/20 FFL/AFL
        mole: [CafeResourcePartsInfo; 2],
        mouth: [CafeResourcePartsInfo; 52], // 37/52 FFL/AFL
        mustache: [CafeResourcePartsInfo; 6],
        /// Noseline
        nline: [CafeResourcePartsInfo; 18],
    }

    #[derive(BinRead, Clone, Copy, Debug)]
    #[br(little)]
    pub struct CafeResourceShapeHeader {
        max_size: [u32; 12],
        beard: [CafeResourcePartsInfo; 4],
        cap_normal: [CafeResourcePartsInfo; 132],
        cap_hat: [CafeResourcePartsInfo; 132],
        faceline: [CafeResourcePartsInfo; 12],
        glass: [CafeResourcePartsInfo; 1],
        mask: [CafeResourcePartsInfo; 12],
        /// Noseline
        nline: [CafeResourcePartsInfo; 18],
        nose: [CafeResourcePartsInfo; 18],
        hair_normal: [CafeResourcePartsInfo; 132],
        /// The hat/cap/headwear variants (as cap/hair/forehead are loaded together)
        /// are for FFL_MODEL_TYPE_HAT, and are meant for the
        /// caller/user to use in conjunction with FFLPartsTransform
        /// to place custom headwear on top of the head model.
        hair_hat: [CafeResourcePartsInfo; 132],
        forehead_normal: [CafeResourcePartsInfo; 132],
        forehead_hat: [CafeResourcePartsInfo; 132],
    }

    ///  f_f_li_resource_parts_info:
    //         doc: |
    //           For verification, see: nn::mii::detail::ResourceCommonAttribute::IsValid()
    //         seq:
    //
    //
    //         instances:
    //           shape_data_header:
    //             io: _root._io
    //             if: size > 0
    //             pos: offset
    //             size: compressed_size # Compressed size.
    //             type: f_f_li_resource_shape_data_header
    //             process: zlib # Deflate will work too.
    //           # Texture footer: Offset is inside DECOMPRESSED block.
    //           # Offset = (decompressed size) - 0x10
    //
    #[derive(BinRead, Clone, Copy, Debug)]
    pub struct CafeResourcePartsInfo {
        offset: u32,
        size: u32,
        compressed_size: u32,
        compress_level: u8,
        window_bits: u8,
        memory_level: u8,
        strategy: u8,
    }

    mod tests {
        use crate::cafe::{ArchResourceHeader, CafeResourceHeader};
        use binrw::BinRead;
        use std::fs::File;
        use std::io::BufReader;

        #[test]
        fn cafe_resources_read() -> Result<(), Box<dyn std::error::Error>> {
            let mut bin = BufReader::new(File::open(format!(
                "{}/resources_here/AFLResHigh_2_3.dat",
                std::env::var("CARGO_WORKSPACE_DIR").unwrap()
            ))?);

            let _ = ArchResourceHeader::read(&mut bin)?;

            Ok(())
        }
    }
}
