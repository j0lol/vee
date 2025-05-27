use std::{
    error::Error,
    io::{self, Read, SeekFrom},
};

pub fn u16_to_f32(num: u16) -> f32 {
    half::f16::from_bits(num).to_f32()
}

/// Signed, normalized Vec3 + w component packed in a u32.
/// [x: 10; y: 10; z: 10; w: 2]
pub struct Vec3PackedSnorm(pub u32);

impl Vec3PackedSnorm {
    /// The w component is discarded.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    pub fn unpack(self) -> [f32; 3] {
        let packed = self.0;

        let nx = (packed << 22) as i32 >> 22;
        let ny = (packed << 12) as i32 >> 22;
        let nz = (packed << 2) as i32 >> 22;

        [nx as f32 / 511.0, ny as f32 / 511.0, nz as f32 / 511.0]
    }
}

/// Abstraction over anything that can be read as Seek or Read
pub trait ReadSeek: io::Seek + Read {}
impl<T: ?Sized> ReadSeek for T where T: io::Seek + Read {}

pub(crate) fn read_byte_slice(
    file: &mut dyn ReadSeek, // Dynamic param: Anything that impl's Read + Seek
    start: u64,
    count: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    file.seek(SeekFrom::Start(start))?;
    let mut buf = vec![0; count];
    file.read_exact(&mut buf)?;

    Ok(buf)
}

#[cfg(feature = "res")]
pub(crate) fn inflate_bytes(bytes: &[u8]) -> io::Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    let mut z = ZlibDecoder::new(bytes);
    // z.read_to_string(&mut s)?;
    let mut vec = Vec::new();
    z.read_to_end(&mut vec)?;
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::Vec3PackedSnorm;

    #[test]
    fn vec3_packed_snorm_test() {
        for test in [0x567a_67ca, 0x567_a6436, 0x5675_9fca, 0x5675_9c36] {
            for x in Vec3PackedSnorm(test).unpack() {
                assert!(x.abs() <= 1.0, "Not normalised!");
            }
        }
    }
}
