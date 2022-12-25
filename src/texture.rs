use std::f64;
use super::vec::{Color, Vec3};
use super::perlin::Perlin;

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

pub struct NoiseTexture {
    noise: Perlin,
    scale: f64
}

impl NoiseTexture {
    pub fn new(scale: f64) -> NoiseTexture {
        NoiseTexture {
            noise: Perlin::new(),
            scale
        }
    }
}

impl Texture for NoiseTexture {
    fn mapping(&self, _u: f64, _v: f64, p: &Vec3) -> Color {
        //Color::new(1.0, 1.0, 1.0) * 0.5 * ( 1.0 + self.noise.perlin(p))

        //Color::new(1.0, 1.0, 1.0) * self.noise.turb(p, self.scale, 7)

        Color::new(1.0, 1.0, 1.0) * 0.5 * (1.0 + f64::sin(self.scale * p.z() + 10.0 * self.noise.turb(p, self.scale, 7)))
    }
}