use crate::buffer::ZBuffer;
use crate::color::Color;
use crate::image::Framebuffer;
use crate::math::{Barycentric, ScreenPoint, ScreenVertex, Vec3};
use crate::mesh::Mesh;

/// Controls whether back-facing triangles are discarded during rasterization.
///
/// With a bottom-left `(0, 0)` screen coordinate origin, counter-clockwise (CCW) triangles
/// have positive signed area and are considered front-facing (matching OpenGL convention).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullMode {
    /// Discard clockwise (back-facing) triangles where signed area is negative.
    Back,
    /// Rasterize both counter-clockwise and clockwise triangles.
    None,
}

/// Rasterize a line segment between `(ax, ay)` and `(bx, by)` in screen coordinates
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

/// Computes twice the signed area of triangle `(a, b, c)` in screen coordinates.
///
/// Positive for counter-clockwise (CCW) winding, negative for clockwise (CW) winding,
/// and zero for degenerate (collinear) triangles. Uses `i128` to avoid overflow for all `i32` inputs.
#[inline]
pub fn signed_area2(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint) -> i128 {
    let ab_x = b.x as i128 - a.x as i128;
    let ab_y = b.y as i128 - a.y as i128;
    let ac_x = c.x as i128 - a.x as i128;
    let ac_y = c.y as i128 - a.y as i128;
    ab_x * ac_y - ab_y * ac_x
}

/// Rasterize a filled triangle using a clipped screen-space bounding box, barycentric
/// coordinates, and per-pixel depth testing against `zbuf`, invoking `shade` only for
/// visible pixels that pass the depth test.
///
/// Degenerate (zero-area) triangles are always skipped. When `cull` is [`CullMode::Back`],
/// clockwise (negative signed area) triangles are also discarded.
///
/// # Panics
///
/// Panics if `fb` and `zbuf` do not have matching `(width, height)` dimensions.
pub fn triangle_with<F, S>(
    verts: [ScreenVertex; 3],
    cull: CullMode,
    fb: &mut F,
    zbuf: &mut ZBuffer,
    mut shade: S,
) where
    F: Framebuffer,
    S: FnMut(Barycentric) -> Color,
{
    let width = fb.width();
    let height = fb.height();
    assert_eq!(
        (width, height),
        (zbuf.width(), zbuf.height()),
        "framebuffer and z-buffer dimensions must match"
    );
    if width == 0 || height == 0 {
        return;
    }

    let [a, b, c] = verts;
    let ap = a.point();
    let bp = b.point();
    let cp = c.point();
    let total = signed_area2(ap, bp, cp);
    if total == 0 || (cull == CullMode::Back && total < 0) {
        return;
    }

    let max_x = (width - 1) as i64;
    let max_y = (height - 1) as i64;
    let min_x = (a.x.min(b.x).min(c.x) as i64).max(0);
    let max_bx = (a.x.max(b.x).max(c.x) as i64).min(max_x);
    let min_y = (a.y.min(b.y).min(c.y) as i64).max(0);
    let max_by = (a.y.max(b.y).max(c.y) as i64).min(max_y);
    if min_x > max_bx || min_y > max_by {
        return;
    }

    let inv_total = 1.0 / (total as f64);
    let is_ccw = total > 0;

    // Incremental edge-function steps: signed_area2(p, u, v) is linear in p, so stepping
    // x by +1 adds (u.y - v.y) and stepping y by +1 adds (v.x - u.x).
    let dw0_dx = bp.y as i128 - cp.y as i128;
    let dw1_dx = cp.y as i128 - ap.y as i128;
    let dw2_dx = ap.y as i128 - bp.y as i128;

    let dw0_dy = cp.x as i128 - bp.x as i128;
    let dw1_dy = ap.x as i128 - cp.x as i128;
    let dw2_dy = bp.x as i128 - ap.x as i128;

    let p0 = ScreenPoint::new(min_x as i32, min_y as i32);
    let mut w0_row = signed_area2(p0, bp, cp);
    let mut w1_row = signed_area2(p0, cp, ap);
    let mut w2_row = signed_area2(p0, ap, bp);

    let x_min = min_x as usize;
    let x_max = max_bx as usize;

    for y in min_y..=max_by {
        let uy = y as usize;
        let mut w0 = w0_row;
        let mut w1 = w1_row;
        let mut w2 = w2_row;

        let z_row = &mut zbuf.row_mut(uy).expect("clamped y is in bounds")[x_min..=x_max];
        for (ux, z_slot) in (x_min..=x_max).zip(z_row.iter_mut()) {
            let inside = if is_ccw {
                w0 >= 0 && w1 >= 0 && w2 >= 0
            } else {
                w0 <= 0 && w1 <= 0 && w2 <= 0
            };

            if inside {
                let bary = Barycentric::new(
                    (w0 as f64 * inv_total) as f32,
                    (w1 as f64 * inv_total) as f32,
                    (w2 as f64 * inv_total) as f32,
                );
                let z = bary.interpolate_f32(a.z, b.z, c.z);
                if z > *z_slot {
                    *z_slot = z;
                    fb.set(ux, uy, shade(bary));
                }
            }

            w0 += dw0_dx;
            w1 += dw1_dx;
            w2 += dw2_dx;
        }

        w0_row += dw0_dy;
        w1_row += dw1_dy;
        w2_row += dw2_dy;
    }
}

/// Rasterize a filled triangle with a single flat `color` and depth testing against `zbuf`.
pub fn triangle_flat<F: Framebuffer>(
    verts: [ScreenVertex; 3],
    cull: CullMode,
    fb: &mut F,
    zbuf: &mut ZBuffer,
    color: Color,
) {
    triangle_with(verts, cull, fb, zbuf, |_| color);
}

/// Rasterize a filled triangle, smoothly interpolating per-vertex `colors` via barycentric weights
/// with depth testing against `zbuf`.
pub fn triangle_interpolated<F: Framebuffer>(
    verts: [ScreenVertex; 3],
    colors: [Color; 3],
    cull: CullMode,
    fb: &mut F,
    zbuf: &mut ZBuffer,
) {
    triangle_with(verts, cull, fb, zbuf, |bary| {
        bary.interpolate_color(colors[0], colors[1], colors[2])
    });
}

/// Projects a normalized model-space vertex `[-1, 1]³` into screen coordinates, preserving depth `z`.
pub fn project_to_screen(v: Vec3, width: usize, height: usize) -> ScreenVertex {
    let x = ((v.x + 1.0) * 0.5 * width.saturating_sub(1) as f32).round() as i32;
    let y = ((v.y + 1.0) * 0.5 * height.saturating_sub(1) as f32).round() as i32;
    ScreenVertex::new(x, y, v.z)
}

/// Rasterize all triangulated face edges of `mesh` as a wireframe in `color`.
///
/// Because [`Mesh`] triangulates polygonal `.obj` faces at load time, any quads or n-gons
/// are rendered with their triangulation diagonals visible.
pub fn wireframe<F: Framebuffer>(mesh: &Mesh, fb: &mut F, color: Color) {
    let width = fb.width();
    let height = fb.height();
    let vertices = mesh.vertices();

    for face in mesh.faces() {
        for i in 0..face.len() {
            let v0 = vertices[face[i]];
            let v1 = vertices[face[(i + 1) % face.len()]];
            let a = project_to_screen(v0, width, height);
            let b = project_to_screen(v1, width, height);
            line(a.x, a.y, b.x, b.y, fb, color);
        }
    }
}

/// Minimal pure-`std` deterministic PRNG (xorshift64) for reproducible per-face colors.
struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { Self::DEFAULT_SEED } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_color(&mut self) -> Color {
        let bits = self.next_u64();
        Color::rgb((bits & 0xFF) as u8, ((bits >> 8) & 0xFF) as u8, ((bits >> 16) & 0xFF) as u8)
    }
}

/// Rasterize all front-facing triangles of `mesh` with deterministic pseudo-random flat colors,
/// back-face culling, and z-buffer hidden-surface removal.
pub fn mesh_random_colors<F: Framebuffer>(
    mesh: &Mesh,
    fb: &mut F,
    zbuf: &mut ZBuffer,
    seed: u64,
) {
    let width = fb.width();
    let height = fb.height();
    assert_eq!(
        (width, height),
        (zbuf.width(), zbuf.height()),
        "framebuffer and z-buffer dimensions must match"
    );
    let vertices = mesh.vertices();
    let mut rng = Xorshift64::new(seed);

    for face in mesh.faces() {
        let color = rng.next_color();
        let a = project_to_screen(vertices[face[0]], width, height);
        let b = project_to_screen(vertices[face[1]], width, height);
        let c = project_to_screen(vertices[face[2]], width, height);
        triangle_flat([a, b, c], CullMode::Back, fb, zbuf, color);
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

    #[test]
    fn test_triangle_ccw_and_backface_culling() {
        let ccw = [
            ScreenVertex::new(1, 1, 0.0),
            ScreenVertex::new(3, 1, 0.0),
            ScreenVertex::new(1, 3, 0.0),
        ];
        let cw = [ccw[0], ccw[2], ccw[1]];

        assert!(signed_area2(ccw[0].point(), ccw[1].point(), ccw[2].point()) > 0);
        assert!(signed_area2(cw[0].point(), cw[1].point(), cw[2].point()) < 0);

        let mut fb_ccw = MockFramebuffer::new(5, 5);
        let mut zb_ccw = ZBuffer::new(5, 5);
        triangle_flat(ccw, CullMode::Back, &mut fb_ccw, &mut zb_ccw, Color::WHITE);
        assert_eq!(
            fb_ccw.lit_pixels(),
            vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (1, 3)]
        );

        let mut fb_cw_culled = MockFramebuffer::new(5, 5);
        let mut zb_cw_culled = ZBuffer::new(5, 5);
        triangle_flat(
            cw,
            CullMode::Back,
            &mut fb_cw_culled,
            &mut zb_cw_culled,
            Color::WHITE,
        );
        assert!(fb_cw_culled.lit_pixels().is_empty());

        let mut fb_cw_unculled = MockFramebuffer::new(5, 5);
        let mut zb_cw_unculled = ZBuffer::new(5, 5);
        triangle_flat(
            cw,
            CullMode::None,
            &mut fb_cw_unculled,
            &mut zb_cw_unculled,
            Color::WHITE,
        );
        assert_eq!(fb_cw_unculled.lit_pixels(), fb_ccw.lit_pixels());
    }

    #[test]
    fn test_triangle_degenerate_and_clipped_and_extreme_coords() {
        // Collinear (zero area) triangle is skipped regardless of CullMode.
        let mut fb_deg = MockFramebuffer::new(5, 5);
        let mut zb_deg = ZBuffer::new(5, 5);
        triangle_flat(
            [
                ScreenVertex::new(0, 0, 0.0),
                ScreenVertex::new(2, 2, 0.0),
                ScreenVertex::new(4, 4, 0.0),
            ],
            CullMode::None,
            &mut fb_deg,
            &mut zb_deg,
            Color::RED,
        );
        assert!(fb_deg.lit_pixels().is_empty());

        // Triangle extending outside framebuffer is safely clipped.
        let mut fb_clip = MockFramebuffer::new(3, 3);
        let mut zb_clip = ZBuffer::new(3, 3);
        triangle_flat(
            [
                ScreenVertex::new(-5, -5, 0.0),
                ScreenVertex::new(10, -5, 0.0),
                ScreenVertex::new(-5, 10, 0.0),
            ],
            CullMode::Back,
            &mut fb_clip,
            &mut zb_clip,
            Color::GREEN,
        );
        assert_eq!(fb_clip.lit_pixels().len(), 9);

        // Extreme i32 coordinates do not overflow signed_area2.
        let mut fb_ext = MockFramebuffer::new(2, 2);
        let mut zb_ext = ZBuffer::new(2, 2);
        triangle_flat(
            [
                ScreenVertex::new(-1_500_000_000, -1_500_000_000, 0.0),
                ScreenVertex::new(i32::MAX, -1_500_000_000, 0.0),
                ScreenVertex::new(-1_500_000_000, i32::MAX, 0.0),
            ],
            CullMode::Back,
            &mut fb_ext,
            &mut zb_ext,
            Color::BLUE,
        );
        assert_eq!(fb_ext.lit_pixels().len(), 4);
    }

    #[test]
    fn test_triangle_interpolated_colors() {
        let mut fb = MockFramebuffer::new(7, 7);
        let mut zb = ZBuffer::new(7, 7);
        let verts = [
            ScreenVertex::new(0, 0, 0.0),
            ScreenVertex::new(6, 0, 0.0),
            ScreenVertex::new(0, 6, 0.0),
        ];
        triangle_interpolated(
            verts,
            [Color::RED, Color::GREEN, Color::BLUE],
            CullMode::None,
            &mut fb,
            &mut zb,
        );

        assert_eq!(fb.get(0, 0), Color::RED);
        assert_eq!(fb.get(6, 0), Color::GREEN);
        assert_eq!(fb.get(0, 6), Color::BLUE);
        // Centroid (2, 2) has equal barycentric weights (1/3, 1/3, 1/3).
        assert_eq!(fb.get(2, 2), Color::rgb(85, 85, 85));
    }

    #[test]
    fn test_zbuffer_occlusion_is_draw_order_independent() {
        let near = [
            ScreenVertex::new(0, 0, 0.8),
            ScreenVertex::new(4, 0, 0.8),
            ScreenVertex::new(0, 4, 0.8),
        ];
        let far = [
            ScreenVertex::new(0, 0, -0.2),
            ScreenVertex::new(4, 0, -0.2),
            ScreenVertex::new(0, 4, -0.2),
        ];

        // Near drawn first, then far: near must remain visible.
        let mut fb_near_first = MockFramebuffer::new(5, 5);
        let mut zb_near_first = ZBuffer::new(5, 5);
        triangle_flat(near, CullMode::Back, &mut fb_near_first, &mut zb_near_first, Color::RED);
        triangle_flat(far, CullMode::Back, &mut fb_near_first, &mut zb_near_first, Color::BLUE);

        // Far drawn first, then near: near must overwrite far.
        let mut fb_far_first = MockFramebuffer::new(5, 5);
        let mut zb_far_first = ZBuffer::new(5, 5);
        triangle_flat(far, CullMode::Back, &mut fb_far_first, &mut zb_far_first, Color::BLUE);
        triangle_flat(near, CullMode::Back, &mut fb_far_first, &mut zb_far_first, Color::RED);

        assert_eq!(fb_near_first.pixels, fb_far_first.pixels);
        assert_eq!(fb_near_first.get(1, 1), Color::RED);
    }

    #[test]
    fn test_zbuffer_resolves_intersecting_slanted_triangles() {
        // Triangle A slopes from z = 1.0 on the left (x = 0) down to z = -1.0 on the right (x = 6).
        let tri_a = [
            ScreenVertex::new(0, 0, 1.0),
            ScreenVertex::new(6, 0, -1.0),
            ScreenVertex::new(0, 6, 1.0),
        ];
        // Triangle B slopes the opposite way: z = -1.0 on the left (x = 0) up to z = 1.0 on the right (x = 6).
        let tri_b = [
            ScreenVertex::new(0, 0, -1.0),
            ScreenVertex::new(6, 0, 1.0),
            ScreenVertex::new(0, 6, -1.0),
        ];

        let mut fb = MockFramebuffer::new(7, 7);
        let mut zb = ZBuffer::new(7, 7);
        triangle_flat(tri_a, CullMode::Back, &mut fb, &mut zb, Color::RED);
        triangle_flat(tri_b, CullMode::Back, &mut fb, &mut zb, Color::BLUE);

        // Left side (x = 1, y = 1): tri_a is closer -> RED.
        assert_eq!(fb.get(1, 1), Color::RED);
        // Right side (x = 5, y = 0): tri_b is closer -> BLUE.
        assert_eq!(fb.get(5, 0), Color::BLUE);
        // Exact tie at x = 3, y = 0 (both z = 0.0): first-drawn (RED) stays because test is strictly >.
        assert_eq!(fb.get(3, 0), Color::RED);
    }

    #[test]
    #[should_panic(expected = "framebuffer and z-buffer dimensions must match")]
    fn test_mismatched_framebuffer_and_zbuffer_panics() {
        let mut fb = MockFramebuffer::new(4, 4);
        let mut zb = ZBuffer::new(4, 5);
        triangle_flat(
            [
                ScreenVertex::new(0, 0, 0.0),
                ScreenVertex::new(2, 0, 0.0),
                ScreenVertex::new(0, 2, 0.0),
            ],
            CullMode::Back,
            &mut fb,
            &mut zb,
            Color::WHITE,
        );
    }

    #[test]
    fn test_mesh_random_colors_is_deterministic_and_culls_backfaces() {
        // Two triangles: face 0 is CCW (front-facing), face 1 is CW (back-facing).
        let obj = "\
v -0.5 -0.5 0.0
v  0.5 -0.5 0.0
v -0.5  0.5 0.0
f 1 2 3
f 1 3 2
";
        let mesh = Mesh::from_obj_str(obj).unwrap();

        let mut fb1 = MockFramebuffer::new(10, 10);
        let mut zb1 = ZBuffer::new(10, 10);
        let mut fb2 = MockFramebuffer::new(10, 10);
        let mut zb2 = ZBuffer::new(10, 10);
        mesh_random_colors(&mesh, &mut fb1, &mut zb1, 42);
        mesh_random_colors(&mesh, &mut fb2, &mut zb2, 42);

        assert!(!fb1.lit_pixels().is_empty());
        assert_eq!(fb1.pixels, fb2.pixels);
    }
}
