//! # Tiny Renderer in Rust
//!
//! A pure-Rust reimplementation of Dmitry V. Sokolov's [`tinyrenderer`](https://github.com/ssloy/tinyrenderer)
//! tutorial ([haqr.eu/tinyrenderer](https://haqr.eu/tinyrenderer/)).

pub mod color;
pub mod image;

pub use color::Color;
pub use image::{Format, TgaImage};
