use std::f64;
use super::vec::{Color, Vec3};
use super::perlin::Perlin;

pub trait Texture: Sync {
    fn mapping(&self, u: f64, v: f64, p: &Vec3) -> Color;
}


#[derive(Clone)]
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


#[derive(Clone)]
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

#[derive(Clone)]
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


#[derive(Clone)]
pub struct ImageTexture {
    data: Vec<u8>,
    width: u32,
    height: u32
}

impl ImageTexture {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> ImageTexture {
        ImageTexture {
            data,
            width,
            height
        }
    }
}

impl Texture for ImageTexture {
    fn mapping(&self, u: f64, v: f64, p: &Vec3) -> Color {
        let width = self.width as usize;
        let height = self.height as usize;
        // clamp input texture coordinates to [0,1] x [1,0]
        let mut i = (u.clamp(0.0, 1.0) * width as f64) as usize;
        // flip V to image coordinates
        let mut j = ((1.0 - v).clamp(0.0, 1.0) * height as f64) as usize;
        // clamp integer mapping, since actual coordinates should be less than 1.0
        if i > width - 1 {
            i = width -1
        }
        if j > height - 1 {
            j = height -1 
        }
        //3 bytes per pixel
        let idx = 3 * i + 3 * width * j;
        let r = self.data[idx] as f64 / 255.0;
        let g = self.data[idx + 1] as f64 / 255.0;
        let b = self.data[idx + 2] as f64 / 255.0;
        Color::new(r, g, b)
    }
}
