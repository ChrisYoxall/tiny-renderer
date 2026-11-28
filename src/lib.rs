//! # Tiny Renderer in Rust
//!
//! A pure-Rust reimplementation of Dmitry V. Sokolov's [`tinyrenderer`](https://github.com/ssloy/tinyrenderer)
//! tutorial ([haqr.eu/tinyrenderer](https://haqr.eu/tinyrenderer/)).

pub mod color;
pub mod image;
pub mod math;
pub mod mesh;
pub mod rasterizer;

pub use color::Color;
pub use image::{Format, Framebuffer, TgaImage};
pub use math::{Barycentric, ScreenPoint, Vec3};
pub use mesh::{Mesh, MeshError};
pub use rasterizer::CullMode;
