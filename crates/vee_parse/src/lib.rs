//! Library for parsing and converting Mii character data.
//!
//! # Parsing
//!
//! There are, roughly, two kinds of Mii data type.
//! - `CharInfo`: an uncompressed format.
//! - `StoreData`: a packed format. This is space minimized for transit (e.g. on the flash memory of an NFC toy.)
//!   - `CharData`: `StoreData` without the checksum footer.
//! - _... there are more that this library does not implement [^morefmt]._
//!
//! [^morefmt]: Extra formats are used in Mii databases, and may need to be added if the library comes to require support for databases.
//!
//! Supported by this library:
//!
//! | ..          | Ntr[^gen1]           | Rvl[^gen1]       | Ctr/Cafe    | Nx             | WebStudio |
//! |-------------|----------------------|------------------|-------------|----------------|-----------|
//! | `CharInfo`  | ❌                   | ❌        | ❌                           | [`.charinfo`](NxCharInfo) | .. |
//! | `StoreData` |  [`.nsd`](NtrStoreData) | [`.rsd`](RvlStoreData) | [`.ffsd`](CtrStoreData)[^ff]   | ❌ | .. |
//! | `CoreData`  | [`.ncd`](NtrCharData) | [`.rcd`](RvlCharData) | ❌ | ❌ | .. |
//! | In-memory   | .. | .. | .. | .. | 🏗️[^mnms]  |
//! [^gen1]: These formats are the same, apart from Ntr being little-endian and Rvl being big-endian.
//! [^ff]: The official format is Ca**f**e **F**ace **S**tore **D**ata, probably due to CFSD being taken by Ctr <sup>[src](https://github.com/HEYimHeroic/MiiDataFiles)</sup>.
//! [^mnms]: Stored in the browser's `localStorage`. Often shared as a base64 string, or sometimes saved with the `.mnms` extension.
//!
//! # Conversion
//!
//! <div class="warning">
//!
//! **Under Construction**
//!
//! This part of the library is still in works.
//! You can instantiate a GenericChar, but
//! conversions have not been implemented yet.
//!
//! </div>
//!
//! This library provides a [`GenericChar`] struct, which provides a common ground for Char data formats.
//! Due to the changes in shape, color and texture indices between {Rvl, Ntr} and later formats,
//! only a one-way conversion can be infallably performed.
//!
#![doc = svgbobdoc::transform!(
/// <center>
/// ```svgbob
///                                               ┌────────────────┐
///                                               │                │
///                                 ┌────────────►│  CtrStoreData  │
/// ┌───────────┐                   │             │                │
/// │           │                   │             └────────────────┘
/// │  NSD,NCD  │                   ▼
/// │           ├────────►┌───────────────┐        ┌──────────────┐
/// └────────┬──┘         │               │        │              │
///    ▲     ▼            │  GenericChar  │◄──────►│  NxCharInfo  │
/// ┌──┴────────┐         │               │        │              │
/// │           ├────────►└───────────────┘        └──────────────┘
/// │  RSD,RCD  │                   ▲
/// │           │                   │             ┌────────────────┐
/// └───────────┘                   │             │                │
///                                 └────────────►│  WsLocalStore  │
///                                               │                │
///                                               └────────────────┘
/// ```
/// </center>
)]
//!
//! # Usage
//!
//! ```no_run
//! use std::{env, fs::File, error::Error};
//! use vee_parse::{NxCharInfo, BinRead};
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let charinfo_path = "./Alice.charinfo";
//!     let mut file = File::open(&charinfo_path)?;
//!
//!     let nx_char = NxCharInfo::read(&mut file)?;
//!
//!     let name = nx_char.nickname.to_string(); // "Alice"
//!
//!     Ok(())
//! }
//!
//!
//! ```

pub mod ctr;
pub mod generic;
pub mod nx;
pub mod rvl_ntr;

pub use binrw::{BinRead, NullWideString, binrw};
pub use ctr::CtrStoreData;
pub use generic::GenericChar;
pub use nx::NxCharInfo;
pub use rvl_ntr::NtrCharData;
pub use rvl_ntr::NtrStoreData;
pub use rvl_ntr::RvlCharData;
pub use rvl_ntr::RvlStoreData;

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
    use crate::{CtrStoreData, NxCharInfo, RvlCharData, rvl_ntr::FavoriteColor};
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
    fn rvl_ntr_deser() -> R {
        let mut rvl = File::open(format!(
            "{}/resources_here/Jain.rcd",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            )
        ))?;
        let rvl = RvlCharData::read(&mut rvl)?;

        assert_eq!(rvl.name.to_string(), "Jain".to_string());

        let mut ntr = File::open(format!(
            "{}/resources_here/Jain.rcd",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            )
        ))?;
        let ntr = RvlCharData::read(&mut ntr)?;

        assert_eq!(ntr.name.to_string(), "Jain".to_string());
        assert_eq!(ntr.name.to_string(), rvl.name.to_string());

        assert_eq!(ntr.personal_info.favorite_color(), FavoriteColor::Purple);
        assert_eq!(
            ntr.personal_info.favorite_color(),
            rvl.personal_info.favorite_color()
        );

        Ok(())
    }
}
