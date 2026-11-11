use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}
impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub center: Point,
    pub half: Point,
    pub rotation: f64,
}
impl Rect {
    pub fn new(center: Point, size: Point, rotation: f64) -> Self {
        Self {
            center,
            half: size * 0.5,
            rotation,
        }
    }
    pub fn axes(self) -> [Point; 2] {
        let (s, c) = self.rotation.to_radians().sin_cos();
        [Point::new(c, s), Point::new(-s, c)]
    }
    pub fn contains(self, point: Point) -> bool {
        let d = point - self.center;
        let [a, b] = self.axes();
        d.dot(a).abs() <= self.half.x && d.dot(b).abs() <= self.half.y
    }
    pub fn overlaps(self, rhs: Self) -> bool {
        let a = self.axes();
        let b = rhs.axes();
        let delta = rhs.center - self.center;
        a.into_iter().chain(b).all(|axis| {
            let ra = self.half.x * axis.dot(a[0]).abs() + self.half.y * axis.dot(a[1]).abs();
            let rb = rhs.half.x * axis.dot(b[0]).abs() + rhs.half.y * axis.dot(b[1]).abs();
            delta.dot(axis).abs() < ra + rb - 1e-7
        })
    }
    pub fn bounds(self) -> (Point, Point) {
        let [a, b] = self.axes();
        let half = Point::new(
            a.x.abs() * self.half.x + b.x.abs() * self.half.y,
            a.y.abs() * self.half.x + b.y.abs() * self.half.y,
        );
        (self.center - half, self.center + half)
    }
}
