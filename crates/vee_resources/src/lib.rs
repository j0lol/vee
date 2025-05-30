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
