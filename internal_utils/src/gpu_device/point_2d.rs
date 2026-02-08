use core::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point2D<T> {
    pub x: T,
    pub y: T,
}
pub static ZERO: Point2D<isize> = Point2D { x: 0, y: 0 };

impl<T> Add for Point2D<T>
where
    T: core::ops::Add<Output = T>,
{
    type Output = Point2D<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Point2D<T>
where
    T: core::ops::Sub<Output = T>,
{
    type Output = Point2D<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul for Point2D<T>
where
    T: core::ops::Mul<Output = T>,
{
    type Output = Point2D<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> Mul<T> for Point2D<T>
where
    T: core::ops::Mul<Output = T> + Copy,
{
    type Output = Point2D<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div for Point2D<T>
where
    T: core::ops::Div<Output = T>,
{
    type Output = Point2D<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T> Div<T> for Point2D<T>
where
    T: core::ops::Div<Output = T> + Copy,
{
    type Output = Point2D<T>;

    fn div(self, rhs: T) -> Self::Output {
        Point2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl From<Point2D<u32>> for Point2D<f32> {
    fn from(p: Point2D<u32>) -> Self {
        Point2D {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

impl From<Point2D<u16>> for Point2D<f32> {
    fn from(p: Point2D<u16>) -> Self {
        Point2D {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

impl From<Point2D<f32>> for Point2D<u32> {
    fn from(p: Point2D<f32>) -> Self {
        Point2D {
            x: p.x as u32,
            y: p.y as u32,
        }
    }
}

impl From<Point2D<f32>> for Point2D<u16> {
    fn from(p: Point2D<f32>) -> Self {
        Point2D {
            x: p.x as u16,
            y: p.y as u16,
        }
    }
}

impl<T> Point2D<T>
where
    T: Ord + core::ops::Sub<Output = T> + Copy,
{
    /// Returns the squared distance between two points.
    pub fn sqr_distance<V>(self, other: Self) -> V
    where
        V: core::ops::Mul<Output = V> + core::ops::Add<Output = V> + Copy,
        T: Into<V>,
    {
        let a = (self.x.max(other.x) - self.x.min(other.x)).into();
        let b = (self.y.max(other.y) - self.y.min(other.y)).into();
        a * a + b * b
    }
}
