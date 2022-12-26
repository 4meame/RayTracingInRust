use std::f64;
use rayon::iter::Positions;

use super::vec::Vec3;
use super::ray::Ray;
use super::hit::{Hittable, HitRecord};
use super::aabb::AABB;

pub enum Axis {
    X,
    Y,
    Z
}

fn get_axis_index(axis: &Axis) -> (usize, usize, usize) {
    match axis {
        Axis::X => (0, 1, 2),
        Axis::Y => (1, 0, 2),
        Axis::Z => (2, 0, 1)
    }
}

pub struct Rotate<H: Hittable> {
    axis: Axis,
    sin_theta: f64,
    cos_theta: f64,
    hittable: H,
    aabb: Option<AABB>
}

impl<H: Hittable> Rotate<H> {
    pub fn new(axis: Axis, hittable: H, angle: f64) -> Rotate<H> {
        let (r_axis, a_axis, b_axis) = get_axis_index(&axis);
        let radiants = (f64::consts::PI / 180.0) * angle;
        let sin_theta = f64::sin(radiants);
        let cos_theta = f64::cos(radiants);

        let aabb = hittable.bounding_box(0.0, 1.0).map(
            |mut aabb| {
                let mut min = Vec3::new(f64::MIN, f64::MIN, f64::MIN);
                let mut max = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
                for i in 0..2 {
                    for j in 0..2 {
                        for k in 0..2 {
                            let r = k as f64 * aabb.max[r_axis] + (1 - k) as f64 * aabb.min[r_axis];
                            let a = i as f64 * aabb.max[a_axis] + (1 - i) as f64 * aabb.min[a_axis];
                            let b = j as f64 * aabb.max[b_axis] + (1 - j) as f64 * aabb.min[b_axis];
                            let new_a = cos_theta * a + sin_theta * b;
                            let new_b = -sin_theta * a + cos_theta * b;
    
                            if new_a < min[a_axis] { min[a_axis] = new_a }
                            if new_b < min[b_axis] { min[b_axis] = new_b }
                            if r < min[r_axis] { min[r_axis] = r }
    
                            if new_a > max[a_axis] { max[a_axis] = new_a }
                            if new_b > max[b_axis] { max[b_axis] = new_b }
                            if r > max[r_axis] { max[r_axis] = r }
                        }
                    }
                }
                aabb.min = min;
                aabb.max = max;
                aabb
            }
        );
        Rotate {
            axis,
            sin_theta,
            cos_theta,
            hittable,
            aabb
        }
    }
}

impl<H: Hittable> Hittable for Rotate<H> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let (_, a_axis, b_axis) = get_axis_index(&self.axis);
        let mut origin = r.origin();
        let mut direction = r.direction();

        origin[a_axis] = &self.cos_theta * r.origin()[a_axis] - &self.sin_theta * r.origin()[b_axis];
        origin[b_axis] = &self.sin_theta * r.origin()[a_axis] + &self.cos_theta * r.origin()[b_axis];

        direction[a_axis] = &self.cos_theta * r.direction()[a_axis] - &self.sin_theta * r.direction()[b_axis];
        direction[b_axis] = &self.sin_theta * r.direction()[a_axis] + &self.cos_theta * r.direction()[b_axis];

        let rotated_ray = Ray::new(origin, direction, r.time());

        self.hittable.hit(&rotated_ray, t_min, t_max).map(
            |mut hit| {
                let mut position = hit.position;
                let mut normal = hit.normal;

                position[a_axis] = &self.cos_theta * hit.position[a_axis] + &self.sin_theta * hit.position[b_axis];
                position[b_axis] = -&self.sin_theta * hit.position[a_axis] + &self.cos_theta * hit.position[b_axis];
        
                normal[a_axis] = &self.cos_theta * hit.normal[a_axis] + &self.sin_theta * hit.normal[b_axis];
                normal[b_axis] = -&self.sin_theta * hit.normal[a_axis] + &self.cos_theta * hit.normal[b_axis];

                hit.position = position;
                hit.set_face_normal(&rotated_ray, normal);
                hit
            }
        )
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        self.aabb.clone()
    }
}