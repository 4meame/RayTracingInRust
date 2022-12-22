use super::vec::{Vec3, Point3};
use super::ray::Ray;
use super::hit::{Hit, HitRecord};
use super::mat::Material;

pub struct Sphere<M: Material> {
    center: Point3,
    radius: f64,
    material: M
}

impl<M: Material> Sphere<M> {
    pub fn new(center: Point3, radius: f64, material: M) -> Self {
        Sphere {
            center,
            radius,
            material
        }
    }
}

impl<M: Material> Hit for Sphere<M> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius.powi(2);
        let discriminant = half_b.powi(2) -  a * c;
        if discriminant < 0.0 {
            return None
        }

        //find the nearest root that lies in the acceptable range
        let sqrt_d = discriminant.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max {
                return None
            }
        }

        let p = r.at(root);
        let mut rec = HitRecord {
            position: p,
            normal: Vec3::new(0.0, 0.0, 0.0),
            t: root,
            front_face: false,
            material: &self.material
        };

        let outward_normal = (rec.position - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

pub struct MovingSphere<M: Material> {
    center0: Point3,
    center1: Point3,
    time0: f64,
    time1: f64,
    radius: f64,
    material: M
}

impl<M: Material> MovingSphere<M> {
    pub fn new(center0: Point3, center1: Point3, time0: f64, time1: f64, radius: f64, material: M) -> Self {
        MovingSphere {
            center0,
            center1,
            time0,
            time1,
            radius,
            material
        }
    }

    pub fn center(&self, time: f64) -> Point3 {
        self.center0 + (time - self.time0) / (self.time1 - self.time0) * (self.center1 - self.center0)
    }
}

impl<M: Material> Hit for MovingSphere<M> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center(r.time());
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius.powi(2);
        let discriminant = half_b.powi(2) -  a * c;
        if discriminant < 0.0 {
            return None
        }

        //find the nearest root that lies in the acceptable range
        let sqrt_d = discriminant.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max {
                return None
            }
        }

        let p = r.at(root);
        let mut rec = HitRecord {
            position: p,
            normal: Vec3::new(0.0, 0.0, 0.0),
            t: root,
            front_face: false,
            material: &self.material
        };

        let outward_normal = (rec.position - self.center(r.time())) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}