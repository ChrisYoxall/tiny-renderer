use crate::color::Color;
use crate::image::{Format, TgaImage};

/// A generic row-major 2D grid storing values of type `T` with bottom-left `(0, 0)` origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer2D<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T> Buffer2D<T> {
    /// Creates a new `width × height` buffer filled with `fill`.
    pub fn new(width: usize, height: usize, fill: T) -> Self
    where
        T: Clone,
    {
        Self {
            width,
            height,
            data: vec![fill; width * height],
        }
    }

    /// Returns the width of the buffer.
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the buffer.
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns a reference to the element at `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            Some(&self.data[y * self.width + x])
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width && y < self.height {
            Some(&mut self.data[y * self.width + x])
        } else {
            None
        }
    }

    /// Sets the element at `(x, y)` to `value`, returning `false` if out of bounds.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = value;
            true
        } else {
            false
        }
    }

    /// Fills the entire buffer with `value`.
    pub fn clear(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    /// Returns the underlying row-major slice.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the underlying mutable row-major slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns a mutable slice for row `y`, or `None` if `y >= height`.
    #[inline]
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y < self.height {
            let start = y * self.width;
            Some(&mut self.data[start..start + self.width])
        } else {
            None
        }
    }

    /// Flips the buffer horizontally in-place.
    pub fn flip_horizontally(&mut self) {
        let half = self.width / 2;
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..half {
                self.data.swap(row + x, row + (self.width - 1 - x));
            }
        }
    }

    /// Flips the buffer vertically in-place.
    pub fn flip_vertically(&mut self) {
        let half = self.height / 2;
        for y in 0..half {
            let top_row = y * self.width;
            let bottom_row = (self.height - 1 - y) * self.width;
            for x in 0..self.width {
                self.data.swap(top_row + x, bottom_row + x);
            }
        }
    }
}

/// A 2D z-buffer storing the closest recorded floating-point depth per pixel.
///
/// Uses the convention that larger `z` values are closer to the viewer. Unwritten pixels
/// are initialized to [`f32::NEG_INFINITY`].
#[derive(Clone, Debug, PartialEq)]
pub struct ZBuffer {
    depths: Buffer2D<f32>,
}

impl ZBuffer {
    /// Creates a new `width × height` z-buffer initialized to `f32::NEG_INFINITY`.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            depths: Buffer2D::new(width, height, f32::NEG_INFINITY),
        }
    }

    /// Returns the width of the z-buffer in pixels.
    pub const fn width(&self) -> usize {
        self.depths.width()
    }

    /// Returns the height of the z-buffer in pixels.
    pub const fn height(&self) -> usize {
        self.depths.height()
    }

    /// Resets all stored depths to `f32::NEG_INFINITY`.
    pub fn clear(&mut self) {
        self.depths.clear(f32::NEG_INFINITY);
    }

    /// Returns the recorded depth at `(x, y)`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        self.depths.get(x, y).copied()
    }

    /// Returns a mutable slice of depth values for row `y`, or `None` if `y >= height`.
    #[inline]
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [f32]> {
        self.depths.row_mut(y)
    }

    /// Performs the depth test at `(x, y)` against candidate depth `z`.
    ///
    /// If `(x, y)` is in bounds and `z` is strictly greater than the currently stored depth,
    /// updates the z-buffer with `z` and returns `true`. Otherwise returns `false`.
    #[inline]
    pub fn test_and_set(&mut self, x: usize, y: usize, z: f32) -> bool {
        if let Some(current) = self.depths.get_mut(x, y)
            && z > *current
        {
            *current = z;
            true
        } else {
            false
        }
    }

    /// Converts the z-buffer into an 8-bit grayscale [`TgaImage`] for debugging.
    ///
    /// Unwritten (`-inf`) pixels map to `0` (black). Written finite depths are dynamically
    /// auto-ranged across `[min_z, max_z]` into `1..=255` (brighter = closer). If all written
    /// pixels share a single depth, they map to `255`.
    pub fn to_grayscale_image(&self) -> TgaImage {
        let width = self.width();
        let height = self.height();
        let mut image = TgaImage::new(width, height, Format::Grayscale);

        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;
        for &z in self.depths.as_slice() {
            if z.is_finite() {
                min_z = min_z.min(z);
                max_z = max_z.max(z);
            }
        }

        if !min_z.is_finite() {
            return image;
        }

        let span = max_z - min_z;
        for y in 0..height {
            for x in 0..width {
                let z = self.depths.get(x, y).copied().unwrap_or(f32::NEG_INFINITY);
                if z.is_finite() {
                    let level = if span == 0.0 {
                        255u8
                    } else {
                        let normalized = (z - min_z) / span;
                        (1.0 + normalized * 254.0).round().clamp(1.0, 255.0) as u8
                    };
                    image.set(x, y, Color::rgb(level, level, level));
                }
            }
        }

        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer2d_get_set_clear_and_flips() {
        let mut buf = Buffer2D::new(2, 2, 0u32);
        assert_eq!(buf.width(), 2);
        assert_eq!(buf.height(), 2);

        assert!(buf.set(0, 0, 1));
        assert!(buf.set(1, 0, 2));
        assert!(buf.set(0, 1, 3));
        assert!(buf.set(1, 1, 4));
        assert!(!buf.set(2, 0, 99));
        assert_eq!(buf.get(2, 0), None);

        buf.flip_horizontally();
        assert_eq!(buf.get(0, 0), Some(&2));
        assert_eq!(buf.get(1, 0), Some(&1));

        buf.flip_vertically();
        assert_eq!(buf.get(0, 0), Some(&4));
        assert_eq!(buf.get(1, 0), Some(&3));

        buf.clear(7);
        assert_eq!(buf.as_slice(), &[7, 7, 7, 7]);
    }

    #[test]
    fn test_zbuffer_depth_test_semantics() {
        let mut zbuf = ZBuffer::new(3, 3);
        assert_eq!(zbuf.get(1, 1), Some(f32::NEG_INFINITY));

        // Any finite depth beats initial -inf.
        assert!(zbuf.test_and_set(1, 1, -0.5));
        assert_eq!(zbuf.get(1, 1), Some(-0.5));

        // Equal depth does not overwrite (strictly greater required).
        assert!(!zbuf.test_and_set(1, 1, -0.5));

        // Farther (smaller) depth is rejected.
        assert!(!zbuf.test_and_set(1, 1, -0.8));

        // Closer (larger) depth wins.
        assert!(zbuf.test_and_set(1, 1, 0.25));
        assert_eq!(zbuf.get(1, 1), Some(0.25));

        // NaN and out-of-bounds are rejected.
        assert!(!zbuf.test_and_set(1, 1, f32::NAN));
        assert!(!zbuf.test_and_set(3, 1, 10.0));

        zbuf.clear();
        assert_eq!(zbuf.get(1, 1), Some(f32::NEG_INFINITY));
    }

    #[test]
    fn test_zbuffer_to_grayscale_auto_ranging() {
        // Empty z-buffer produces all-black grayscale image.
        let empty = ZBuffer::new(2, 1);
        let empty_img = empty.to_grayscale_image();
        assert_eq!(empty_img.format(), Format::Grayscale);
        assert_eq!(empty_img.get(0, 0), Some(Color::BLACK));
        assert_eq!(empty_img.get(1, 0), Some(Color::BLACK));

        // Single flat depth maps written pixel to 255 and unwritten to 0.
        let mut flat = ZBuffer::new(2, 1);
        flat.test_and_set(0, 0, 0.4);
        let flat_img = flat.to_grayscale_image();
        assert_eq!(flat_img.get(0, 0), Some(Color::WHITE));
        assert_eq!(flat_img.get(1, 0), Some(Color::BLACK));

        // Multiple depths auto-range [min, max] -> [1, 255], leaving unwritten at 0.
        let mut ranged = ZBuffer::new(3, 1);
        ranged.test_and_set(0, 0, -2.0);
        ranged.test_and_set(1, 0, 2.0);
        let ranged_img = ranged.to_grayscale_image();
        assert_eq!(ranged_img.get(0, 0), Some(Color::rgb(1, 1, 1)));
        assert_eq!(ranged_img.get(1, 0), Some(Color::WHITE));
        assert_eq!(ranged_img.get(2, 0), Some(Color::BLACK));
    }
}
