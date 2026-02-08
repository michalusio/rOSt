use core::ops::{Add, Div, Mul, Sub};

use crate::div_255_fast;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct VGAColor<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: T,
}
pub static TRANSPARENT: VGAColor<u8> = VGAColor {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0,
};
pub static WHITE: VGAColor<u8> = VGAColor {
    red: 255,
    green: 255,
    blue: 255,
    alpha: 255,
};
pub static BLACK: VGAColor<u8> = VGAColor {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 255,
};
pub static RED: VGAColor<u8> = VGAColor {
    red: 255,
    green: 0,
    blue: 0,
    alpha: 255,
};
pub static GREEN: VGAColor<u8> = VGAColor {
    red: 0,
    green: 255,
    blue: 0,
    alpha: 255,
};
pub static BLUE: VGAColor<u8> = VGAColor {
    red: 0,
    green: 0,
    blue: 255,
    alpha: 255,
};
pub static CLAY: VGAColor<u8> = VGAColor {
    red: 128,
    green: 64,
    blue: 11,
    alpha: 255,
};
pub static BSOD_BLUE: VGAColor<u8> = VGAColor {
    red: 9,
    green: 78,
    blue: 130,
    alpha: 255,
};
pub static CHARLOTTE: VGAColor<u8> = VGAColor {
    red: 161,
    green: 232,
    blue: 223,
    alpha: 255,
};

impl VGAColor<u8> {
    #[inline(always)]
    pub const fn from_rgba(data: &[u8]) -> VGAColor<u8> {
        VGAColor {
            red: data[0],
            green: data[1],
            blue: data[2],
            alpha: data[3],
        }
    }

    #[inline(always)]
    pub const fn from_bgra(data: &[u8]) -> VGAColor<u8> {
        VGAColor {
            blue: data[0],
            green: data[1],
            red: data[2],
            alpha: data[3],
        }
    }

    /// Interpolates between two colors, where t=0 -> First color, t=255 -> Second color
    pub fn interpolate(a: VGAColor<u8>, b: VGAColor<u8>, t: u8) -> VGAColor<u8> {
        let _t = t as u16;
        let t1 = 255 - _t;
        VGAColor {
            red: div_255_fast(a.red as u16 * t1 + b.red as u16 * _t),
            green: div_255_fast(a.green as u16 * t1 + b.green as u16 * _t),
            blue: div_255_fast(a.blue as u16 * t1 + b.blue as u16 * _t),
            alpha: div_255_fast(a.alpha as u16 * t1 + b.alpha as u16 * _t),
        }
    }

    #[inline(always)]
    pub const fn rgba(&self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    #[inline(always)]
    pub const fn bgra(&self) -> [u8; 4] {
        [self.blue, self.green, self.red, self.alpha]
    }
}

impl<T> Add for VGAColor<T>
where
    T: core::ops::Add<Output = T>,
{
    type Output = VGAColor<T>;

    fn add(self, rhs: Self) -> Self::Output {
        VGAColor {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
            alpha: self.alpha + rhs.alpha,
        }
    }
}

impl<T> Sub for VGAColor<T>
where
    T: core::ops::Sub<Output = T>,
{
    type Output = VGAColor<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        VGAColor {
            red: self.red - rhs.red,
            green: self.green - rhs.green,
            blue: self.blue - rhs.blue,
            alpha: self.alpha - rhs.alpha,
        }
    }
}

impl<T> Mul for VGAColor<T>
where
    T: core::ops::Mul<Output = T>,
{
    type Output = VGAColor<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        VGAColor {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
            alpha: self.alpha * rhs.alpha,
        }
    }
}

impl<T> Mul<T> for VGAColor<T>
where
    T: core::ops::Mul<Output = T> + Copy,
{
    type Output = VGAColor<T>;

    fn mul(self, rhs: T) -> Self::Output {
        VGAColor {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
            alpha: self.alpha * rhs,
        }
    }
}

impl<T> Div for VGAColor<T>
where
    T: core::ops::Div<Output = T>,
{
    type Output = VGAColor<T>;

    fn div(self, rhs: Self) -> Self::Output {
        VGAColor {
            red: self.red / rhs.red,
            green: self.green / rhs.green,
            blue: self.blue / rhs.blue,
            alpha: self.alpha / rhs.alpha,
        }
    }
}

impl<T> Div<T> for VGAColor<T>
where
    T: core::ops::Div<Output = T> + Copy,
{
    type Output = VGAColor<T>;

    fn div(self, rhs: T) -> Self::Output {
        VGAColor {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
            alpha: self.alpha / rhs,
        }
    }
}

impl<T> VGAColor<T>
where
    T: Into<u32> + Copy,
{
    /// Returns the value for the grayscale version of the color, using the human light perception formula
    pub fn to_grayscale(self) -> u32 {
        (self.red.into() * 299 + self.green.into() * 587 + self.blue.into() * 114) / 1000
    }
}

impl VGAColor<u8> {
    /// Multiplies only the alpha value by the opacity value, returning a new color with alpha scaled back to 0-255.
    pub fn mul_alpha(self, opacity: u8) -> VGAColor<u8> {
        VGAColor {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: div_255_fast(self.alpha as u16 * opacity as u16),
        }
    }
}
