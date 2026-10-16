use rand::prelude::*;
use rand::rngs::StdRng;

/// Taken from <https://docs.rs/ggez/0.8.1/src/ggez/graphics/types.rs.html#335-340>
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// White (#FFFFFFFF)
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// Red
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// Cyan
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Magenta
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Create a new `Color` from four `f32`'s in the range `[0.0-1.0]`
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

pub fn lerp_color(a: Color, b: Color, pct: f32) -> Color {
    let red = a.r * (1.0 - pct) + b.r * pct;
    let green = a.g * (1.0 - pct) + b.g * pct;
    let blue = a.b * (1.0 - pct) + b.b * pct;
    let alpha = a.a * (1.0 - pct) + b.a * pct;
    Color::new(red, green, blue, alpha)
}

pub fn get_random_color(rng: &mut StdRng) -> Color {
    let r = rng.random_range(0.0..1.0);
    let g = rng.random_range(0.0..1.0);
    let b = rng.random_range(0.0..1.0);
    Color::new(r, g, b, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_color() {
        let red = Color::RED;
        let white = Color::WHITE;
        let cyan = Color::CYAN;
        let magenta = Color::MAGENTA;

        let expected_1 = Color::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(lerp_color(red, white, 1.0), expected_1);

        let expected_2 = Color::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(lerp_color(white, red, 1.0), expected_2);

        let expected_3 = Color::new(0.54, 0.45999998, 1.0, 1.0);
        assert_eq!(lerp_color(cyan, magenta, 0.54), expected_3);

        let expected_4 = Color::new(0.14445001, 0.85555, 1.0, 1.0);
        assert_eq!(lerp_color(magenta, cyan, 0.85555), expected_4);
    }

    #[test]
    fn test_get_random_color() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let color = get_random_color(&mut rng);
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
        assert_eq!(color.a, 1.0);
    }
}
