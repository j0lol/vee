use std::{
    error::Error,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use flate2::read::ZlibDecoder;

pub fn u16_to_f32(num: u16) -> f32 {
    half::f16::from_bits(num).to_f32()
}

/// Vec3 packed in a u32:
/// [10; 10; 10; 2]
/// The w component is discarded.
pub fn vec3_packed_snorm(packed: u32) -> [f32; 3] {
    let nx = (packed << 22) as i32 >> 22;
    let ny = (packed << 12) as i32 >> 22;
    let nz = (packed << 2) as i32 >> 22;
    let ret = [nx as f32 / 511.0, ny as f32 / 511.0, nz as f32 / 511.0];

    // assert!(
    //     glam::Vec3::from_array(ret).is_normalized(),
    //     "{}, close to {}",
    //     glam::Vec3::from_array(ret),
    //     glam::Vec3::from_array(ret).normalize()
    // );

    // println!("{ret:?}");

    // glam::Vec3::from_array(ret).normalize().to_array()

    ret
}

pub fn read_file_slice(
    file: &mut File,
    start: u64,
    count: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    file.seek(SeekFrom::Start(start))?;
    let mut buf = vec![0; count];
    file.read_exact(&mut buf)?;

    Ok(buf)
}

pub fn inflate_bytes(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    // z.read_to_string(&mut s)?;
    let mut vec = Vec::new();
    z.read_to_end(&mut vec)?;
    Ok(vec)
}

pub fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use crate::utils::vec3_packed_snorm;

    #[test]
    fn vec3_packed_snorm_test() {
        for test in [0x567a67ca, 0x567a6436, 0x56759fca, 0x56759c36] {
            vec3_packed_snorm(test).iter().for_each(|x| {
                if x.abs() > 1.0 {
                    panic!("Not normalised!")
                }
            });
        }
    }
}
