//! This is a library for working with Mii data. This library is mostly re-exports of sub-crates.
//!
//! Unless you're doing rendering, you probably don't need to rely on this whole crate.
//! Features are added for convenience, or you can import a sub-crate directly.
//!
//! # Features
//! - `res`: Imports model loading, building, resource file parsing, etc.
//! - `wgpu`: Pulls in `wgpu` integration. Heavy, but allows for full `Char` rendering.
//!
//! # Why so split up?
//! - There's a lot that goes into rendering, and a lot of components can be used as standalone
//!   tools. Being structured this way will allow for better code separation, testing, etc.
//!   And! It should allow for easier bindings to other languages.
//!
//! # Why is it called `vfl` and not `vee`?
//! Someone took `vee`... :-(
//!
//! VFL is a riff of Nintendo's official libraries: RFL, FFL.

#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::must_use_candidate)]

/// Parsing Mii data
pub use vee_parse as parse;


#[cfg(feature = "res")]
/// Loading Mii textures, meshes, colors
pub use vee_resources as res;

#[cfg(feature = "res")]
/// Building Mii models
pub use vee_models as model;

#[cfg(feature = "wgpu")]
/// Integration with `wgpu`
pub use vee_wgpu as impl_wgpu;
