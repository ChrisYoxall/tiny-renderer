use crate::color::Color;

/// A 3D vector / vertex position in model space.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Creates a new `Vec3` with the given `x`, `y`, and `z` components.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// A 2D discrete coordinate in screen space with bottom-left `(0, 0)` origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl ScreenPoint {
    /// Creates a new `ScreenPoint` at `(x, y)`.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Normalized barycentric coordinates `(alpha, beta, gamma)` with respect to a triangle's
/// three vertices `(A, B, C)`, where `alpha + beta + gamma == 1`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Barycentric {
    pub alpha: f32,
    pub beta: f32,
    pub gamma: f32,
}

impl Barycentric {
    /// Creates a new `Barycentric` triple.
    pub const fn new(alpha: f32, beta: f32, gamma: f32) -> Self {
        Self { alpha, beta, gamma }
    }

    /// Returns `true` if the point lies inside or on the boundary of the triangle.
    pub fn is_inside(&self) -> bool {
        self.alpha >= 0.0 && self.beta >= 0.0 && self.gamma >= 0.0
    }

    /// Linearly interpolates three scalar vertex attributes using these barycentric weights.
    pub fn interpolate_f32(&self, a: f32, b: f32, c: f32) -> f32 {
        self.alpha * a + self.beta * b + self.gamma * c
    }

    /// Linearly interpolates three RGBA vertex colors using these barycentric weights.
    pub fn interpolate_color(&self, a: Color, b: Color, c: Color) -> Color {
        let interp = |ca: u8, cb: u8, cc: u8| -> u8 {
            self.interpolate_f32(ca as f32, cb as f32, cc as f32)
                .clamp(0.0, 255.0)
                .round() as u8
        };

        Color::rgba(
            interp(a.r, b.r, c.r),
            interp(a.g, b.g, c.g),
            interp(a.b, b.b, c.b),
            interp(a.a, b.a, c.a),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barycentric_inside_and_outside() {
        assert!(Barycentric::new(1.0, 0.0, 0.0).is_inside());
        assert!(Barycentric::new(0.25, 0.5, 0.25).is_inside());
        assert!(!Barycentric::new(-0.01, 0.51, 0.5).is_inside());
    }

    #[test]
    fn test_barycentric_interpolate_f32_and_color() {
        let centroid = Barycentric::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
        assert!((centroid.interpolate_f32(0.0, 3.0, 6.0) - 3.0).abs() < 1e-5);

        let mixed = centroid.interpolate_color(Color::RED, Color::GREEN, Color::BLUE);
        assert_eq!(mixed, Color::rgb(85, 85, 85));

        let vertex_a = Barycentric::new(1.0, 0.0, 0.0);
        assert_eq!(
            vertex_a.interpolate_color(Color::RED, Color::GREEN, Color::BLUE),
            Color::RED
        );
    }
}
