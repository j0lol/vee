#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]
pub mod charinfo;
pub mod color;
#[cfg(feature = "res")]
pub mod mask;
#[cfg(feature = "res")]
pub mod shape_load;
#[cfg(feature = "res")]
pub mod tex_load;
mod utils;
