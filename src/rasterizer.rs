use crate::color::Color;
use crate::image::Framebuffer;
use crate::math::Vec3;
use crate::mesh::Mesh;

/// Rasterizes a line segment between `(ax, ay)` and `(bx, by)` in screen coordinates
/// using integer-only Bresenham's algorithm, clipping pixels outside the framebuffer.
pub fn line<F: Framebuffer>(
    ax: i32,
    ay: i32,
    bx: i32,
    by: i32,
    fb: &mut F,
    color: Color,
) {
    let (mut ax, mut ay, mut bx, mut by) = (ax as i64, ay as i64, bx as i64, by as i64);

    let steep = (ax - bx).abs() < (ay - by).abs();
    if steep {
        std::mem::swap(&mut ax, &mut ay);
        std::mem::swap(&mut bx, &mut by);
    }

    if ax > bx {
        std::mem::swap(&mut ax, &mut bx);
        std::mem::swap(&mut ay, &mut by);
    }

    let width = fb.width();
    let height = fb.height();
    if width == 0 || height == 0 {
        return;
    }

    let max_primary = if steep { height } else { width } as i64 - 1;
    let x_start = ax.max(0);
    let x_end = bx.min(max_primary);
    if x_start > x_end {
        return;
    }

    let dx = bx - ax;
    let dy = by - ay;
    let derror2 = dy.abs() * 2;
    let ystep: i64 = if by > ay { 1 } else { -1 };

    let mut y = ay;
    let mut error2: i64 = 0;
    if x_start > ax {
        // Fast-forward Bresenham state to `x_start` in O(1) without rounding drift.
        let k = (x_start - ax) as i128;
        let dx128 = dx as i128;
        let total = k * (derror2 as i128);
        let steps_y = (total + dx128 - 1) / (2 * dx128);
        y += (steps_y as i64) * ystep;
        error2 = (total - steps_y * 2 * dx128) as i64;
    }

    for x in x_start..=x_end {
        let (px, py) = if steep { (y, x) } else { (x, y) };
        if let (Ok(ux), Ok(uy)) = (usize::try_from(px), usize::try_from(py))
            && ux < width
            && uy < height
        {
            fb.set(ux, uy, color);
        }

        error2 += derror2;
        if error2 > dx {
            y += ystep;
            error2 -= dx * 2;
        }
    }
}

/// Projects a normalized model-space vertex `[-1, 1]³` into 2D screen coordinates.
pub fn project_to_screen(v: Vec3, width: usize, height: usize) -> (i32, i32) {
    let x = ((v.x + 1.0) * 0.5 * (width.saturating_sub(1)) as f32).round() as i32;
    let y = ((v.y + 1.0) * 0.5 * (height.saturating_sub(1)) as f32).round() as i32;
    (x, y)
}

/// Rasterizes all face edges of `mesh` as a wireframe in `color`.
pub fn wireframe<F: Framebuffer>(mesh: &Mesh, fb: &mut F, color: Color) {
    let width = fb.width();
    let height = fb.height();
    let vertices = mesh.vertices();

    for face in mesh.faces() {
        for i in 0..face.len() {
            let v0 = vertices[face[i]];
            let v1 = vertices[face[(i + 1) % face.len()]];
            let (ax, ay) = project_to_screen(v0, width, height);
            let (bx, by) = project_to_screen(v1, width, height);
            line(ax, ay, bx, by, fb, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lightweight in-memory framebuffer verifying the rasterizer is decoupled from `TgaImage`.
    struct MockFramebuffer {
        width: usize,
        height: usize,
        pixels: Vec<Color>,
    }

    impl MockFramebuffer {
        fn new(width: usize, height: usize) -> Self {
            Self {
                width,
                height,
                pixels: vec![Color::BLACK; width * height],
            }
        }

        fn get(&self, x: usize, y: usize) -> Color {
            self.pixels[y * self.width + x]
        }

        fn lit_pixels(&self) -> Vec<(usize, usize)> {
            let mut coords = Vec::new();
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.get(x, y) != Color::BLACK {
                        coords.push((x, y));
                    }
                }
            }
            coords
        }
    }

    impl Framebuffer for MockFramebuffer {
        fn width(&self) -> usize {
            self.width
        }

        fn height(&self) -> usize {
            self.height
        }

        fn set(&mut self, x: usize, y: usize, color: Color) -> bool {
            if x < self.width && y < self.height {
                self.pixels[y * self.width + x] = color;
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn test_horizontal_vertical_and_single_point() {
        let mut fb = MockFramebuffer::new(5, 5);
        line(1, 2, 3, 2, &mut fb, Color::WHITE);
        assert_eq!(fb.lit_pixels(), vec![(1, 2), (2, 2), (3, 2)]);

        let mut fb_v = MockFramebuffer::new(5, 5);
        line(2, 4, 2, 1, &mut fb_v, Color::RED);
        assert_eq!(fb_v.lit_pixels(), vec![(2, 1), (2, 2), (2, 3), (2, 4)]);

        let mut fb_p = MockFramebuffer::new(5, 5);
        line(3, 3, 3, 3, &mut fb_p, Color::GREEN);
        assert_eq!(fb_p.lit_pixels(), vec![(3, 3)]);
    }

    #[test]
    fn test_symmetry_across_endpoint_order_and_steepness() {
        let mut fb_a = MockFramebuffer::new(20, 20);
        let mut fb_b = MockFramebuffer::new(20, 20);
        line(2, 3, 15, 7, &mut fb_a, Color::WHITE);
        line(15, 7, 2, 3, &mut fb_b, Color::WHITE);
        assert_eq!(fb_a.lit_pixels(), fb_b.lit_pixels());

        let mut fb_steep_a = MockFramebuffer::new(20, 20);
        let mut fb_steep_b = MockFramebuffer::new(20, 20);
        line(3, 2, 7, 16, &mut fb_steep_a, Color::WHITE);
        line(7, 16, 3, 2, &mut fb_steep_b, Color::WHITE);
        assert_eq!(fb_steep_a.lit_pixels(), fb_steep_b.lit_pixels());
    }

    #[test]
    fn test_clips_negative_and_out_of_bounds_coordinates() {
        let mut fb = MockFramebuffer::new(4, 4);
        line(-2, -2, 5, 5, &mut fb, Color::WHITE);
        assert_eq!(fb.lit_pixels(), vec![(0, 0), (1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn test_extreme_i32_coordinates_do_not_overflow() {
        let mut fb = MockFramebuffer::new(3, 3);
        line(i32::MIN, i32::MIN, i32::MAX, i32::MAX, &mut fb, Color::WHITE);
        assert_eq!(fb.lit_pixels(), vec![(0, 0), (1, 1), (2, 2)]);

        let mut fb_rev = MockFramebuffer::new(3, 3);
        line(
            1_500_000_000,
            -1_499_999_998,
            -1_500_000_000,
            1_500_000_002,
            &mut fb_rev,
            Color::WHITE,
        );
        assert_eq!(fb_rev.lit_pixels(), vec![(2, 0), (1, 1), (0, 2)]);
    }
}
