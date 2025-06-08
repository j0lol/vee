//! Very simple library for parsing `Char` data. Currently only supports the Nx `Char` format.
//! In the future, multiple formats will be supported and a trait will allow
//! for being generic over formats.

pub mod cafe;
pub mod nx;

pub use cafe::CafeCharStoreData;
pub use nx::NxCharInfo;

pub use binrw::{BinRead, NullWideString, binrw};

/// A UTF-16 String with a fixed length and non-enforced null termination.
/// The string is allowed to reach the maximum length without a null terminator,
/// and any nulls are stripped.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[binrw]
#[repr(transparent)]
pub struct FixedLengthWideString<const CHARS: usize>(pub [u16; CHARS]);

impl<const N: usize> std::fmt::Debug for FixedLengthWideString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FixedLengthWideString(\"{}\")", self)
    }
}

impl<const N: usize> std::fmt::Display for FixedLengthWideString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parse_utf16())
    }
}

impl<const N: usize> FixedLengthWideString<N> {
    /// Will fail on big endian hardware. Wait for `str_from_utf16_endian` to drop.
    fn parse_utf16(self) -> String {
        String::from_utf16(&self.0[..])
            .expect(
                "UTF-16 string parse error. Parsing little endian string on big endian hardware?",
            )
            .replace("\0", "")
    }
}

#[cfg(test)]
mod tests {
    use crate::{CafeCharStoreData, NxCharInfo};
    use binrw::BinRead;
    use std::{error::Error, fs::File};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn nx_deser() -> R {
        let mut mii = File::open(format!(
            "{}/resources_here/j0.charinfo",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap()
        ))?;

        let mii = NxCharInfo::read(&mut mii)?;

        assert_eq!(mii.nickname.to_string(), "Jo Null".to_string());

        assert_eq!(mii.glass_color.0, 17);
        assert_eq!(mii.reserved, 0);

        Ok(())
    }

    #[test]
    fn cafe_deser() -> R {
        let mut mii = File::open(format!(
            "{}/resources_here/j0.ffsd",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap()
        ))?;

        let mii = CafeCharStoreData::read(&mut mii)?;

        assert_eq!(mii.name.to_string(), "Jo Null".to_string());
        assert_eq!(mii.personal_info.favorite_color().value(), 8);

        Ok(())
    }
}
