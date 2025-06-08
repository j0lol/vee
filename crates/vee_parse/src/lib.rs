//! Light library for parsing Mii character data.
//!
//! There are, roughly, two kinds of Mii data type.
//! - `CharInfo`: an uncompressed format.
//! - `StoreData`: a packed format. This is space minimized for transit (e.g. on the flash memory of an NFC toy.)
//! - `CharData`: `StoreData` without the checksum footer.
//! - _... there are more that this library does not implement._
//!
//! Supported by this library:
//!
//! | ..          | Ntr[^gen1]           | Rvl[^gen1]       | Ctr/Cafe    | Nx             |
//! |-------------|---------------|-----------|-------------|----------------|
//! | `CharInfo`  | ❌            | ❌        | ❌                           | [✅ `.charinfo`](NxCharInfo) |
//! | `StoreData` |  ❌ | ❌ | [✅ `.ffsd`](CtrStoreData)   | ❌ |
//! | `CoreData`  |  ❌| ❌ | ❌ | ❌ |
//!
//! [^gen1]: These formats are the same, apart from Ntr being little-endian and Rvl being big-endian.

pub mod ctr;
pub mod generic;
pub mod nx;

pub use ctr::CtrStoreData;
pub use nx::NxCharInfo;
pub use rvl_ntr::NtrCharData;
pub use rvl_ntr::NtrStoreData;
pub use rvl_ntr::RvlCharData;
pub use rvl_ntr::RvlStoreData;

pub use binrw::{binrw, BinRead, NullWideString};

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
    use crate::{CtrStoreData, NxCharInfo};
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
    fn ctr_deser() -> R {
        let mut mii = File::open(format!(
            "{}/resources_here/j0.ffsd",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap()
        ))?;

        let mii = CtrStoreData::read(&mut mii)?;

        assert_eq!(mii.name.to_string(), "Jo Null".to_string());
        assert_eq!(mii.personal_info_2.favorite_color().value(), 8);

        Ok(())
    }

    #[test]
    fn rvl_deser() -> R {
        let mut mii = File::open(format!(
            "{}/resources_here/j0.ffsd",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap()
        ))?;

        let mii = CtrStoreData::read(&mut mii)?;

        assert_eq!(mii.name.to_string(), "Jo Null".to_string());
        assert_eq!(mii.personal_info_2.favorite_color().value(), 8);

        Ok(())
    }
}
