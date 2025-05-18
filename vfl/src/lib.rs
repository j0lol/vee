#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]
pub mod charinfo;
pub mod color;

#[cfg(feature = "res")]
pub mod draw;

#[cfg(feature = "res")]
pub mod res;
mod utils;
