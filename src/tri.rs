use std::f64;
use super::vec::{Vec3, Point3};
use super::ray::Ray;
use super::hit::{Hittable, HitRecord};
use super::mat::Material;
use super::aabb::AABB;

pub struct Triangle<M: Material> {
    vertices: [Point3; 3],
    material: M
}

impl<M: Material> Triangle<M> {
    pub fn new(vertices: [Point3; 3], material: M) -> Triangle<M> {
        Triangle {
            vertices,
            material
        }
    }
}

impl<M: Material> Hittable for Triangle<M> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Möller–Trumbore algorithm
        let s = r.origin() - self.vertices[0];
        let e1 = self.vertices[1] - self.vertices[0];
        let e2 = self.vertices[2] - self.vertices[0];
        let s1 = r.direction().cross(e2);
        let s2 = s.cross(e1);
        let s1_e1 = s1.dot(e1);
        let t = s2.dot(e2) / s1_e1;
        let b1 = s1.dot(s) / s1_e1;
        let b2 = s2.dot(r.direction()) / s1_e1;

        if t < t_min || t> t_max {
            None
        } else {
            if b1 < 0.0 || b2 < 0.0 || (1.0 - b1 - b2) < 0.0 {
                None
            } else {
                let p = r.at(t);
                let normal = e1.cross(e2).normalized();
                let mut rec = HitRecord {
                    position: p,
                    normal,
                    t,
                    u: b1,
                    v: b2,
                    front_face: false,
                    material: &self.material
                };
                rec.set_face_normal(r, normal);
                Some(rec)
            }
        }
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        let min_x = self.vertices[0].x().min(f64::min(self.vertices[1].x(), self.vertices[2].x()));
        let min_y = self.vertices[0].y().min(f64::min(self.vertices[1].y(), self.vertices[2].y()));
        let min_z = self.vertices[0].z().min(f64::min(self.vertices[1].z(), self.vertices[2].z()));
        let max_x = self.vertices[0].x().max(f64::max(self.vertices[1].x(), self.vertices[2].x()));
        let max_y = self.vertices[0].y().max(f64::max(self.vertices[1].y(), self.vertices[2].y()));
        let max_z = self.vertices[0].z().max(f64::max(self.vertices[1].z(), self.vertices[2].z()));

        let min = Vec3::new(min_x, min_y, min_z);
        let max = Vec3::new(max_x, max_y, max_z);

        Some(AABB::new(min, max))
    }
}