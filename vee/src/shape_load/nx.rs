use crate::utils::{ReadSeek, Vec3PackedSnorm, inflate_bytes, read_byte_slice, u16_to_f32};
use binrw::{BinRead, Endian};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
};

#[cfg(feature = "gltf")]
use mesh_tools::GltfBuilder;

enum AttributeType {
    /// Vertex positions. Format: `AttributeFormat_16_16_16_16_Float`
    Position = 0, // `AttributeFormat_16_16_16_16_Float`
    Normal = 1,  // `AttributeFormat_10_10_10_2_Snorm`
    Uv = 2,      // `AttributeFormat_16_16_Float`
    Tangent = 3, // `AttributeFormat_8_8_8_8_Snorm`
    Param = 4,   // `AttributeFormat_8_8_8_8_Unorm`
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
    #[cfg(feature = "gltf")]
    fn gltf(&self, _bounding_box: [[f32; 3]; 2]) -> GltfBuilder {
        let mut builder = GltfBuilder::new();

        let ShapeData {
            positions,
            indices,
            normals: _,
            uvs: _,
            color_params: _,
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

        builder
    }
}

impl BinRead for ShapeData {
    type Args<'a> = ResourceShapeAttribute;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        const PER_VERTEX_SIZE: u32 = 8;
        const PER_INDEX_SIZE: u32 = 2;

        // let offset = u32::read_options(reader, endian, ())?;
        // let saved_position = reader.stream_position()?;

        // Read positions
        reader.seek(SeekFrom::Start(u64::from(
            args.attr_offset[AttributeType::Position as usize],
        )))?;

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
        reader.seek(SeekFrom::Start(u64::from(args.index_offset)))?;

        let index_count = args.index_size / PER_INDEX_SIZE;

        let mut indices = vec![];
        for _index in 0..index_count {
            let value = <u16>::read_options(reader, endian, ())?;
            indices.push(value);
        }

        // Read normals
        let normals = if args.is_valid_attribute(AttributeType::Normal) {
            reader.seek(SeekFrom::Start(u64::from(
                args.attr_offset[AttributeType::Normal as usize],
            )))?;

            let mut normals = vec![];
            for _vertex in 0..vertex_count {
                let packed = <u32>::read_options(reader, endian, ())?;
                normals.push(Vec3PackedSnorm(packed).unpack());
            }

            Some(normals)
        } else {
            None
        };

        // Read UVs
        let uvs = if args.is_valid_attribute(AttributeType::Uv) {
            reader.seek(SeekFrom::Start(u64::from(
                args.attr_offset[AttributeType::Uv as usize],
            )))?;

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
            reader.seek(SeekFrom::Start(u64::from(
                args.attr_offset[AttributeType::Param as usize],
            )))?;

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

impl ShapeElement {
    /// # Errors
    /// Can error if:
    /// - Shape data is in a malformed zlib format
    /// - Writing out shape data file errors
    /// - Parsing vertices and etc from data fails
    pub fn shape_data(&mut self, file: &mut dyn ReadSeek) -> Result<ShapeData, Box<dyn Error>> {
        // exporter set boundingbox

        let shape_data = read_byte_slice(
            file,
            self.common.offset.into(),
            self.common.size.try_into()?,
        )?;

        let shape_data = inflate_bytes(&shape_data)?;

        std::fs::write("./shape.dat", shape_data.clone())?;

        let data =
            ShapeData::read_options(&mut Cursor::new(shape_data), Endian::Little, self.shape)?;

        Ok(data)
    }

    /// # Errors
    /// Can error if:
    /// - Shape data cannot be parsed
    #[cfg(feature = "gltf")]
    pub fn gltf(&mut self, file: &mut File) -> Result<GltfBuilder, Box<dyn Error>> {
        let data = self.shape_data(file)?;

        Ok(data.gltf(self.shape.bounding_box))
    }
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

#[derive(Clone, Copy)]
pub enum Shape {
    Beard,
    FaceLine,
    Mask,
    HatNormal,
    HatCap,
    ForeheadNormal,
    ForeheadCap,
    HairNormal,
    HairCap,
    Glasses,
    Nose,
    NoseLine,
    HairTransform,
    FaceLineTransform,
}
#[derive(Clone, Copy)]
pub enum GenericResourceShape {
    Element(ShapeElement),
    HairTransform(ResourceShapeHairTransform),
    FaceLineTransform(ResourceShapeFacelineTransform),
}

#[derive(BinRead, Debug, Clone, Copy)]
#[br(little, magic = b"NFSR")]
pub struct ResourceShape {
    ver: u32,
    file_size: u32,
    max_size: [u32; 12],
    max_alignment: [u32; 12],
    pub beard: [ShapeElement; 4],
    pub face_line: [ShapeElement; 12],
    pub mask: [ShapeElement; 12],
    pub hat_normal: [ShapeElement; 132],
    pub hat_cap: [ShapeElement; 132],
    pub forehead_normal: [ShapeElement; 132],
    pub forehead_cap: [ShapeElement; 132],
    pub hair_normal: [ShapeElement; 132],
    pub hair_cap: [ShapeElement; 132],

    pub glasses: [ShapeElement; 1],

    pub nose: [ShapeElement; 18],
    pub nose_line: [ShapeElement; 18],

    pub hair_transform: [ResourceShapeHairTransform; 132],
    pub face_line_transform: [ResourceShapeFacelineTransform; 12],
}

impl ResourceShape {
    #[allow(clippy::must_use_candidate)]
    pub fn fetch_shape(&self, shape: Shape, index: usize) -> Option<GenericResourceShape> {
        let shape_el = |x: &ShapeElement| GenericResourceShape::Element(*x);
        let hair_t = |x: &ResourceShapeHairTransform| GenericResourceShape::HairTransform(*x);
        let fl_t = |x: &ResourceShapeFacelineTransform| GenericResourceShape::FaceLineTransform(*x);

        match shape {
            Shape::Beard => self.beard.get(index).map(shape_el),
            Shape::FaceLine => self.face_line.get(index).map(shape_el),
            Shape::Mask => self.mask.get(index).map(shape_el),
            Shape::HatNormal => self.hat_normal.get(index).map(shape_el),
            Shape::HatCap => self.hat_cap.get(index).map(shape_el),
            Shape::ForeheadNormal => self.forehead_normal.get(index).map(shape_el),
            Shape::ForeheadCap => self.forehead_cap.get(index).map(shape_el),
            Shape::HairNormal => self.hair_normal.get(index).map(shape_el),
            Shape::HairCap => self.hair_cap.get(index).map(shape_el),
            Shape::Glasses => self.glasses.get(index).map(shape_el),
            Shape::Nose => self.nose.get(index).map(shape_el),
            Shape::NoseLine => self.nose_line.get(index).map(shape_el),
            Shape::HairTransform => self.hair_transform.get(index).map(hair_t),
            Shape::FaceLineTransform => self.face_line_transform.get(index).map(fl_t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use half::f16;
    use std::{error::Error, fs::File, io::BufReader};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn read() -> R {
        let mut bin = BufReader::new(File::open("ShapeMid.dat")?);

        let _ = ResourceShape::read(&mut bin)?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "gltf")]
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
    fn u16_to_f16_to_f32() {
        let within_tolerance = (1.0 - f16::from_bits(15360).to_f32()).abs() < 1.0;
        assert!(within_tolerance);
    }
}
