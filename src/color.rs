/// Represents an RGBA 32-bit color.
///
/// This is a deliberately simple approach for now.  Will likely move to a generic approach later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);
    pub const RED: Self = Self::rgba(255, 0, 0, 255);
    pub const GREEN: Self = Self::rgba(0, 255, 0, 255);
    pub const BLUE: Self = Self::rgba(0, 0, 255, 255);
    pub const YELLOW: Self = Self::rgba(255, 255, 0, 255);

    /// Constructs an opaque RGB color with alpha = 255.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Constructs a color with RGBA components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the color channels in TGA's native little-endian BGR order (24 bits).
    pub const fn to_bgr(self) -> [u8; 3] {
        [self.b, self.g, self.r]
    }

    /// Returns the color channels in TGA's native little-endian BGRA order (32 bits).
    pub const fn to_bgra(self) -> [u8; 4] {
        [self.b, self.g, self.r, self.a]
    }
}
