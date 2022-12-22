use super::vec::{Vec3, Point3};

pub struct Ray {
    orig: Point3,
    dir: Vec3,
    time: f64
}

impl Ray {
    pub fn new(origin: Point3, directon: Vec3, time: f64) -> Ray {
        Ray {
            orig: origin,
            dir: directon,
            time
        }
    }

    pub fn origin(&self) -> Point3 {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }

    pub fn time(&self) -> f64 {
        self.time
    }
    
}