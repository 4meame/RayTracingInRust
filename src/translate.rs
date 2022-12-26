use super::vec::Vec3;
use super::ray::Ray;
use super::hit::{Hittable, HitRecord};
use super::aabb::AABB;

pub struct Translate<H: Hittable> {
    hittable: H,
    offset: Vec3
}

impl<H: Hittable> Translate<H> {
    pub fn new(hittable: H, offset: Vec3) -> Translate<H> {
        Translate {
            hittable,
            offset
        }
    }
}

impl<H: Hittable> Hittable for Translate<H> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let translated_ray = Ray::new(r.origin() - self.offset, r.direction(), r.time());
        self.hittable.hit(&translated_ray, t_min, t_max).map(
            |mut hit| {
                hit.position += self.offset;
                hit
            }
        )
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        self.hittable.bounding_box(t0, t1).map(
            |mut aabb| {
                aabb.min += self.offset;
                aabb.max += self.offset;
                aabb
            }
        )
    }
}