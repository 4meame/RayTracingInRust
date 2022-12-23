use std::f64;
use super::vec::{Color, Vec3};

pub trait Texture: Sync {
    fn mapping(&self, u: f64, v: f64, p: &Vec3) -> Color;
}

pub struct ConstantTexture {
    value: Color
}

impl ConstantTexture {
    pub fn new(color: Color) -> ConstantTexture {
        ConstantTexture {
            value: color
        }
    }
}

impl Texture for ConstantTexture {
    fn mapping(&self, _u: f64, _v: f64, _p: &Vec3) -> Color {
        self.value
    }
}

pub struct CheckTexture<T: Texture, U: Texture> {
    odd: T,
    even: U
}

impl<T: Texture, U: Texture> CheckTexture<T, U> {
    pub fn new(odd: T, even: U) -> CheckTexture<T, U> {
        CheckTexture {
            odd,
            even
        }
    }
}

impl<T: Texture, U: Texture> Texture for CheckTexture<T, U> {
    fn mapping(&self, u: f64, v: f64, p: &Vec3) -> Color {
        let sines = f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
        if sines < 0.0 {
            self.odd.mapping(u, v, p)
        } else {
            self.even.mapping(u, v, p)
        }
    }
}