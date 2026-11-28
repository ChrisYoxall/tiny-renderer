//! # Tiny Renderer in Rust
//!
//! A pure-Rust reimplementation of Dmitry V. Sokolov's [`tinyrenderer`](https://github.com/ssloy/tinyrenderer)
//! tutorial ([haqr.eu/tinyrenderer](https://haqr.eu/tinyrenderer/)).

pub mod buffer;
pub mod color;
pub mod image;
pub mod math;
pub mod mesh;
pub mod rasterizer;

pub use buffer::{Buffer2D, ZBuffer};
pub use color::Color;
pub use image::{Format, Framebuffer, TgaImage};
pub use math::{Barycentric, ScreenPoint, ScreenVertex, Vec3};
pub use mesh::{Mesh, MeshError};
pub use rasterizer::CullMode;
