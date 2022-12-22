use std::f64;
use super::vec::Vec3;
use super::ray::Ray;

#[derive(Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB { 
            min,
            max
        }
    }

    pub fn hit(&self, r: &Ray, mut t_in: f64, mut t_out: f64) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / r.direction()[a];
            let t0 = (self.min[a] - r.origin()[a]) * inv_d;
            let t1 = (self.max[a] - r.origin()[a]) * inv_d;
            let (t0, t1) = if inv_d < 0.0 {
                (t1, t0)
            } else {
                (t0, t1)
            };
            t_in = t_in.max(t0);
            t_out = t_out.min(t1);
            if t_out <= t_in {
                return false
            }
        }
        true
    }
}

/// merge 2 AABB into 1
pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
    let min = Vec3::new(
        f64::min(box0.min.x(), box1.min.x()),
        f64::min(box0.min.y(), box1.min.y()),
        f64::min(box0.min.z(), box1.min.z()));
    let max = Vec3::new(
        f64::max(box0.max.x(), box1.max.x()),
        f64::max(box0.max.y(), box1.max.y()),
        f64::max(box0.max.z(), box1.max.z()));

    AABB { min, max }
}