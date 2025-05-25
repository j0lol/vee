#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::must_use_candidate)]

pub mod charinfo;
pub mod color;

#[cfg(feature = "res")]
pub mod draw;

#[cfg(feature = "res")]
pub mod res;
mod utils;
