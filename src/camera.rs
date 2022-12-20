use std::f64;
use super::vec::{Vec3, Point3};
use super::ray::Ray;

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3
}

impl Camera {
    pub fn new(lookfrom: Point3, lookat: Point3, vup: Vec3, vfov: f64, aspect_ratio: f64) -> Camera {
        // const ASPECT_RATIO: f64 = 16.0 / 9.0;
        // const VIEWPORT_HEIGHT: f64 = 2.0;
        // const VIEWPORT_WIDTH: f64 = ASPECT_RATIO * VIEWPORT_HEIGHT;
        // const FOCAL_LENGTH: f64 = 1.0;

        // vertical field-of-view in degrees
        let theta = std::f64::consts::PI / 180.0 * vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = viewport_height * aspect_ratio;

        let cw = (lookfrom - lookat).normalized();
        let cu = vup.cross(cw).normalized();
        let cv = cw.cross(cu);

        let h = viewport_width * cu;
        let v = viewport_height * cv;
        let llc = lookfrom - h / 2.0 - v / 2.0 - cw;

        Camera {
            origin: lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc 
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        Ray::new(self.origin, self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin)
    }
}