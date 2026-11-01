use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::color::Color;

/// TGA 2.0 footer signature: 4 bytes extension offset + 4 bytes developer offset + "TRUEVISION-XFILE.\0"
const TGA_FOOTER: [u8; 26] = [
    0, 0, 0, 0, // Extension area offset
    0, 0, 0, 0, // Developer area offset
    b'T', b'R', b'U', b'E', b'V', b'I', b'S', b'I', b'O', b'N', b'-', b'X', b'F', b'I', b'L', b'E',
    b'.', 0,
];

/// Supported pixel formats for TGA images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 24-bit RGB (stored as BGR on disk)
    Rgb,
    /// 32-bit RGBA (stored as BGRA on disk)
    Rgba,
}

impl Format {
    /// Number of bytes per pixel.
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }

    /// Bits per pixel for the TGA header.
    pub const fn bits_per_pixel(self) -> u8 {
        (self.bytes_per_pixel() * 8) as u8
    }
}

/// A 2D framebuffer image capable of being exported to TGA format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TgaImage {
    width: usize,
    height: usize,
    format: Format,
    pixels: Vec<Color>,
}

impl TgaImage {
    /// Creates a new image filled with black (`Color::BLACK`).
    pub fn new(width: usize, height: usize, format: Format) -> Self {
        Self::with_color(width, height, format, Color::BLACK)
    }

    /// Creates a new image filled with the specified background color.
    pub fn with_color(width: usize, height: usize, format: Format, initial_color: Color) -> Self {
        let pixels = vec![initial_color; width * height];
        Self {
            width,
            height,
            format,
            pixels,
        }
    }

    /// Returns the width of the image in pixels.
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the image in pixels.
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns the pixel format of the image.
    pub const fn format(&self) -> Format {
        self.format
    }

    /// Sets the pixel at coordinate `(x, y)` to `color`.
    ///
    /// Coordinates are 0-indexed with `(0, 0)` in the bottom-left corner
    /// (matching OpenGL convention).
    ///
    /// Returns `true` if the pixel was within bounds and set, or `false` if it was clipped.
    pub fn set(&mut self, x: usize, y: usize, color: Color) -> bool {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.pixels[index] = color;
            true
        } else {
            false
        }
    }

    /// Retrieves the color of the pixel at coordinate `(x, y)`.
    ///
    /// Returns `None` if `(x, y)` is outside image bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.pixels[index])
        } else {
            None
        }
    }

    /// Fills the entire framebuffer with a single color.
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }

    /// Flips the image horizontally in-place.
    pub fn flip_horizontally(&mut self) {
        let half = self.width / 2;
        for y in 0..self.height {
            for x in 0..half {
                let left_idx = y * self.width + x;
                let right_idx = y * self.width + (self.width - 1 - x);
                self.pixels.swap(left_idx, right_idx);
            }
        }
    }

    /// Flips the image vertically in-place.
    pub fn flip_vertically(&mut self) {
        let half = self.height / 2;
        for y in 0..half {
            let top_row_start = y * self.width;
            let bottom_row_start = (self.height - 1 - y) * self.width;
            for x in 0..self.width {
                self.pixels.swap(top_row_start + x, bottom_row_start + x);
            }
        }
    }

    /// Writes the image to an arbitrary byte writer in Truevision TGA format.
    ///
    /// When `vflip` is `true` (the default in `write_tga_file`), the origin is set to
    /// bottom-left (`0x00`), conforming with standard OpenGL coordinate systems.
    /// When `vflip` is `false`, the origin is set to top-left (`0x20`).
    pub fn write_tga<W: Write>(&self, writer: &mut W, vflip: bool) -> io::Result<()> {
        let mut buf_writer = BufWriter::new(writer);

        // 18-byte TGA Header
        let mut header = [0u8; 18];
        header[2] = 2; // Data type code: 2 = Uncompressed True-Color
        header[12..14].copy_from_slice(&(self.width as u16).to_le_bytes());
        header[14..16].copy_from_slice(&(self.height as u16).to_le_bytes());
        header[16] = self.format.bits_per_pixel();
        header[17] = if vflip { 0x00 } else { 0x20 };

        buf_writer.write_all(&header)?;

        // Write pixel data in little-endian order (BGR / BGRA)
        match self.format {
            Format::Rgb => {
                for pixel in &self.pixels {
                    buf_writer.write_all(&pixel.to_bgr())?;
                }
            }
            Format::Rgba => {
                for pixel in &self.pixels {
                    buf_writer.write_all(&pixel.to_bgra())?;
                }
            }
        }

        // Write standard TGA 2.0 footer
        buf_writer.write_all(&TGA_FOOTER)?;
        buf_writer.flush()?;

        Ok(())
    }

    /// Writes the image to a file path in uncompressed TGA format with bottom-left origin (`vflip = true`).
    pub fn write_tga_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.write_tga_file_with_options(path, true)
    }

    /// Writes the image to a file path with configurable vertical orientation.
    pub fn write_tga_file_with_options<P: AsRef<Path>>(&self, path: P, vflip: bool) -> io::Result<()> {
        let mut file = File::create(path)?;
        self.write_tga(&mut file, vflip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation_and_bounds() {
        let mut img = TgaImage::new(10, 10, Format::Rgb);
        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 10);
        assert_eq!(img.get(0, 0), Some(Color::BLACK));

        assert!(img.set(5, 5, Color::RED));
        assert_eq!(img.get(5, 5), Some(Color::RED));

        // Out of bounds checks
        assert!(!img.set(10, 5, Color::RED));
        assert!(!img.set(5, 10, Color::RED));
        assert_eq!(img.get(10, 5), None);
    }

    #[test]
    fn test_tga_header_serialization() {
        let mut img = TgaImage::new(2, 2, Format::Rgb);
        img.set(0, 0, Color::WHITE);

        let mut output = Vec::new();
        img.write_tga(&mut output, true).unwrap();

        // 18 bytes header + 4 pixels * 3 bytes (12 bytes) + 26 bytes footer = 56 bytes
        assert_eq!(output.len(), 18 + 12 + 26);

        // Check header fields
        assert_eq!(output[2], 2); // Uncompressed true-color
        let width = u16::from_le_bytes([output[12], output[13]]);
        let height = u16::from_le_bytes([output[14], output[15]]);
        assert_eq!(width, 2);
        assert_eq!(height, 2);
        assert_eq!(output[16], 24); // 24 bpp
        assert_eq!(output[17], 0x00); // vflip = true -> bottom-left origin

        // Check TGA footer signature
        assert_eq!(&output[output.len() - 18..], b"TRUEVISION-XFILE.\0");
    }

    #[test]
    fn test_flip_vertical_and_horizontal() {
        let mut img = TgaImage::new(2, 2, Format::Rgb);
        img.set(0, 0, Color::RED);
        img.set(1, 0, Color::GREEN);
        img.set(0, 1, Color::BLUE);
        img.set(1, 1, Color::WHITE);

        img.flip_horizontally();
        assert_eq!(img.get(0, 0), Some(Color::GREEN));
        assert_eq!(img.get(1, 0), Some(Color::RED));

        img.flip_vertically();
        assert_eq!(img.get(0, 0), Some(Color::WHITE));
        assert_eq!(img.get(1, 0), Some(Color::BLUE));
    }
}
