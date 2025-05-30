//! Packed data structures used in resources.

use binrw::{BinRead, BinWrite};
use half::f16 as half_f16;

// TODO: unpack on GPU?
// Wgsl doesn't have a [10,10,10,2] unpacking method

/// WebGPU does not suport Snorm_10_10_10_2 so we have to convert this on the CPU.
/// This format contains 4 signed, normalised floats packed into 32 bits.
///
/// Format: `[10; 10; 10; 2]`
#[repr(transparent)]
#[derive(BinRead, BinWrite, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
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

/// Wrapper type for 16-bit floats 'cos Rust doesn't have stable support yet.
/// We can't just use `half::f16` because we need to de/serialize and send to GPU.
/// On the GPU we can implicitly convert any vector of `f16`s to `f32`.
#[repr(transparent)]
#[derive(BinRead, BinWrite, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct Float16(u16);

impl Default for Float16 {
    fn default() -> Self {
        Float16(half_f16::default().to_bits())
    }
}

impl Float16 {
    pub fn from_bits(bits: u16) -> Self {
        Float16(bits)
    }
    pub fn from_half(half: half_f16) -> Self {
        Float16(half.to_bits())
    }
    pub fn from_f32(float: f32) -> Self {
        Float16(half_f16::from_f32(float).to_bits())
    }
    pub fn as_half(self) -> half_f16 {
        half_f16::from_bits(self.0)
    }
    pub fn as_f32(self) -> f32 {
        self.as_half().to_f32()
    }
}

#[cfg(test)]
mod tests {
    use crate::packing::Vec3PackedSnorm;

    #[test]
    fn vec3_packed_snorm_test() {
        for test in [0x567a_67ca, 0x567_a6436, 0x5675_9fca, 0x5675_9c36] {
            for x in Vec3PackedSnorm(test).unpack() {
                assert!(x.abs() <= 1.0, "Not normalised!");
            }
        }
    }
}
