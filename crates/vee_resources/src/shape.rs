//! Parsing mesh data
use crate::inflate_bytes;
use crate::packing::{Float16, Vec3PackedSnorm};
use binrw::{BinRead, Endian};
use num_enum::TryFromPrimitive;
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
};
// #[cfg(feature = "gltf")]
// use mesh_tools::GltfBuilder;

#[allow(unused)]
enum AttributeType {
    Position = 0, // `AttributeFormat_16_16_16_16_Float`
    Normal = 1,   // `AttributeFormat_10_10_10_2_Snorm`
    Uv = 2,       // `AttributeFormat_16_16_Float`
    Tangent = 3,  // `AttributeFormat_8_8_8_8_Snorm`
    Param = 4,    // `AttributeFormat_8_8_8_8_Unorm`
}

/// Mesh of a shape, fresh from the resource file.
/// Gets turned into a `vee_models::GenericModel3D` and paired with a texture.
#[derive(Debug, Clone)]
pub struct ShapeMesh {
    pub positions: Vec<[Float16; 4]>,
    pub indices: Vec<u16>,
    pub normals: Option<Vec<Vec3PackedSnorm>>,
    pub uvs: Option<Vec<[Float16; 2]>>,
    pub color_params: Option<Vec<u8>>,
}
// TODO: add gltf?
// impl ShapeData {
//     #[cfg(feature = "gltf")]
//     fn gltf(&self, _bounding_box: [[f32; 3]; 2]) -> GltfBuilder {
//         let mut builder = GltfBuilder::new();
//
//         let ShapeData {
//             positions,
//             indices,
//             normals: _,
//             uvs: _,
//             color_params: _,
//         } = self;
//
//         let material = builder.create_basic_material(
//             Some("Hair texture".to_string()),
//             [0.118, 0.102, 0.094, 1.000],
//         );
//
//         let mesh = builder.create_simple_mesh(
//             Some("Mii Shape".to_string()),
//             positions.as_flattened(),
//             indices,
//             // normals.clone().map(|v| v.into_flattened()).as_deref(),
//             // uvs.clone().map(|v| v.into_flattened()).as_deref(),
//             None,
//             None,
//             Some(material),
//         );
//
//         let mii_shape_node = builder.add_node(
//             Some("Mii Node".to_string()),
//             Some(mesh),
//             Some([0.0, 0.0, 0.0]),
//             None,
//             None,
//         );
//
//         builder.add_scene(Some("Mii Scene".to_string()), Some(vec![mii_shape_node]));
//
//         builder
//     }
// }

impl BinRead for ShapeMesh {
    type Args<'a> = ResourceShapeAttribute;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
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

        // Pre-size the Vec
        let mut positions = Vec::with_capacity(usize::try_from(vertex_count).unwrap());

        for _vertex in 0..vertex_count {
            positions.push(<[Float16; 4]>::read_options(reader, endian, ())?);
        }

        // Read indices
        reader.seek(SeekFrom::Start(u64::from(args.index_offset)))?;

        let index_count = args.index_size / PER_INDEX_SIZE;

        let mut indices = Vec::with_capacity(usize::try_from(index_count).unwrap());
        for _index in 0..index_count {
            indices.push(<u16>::read_options(reader, endian, ())?);
        }

        // Read normals
        let normals = if args.is_valid_attribute(AttributeType::Normal) {
            reader.seek(SeekFrom::Start(u64::from(
                args.attr_offset[AttributeType::Normal as usize],
            )))?;

            let mut normals = Vec::with_capacity(usize::try_from(vertex_count).unwrap());
            for _vertex in 0..vertex_count {
                normals.push(<Vec3PackedSnorm>::read_options(reader, endian, ())?);
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

            let mut uvs = Vec::with_capacity(usize::try_from(vertex_count).unwrap());
            for _vertex in 0..vertex_count {
                uvs.push(<[Float16; 2]>::read_options(reader, endian, ())?);
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

            let mut color_params = Vec::with_capacity(usize::try_from(color_count).unwrap());
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

        Ok(ShapeMesh {
            positions,
            indices,
            normals,
            uvs,
            color_params,
        })
    }
}

/// Specifies where data is, now big it is, and how compressed it is.
/// Used for both textures and shapes.
#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceCommonAttribute {
    pub offset: u32,
    pub size: u32,
    pub size_compressed: u32,
    pub compression_level: u8,
    pub memory_level: u8,
    _pad: u16,
}

/// Specifies where {vertex,index} buffers are, and how big they are.
#[derive(BinRead, Default, Debug, Clone, Copy)]
pub struct ResourceShapeAttribute {
    pub attr_offset: [u32; 5],
    pub attr_size: [u32; 5],
    pub index_offset: u32,
    pub index_size: u32,
    pub bounding_box: [[f32; 3]; 2],
}
impl ResourceShapeAttribute {
    fn is_valid_attribute(&self, attr_type: AttributeType) -> bool {
        // assert!(attr_type as usize < AttributeType::End as usize);
        self.attr_size[attr_type as usize] != 0
    }
}

/// All the data required to read a mesh from the shape file.
/// Essentially a 'pointer' to the data.
#[derive(BinRead, Debug, Clone, Copy)]
pub struct ShapeElement {
    pub common: ResourceCommonAttribute,
    pub shape: ResourceShapeAttribute,
}

impl ShapeElement {
    /// Reads a mesh from the data in `ShapeElement`
    /// # Errors
    /// Can error if:
    /// - Shape data is in a malformed zlib format
    /// - Writing out shape data file errors
    /// - Parsing vertices etc. from data fails
    pub fn mesh(&mut self, file: &[u8]) -> Result<ShapeMesh, Box<dyn Error>> {
        // exporter set boundingbox

        // println!("shapeload");
        let start: usize = self.common.offset as usize;
        let end: usize = self.common.offset as usize + self.common.size_compressed as usize;

        let range = start..end;

        let shape_data = inflate_bytes(&file[range])?;

        // if !cfg!(target_family = "wasm") {
        //     std::fs::write("./shape.dat", shape_data.clone())?;
        // }

        let data =
            ShapeMesh::read_options(&mut Cursor::new(shape_data), Endian::Little, self.shape)?;

        Ok(data)
    }
}

/// Contains positional data for any headwear that
/// may be placed on the `CharModel` post-render.
#[expect(unused)]
#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceShapeHairTransform {
    front_translate: [f32; 3],
    front_rotate: [f32; 3],
    side_translate: [f32; 3],
    side_rotate: [f32; 3],
    top_translate: [f32; 3],
    top_rotate: [f32; 3],
}

/// Contains positional data used to move face parts
/// like the beard, nose, hair, glasses, etc.
#[derive(BinRead, Debug, Clone, Copy)]
pub struct ResourceShapeFacelineTransform {
    pub hair_translate: [f32; 3],
    pub nose_translate: [f32; 3],
    pub beard_translate: [f32; 3],
}

/// Every type of shape mesh stored in the resource data.
#[derive(Clone, Copy, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Shape {
    Beard = 0,
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

/// Generic 'pointer' in the resource data.
#[derive(Clone, Copy)]
pub enum GenericResourceShape {
    Element(ShapeElement),
    HairTransform(ResourceShapeHairTransform),
    FaceLineTransform(ResourceShapeFacelineTransform),
}

/// Header of the `Shape` resource file. Contains model data for `CharModel`s.
#[allow(unused)]
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
    pub fn index_by_shape(&self, shape: Shape, index: usize) -> Option<GenericResourceShape> {
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
        let mut bin = BufReader::new(File::open(format!(
            "{}/resources_here/ShapeMid.dat",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap()
        ))?);

        let _ = ResourceShape::read(&mut bin)?;

        Ok(())
    }

    // #[test]
    // #[cfg(feature = "gltf")]
    // fn jas() -> R {
    //     let mut bin = BufReader::new(File::open(SHAPE_MID_DAT)?);

    //     let res = ResourceShape::read(&mut bin)?;

    //     let mut shape = res.hair_normal[123];

    //     let mut file = File::open(SHAPE_MID_DAT)?;

    //     let gltf = shape.gltf(&mut file)?;
    //     gltf.export_glb(concat!(env!("CARGO_MANIFEST_DIR"), "/test_output/jas.glb"))?;

    //     Ok(())
    // }

    #[test]
    fn u16_to_f16_to_f32() {
        let within_tolerance = (1.0 - f16::from_bits(15360).to_f32()).abs() < 1.0;
        assert!(within_tolerance);
    }
}
