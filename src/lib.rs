mod utils;

use binrw::{BinRead, Endian};
use mesh_tools::GltfBuilder;
use std::{
    error::Error,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};
use utils::{decode_reader, inflate_bytes, read_file_slice, u16_to_f32, vec3_packed_snorm};

enum AttributeType {
    /// Vertex positions. Format: AttributeFormat_16_16_16_16_Float
    Position = 0, // AttributeFormat_16_16_16_16_Float
    Normal = 1,  // AttributeFormat_10_10_10_2_Snorm
    Uv = 2,      // AttributeFormat_16_16_Float
    Tangent = 3, // AttributeFormat_8_8_8_8_Snorm
    Param = 4,   // AttributeFormat_8_8_8_8_Unorm
    End = 5,
}

#[derive(Debug, Clone)]
pub struct ShapeData {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub color_params: Option<Vec<u8>>,
}
impl ShapeData {
    fn gltf(&self, bounding_box: [[f32; 3]; 2]) -> Result<GltfBuilder, Box<dyn Error>> {
        let mut builder = GltfBuilder::new();

        let ShapeData {
            positions,
            indices,
            normals,
            uvs,
            color_params,
        } = self;

        let material = builder.create_basic_material(
            Some("Hair texture".to_string()),
            [0.118, 0.102, 0.094, 1.000],
        );

        let mesh = builder.create_simple_mesh(
            Some("Mii Shape".to_string()),
            positions.as_flattened(),
            indices,
            // normals.clone().map(|v| v.into_flattened()).as_deref(),
            // uvs.clone().map(|v| v.into_flattened()).as_deref(),
            None,
            None,
            Some(material),
        );

        let mii_shape_node = builder.add_node(
            Some("Mii Node".to_string()),
            Some(mesh),
            Some([0.0, 0.0, 0.0]),
            None,
            None,
        );

        builder.add_scene(Some("Mii Scene".to_string()), Some(vec![mii_shape_node]));

        Ok(builder)
    }
}

impl BinRead for ShapeData {
    type Args<'a> = ResourceShapeAttribute;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let offset = u32::read_options(reader, endian, ())?;
        let saved_position = reader.stream_position()?;

        // Read positions
        reader.seek(SeekFrom::Start(
            args.attr_offset[AttributeType::Position as usize] as u64,
        ))?;

        const PER_VERTEX_SIZE: u32 = 8;
        let vertex_count = args.attr_size[AttributeType::Position as usize] / PER_VERTEX_SIZE;

        let mut positions = vec![];
        for _vertex in 0..vertex_count {
            positions.push({
                let positions = <[u16; 3]>::read_options(reader, endian, ())?;
                let _ = <u16>::read_options(reader, endian, ())?;

                // Skip 2 bytes for padding
                // let _ = reader.take(2);

                positions.map(u16_to_f32)
            });
        }

        // Read indices
        reader.seek(SeekFrom::Start(args.index_offset as u64))?;

        const PER_INDEX_SIZE: u32 = 2;
        let index_count = args.index_size / PER_INDEX_SIZE;

        let mut indices = vec![];
        for _index in 0..index_count {
            let value = <u16>::read_options(reader, endian, ())?;
            indices.push(value);
        }

        // Read normals
        let normals = if args.is_valid_attribute(AttributeType::Normal) {
            reader.seek(SeekFrom::Start(
                args.attr_offset[AttributeType::Normal as usize] as u64,
            ))?;

            let mut normals = vec![];
            for _vertex in 0..vertex_count {
                let packed = <u32>::read_options(reader, endian, ())?;
                normals.push(vec3_packed_snorm(packed));
            }

            Some(normals)
        } else {
            None
        };

        // Read UVs
        let uvs = if args.is_valid_attribute(AttributeType::Uv) {
            reader.seek(SeekFrom::Start(
                args.attr_offset[AttributeType::Uv as usize] as u64,
            ))?;

            let mut uvs = vec![];
            for _vertex in 0..vertex_count {
                uvs.push(<[u16; 2]>::read_options(reader, endian, ())?.map(u16_to_f32));
            }

            Some(uvs)
        } else {
            None
        };

        // Read Params (Colors)
        let color_params = if args.is_valid_attribute(AttributeType::Param) {
            reader.seek(SeekFrom::Start(
                args.attr_offset[AttributeType::Param as usize] as u64,
            ))?;

            let color_count = args.attr_size[AttributeType::Param as usize] / PER_VERTEX_SIZE;

            let mut color_params = vec![];
            for _color in 0..color_count {
                color_params.push(<u8>::read_options(reader, endian, ())?);
            }

            Some(color_params)
        } else {
            None
        };

        // // Read from an offset with a provided base offset.
        // reader.seek(SeekFrom::Start(args + offset as u64))?;
        // let value = u16::read_options(reader, endian, ())?;

        // reader.seek(SeekFrom::Start(saved_position))?;

        Ok(ShapeData {
            positions,
            indices,
            normals,
            uvs,
            color_params,
        })
    }
}
#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceCommonAttribute {
    offset: u32,
    size: u32,
    size_compressed: u32,
    compression_level: u8,
    memory_level: u8,
    pad: u16,
}
#[derive(BinRead, Default, Debug, Clone, Copy)]
pub struct ResourceShapeAttribute {
    attr_offset: [u32; 5],
    attr_size: [u32; 5],
    index_offset: u32,
    index_size: u32,
    bounding_box: [[f32; 3]; 2],
}
impl ResourceShapeAttribute {
    fn is_valid_attribute(&self, attr_type: AttributeType) -> bool {
        // assert!(attr_type as usize < AttributeType::End as usize);
        self.attr_size[attr_type as usize] != 0
    }
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct ShapeElement {
    common: ResourceCommonAttribute,
    shape: ResourceShapeAttribute,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceShapeHairTransform {
    front_translate: [f32; 3],
    front_rotate: [f32; 3],
    side_translate: [f32; 3],
    side_rotate: [f32; 3],
    top_translate: [f32; 3],
    top_rotate: [f32; 3],
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceShapeFacelineTransform {
    hair_translate: [f32; 3],
    nose_translate: [f32; 3],
    beard_translate: [f32; 3],
}

#[derive(BinRead, Debug, Clone, Copy)]
#[br(little, magic = b"NFSR")]
pub struct ResourceShape {
    ver: u32,
    file_size: u32,
    max_size: [u32; 12],
    max_alignment: [u32; 12],
    beard: [ShapeElement; 4],
    face_line: [ShapeElement; 12],
    mask: [ShapeElement; 12],
    hat_normal: [ShapeElement; 132],
    hat_cap: [ShapeElement; 132],
    forehead_normal: [ShapeElement; 132],
    forehead_cap: [ShapeElement; 132],
    pub hair_normal: [ShapeElement; 132],
    hair_cap: [ShapeElement; 132],

    glasses: [ShapeElement; 1],

    nose: [ShapeElement; 18],
    nose_line: [ShapeElement; 18],

    hair_transform: [ResourceShapeHairTransform; 132],
    face_line_transform: [ResourceShapeFacelineTransform; 12],
}
impl ShapeElement {
    pub fn shape_data(&mut self, file: &mut File) -> Result<ShapeData, Box<dyn Error>> {
        // exporter set boundingbox

        let shape_data = read_file_slice(
            file,
            self.common.offset.into(),
            self.common.size.try_into()?,
        )?;

        let shape_data = inflate_bytes(shape_data)?;

        std::fs::write("./shape.dat", shape_data.clone()).unwrap();

        let data =
            ShapeData::read_options(&mut Cursor::new(shape_data), Endian::Little, self.shape)?;

        Ok(data)
    }

    pub fn gltf(&mut self, file: &mut File) -> Result<GltfBuilder, Box<dyn Error>> {
        let min = self.shape.bounding_box[0];
        let max = self.shape.bounding_box[1];

        let data = self.shape_data(file)?;

        data.gltf(self.shape.bounding_box)
    }
}

// impl ResourceShape {
//     pub fn decompress_resource(&self, data: &[u8]) {}
//     // pub fn gltf(self) {
//     //     todo!();
//     //     // let min = self.
//     // }
// }

#[cfg(test)]
mod tests {
    use std::io::{self, Read, Write};
    use std::{error::Error, fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress};
    use half::f16;
    use utils::{decode_reader, read_file_slice};

    use super::*;

    #[test]
    fn read() -> R {
        let mut bin = BufReader::new(File::open("ShapeMid.dat")?);

        let _ = ResourceShape::read(&mut bin)?;

        Ok(())
    }

    #[test]
    fn jas() -> R {
        let mut bin = BufReader::new(File::open("ShapeMid.dat")?);

        let res = ResourceShape::read(&mut bin)?;

        let mut shape = res.hair_normal[123];

        let mut file = File::open("ShapeMid.dat")?;

        let gltf = shape.gltf(&mut file)?;
        gltf.export_glb("jas.glb")?;

        Ok(())
    }

    #[test]
    fn u16_to_f16_to_f32() -> R {
        assert_eq!(1.0, f16::from_bits(15360).to_f32());

        Ok(())
    }

    #[test]
    fn inflate() -> R {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"Hello World").unwrap();
        let bytes = e.finish().unwrap();

        // Uncompresses a Zlib Encoded vector of bytes and returns a string or error
        // Here &[u8] implements Read

        assert_eq!(decode_reader(bytes).unwrap(), "Hello World");

        Ok(())
    }
}
