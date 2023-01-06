use std::f64;
use rand::Rng;
use super::vec::{Vec3, Point3};
use super::ray::Ray;
use super::hit::{Hittable, HitRecord};
use super::mat::Material;
use super::aabb;
use super::aabb::AABB;
use super::onb::ONB;

fn get_sphere_uv(p: &Vec3) -> (f64, f64) {
    // p: a given point on the sphere of radius one, centered at the origin.
    // u: returned value [0,1] of angle around the Y axis from X=-1.
    // v: returned value [0,1] of angle from Y=-1 to Y=+1.
    //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
    //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
    //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>
    let phi = (-p.z()).atan2(p.x()) + f64::consts::PI;
    let theta = (-p.y()).acos();

    let u = phi / (2.0 * f64::consts::PI);
    let v = theta / f64::consts::PI;

    (u, v)
}

fn random_to_sphere(radius: f64, distance_squared: f64) -> Vec3 {
    let mut rng = rand::thread_rng();
    let r1 = rng.gen::<f64>();
    let r2 = rng.gen::<f64>();
    let z = 1.0 + r2 * ((1.0 - radius.powi(2) / distance_squared).sqrt() - 1.0);
    let phi = 2.0 * f64::consts::PI * r1;
    let x = phi.cos() * (1.0 - z.powi(2)).sqrt();
    let y = phi.sin() * (1.0 - z.powi(2)).sqrt();
    Vec3::new(x, y, z)
}

#[derive(Clone, Copy)]
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

impl<M: Material> Hittable for Sphere<M> {
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
            u: 0.0,
            v: 0.0,
            front_face: false,
            material: &self.material
        };

        let outward_normal = (rec.position - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        let (u, v) = get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;

        Some(rec)
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        let min = self.center - Vec3::new(self.radius, self.radius, self.radius);
        let max = self.center + Vec3::new(self.radius, self.radius, self.radius);

        Some(AABB{min, max})
    }

    fn pdf_value(&self, o: Point3, v: Vec3) -> f64 {
        if let Some(hit) = self.hit(&Ray::new(o, v, 0.0), 0.001, f64::MAX) {
            let cos_theta_max = (1.0 - self.radius.powi(2) / (self.center - o).length().powi(2)).sqrt();
            let solid_angle = 2.0 * f64::consts::PI * (1.0 - cos_theta_max);
            1.0 / solid_angle
        } else {
            0.0
        }
    }

    fn random(&self, o: Vec3) -> Vec3 {
        let direction = self.center - o;
        let distance_squared = direction.length().powi(2);
        let uvw = ONB::build_from_w(&direction);
        uvw.local(&random_to_sphere(self.radius, distance_squared))
    }
}

#[derive(Clone)]
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

impl<M: Material> Hittable for MovingSphere<M> {
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
            u: 0.0,
            v: 0.0,
            front_face: false,
            material: &self.material
        };

        let outward_normal = (rec.position - self.center(r.time())) / self.radius;
        rec.set_face_normal(r, outward_normal);

        let (u, v) = get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;

        Some(rec)
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        let min0 = self.center0 - Vec3::new(self.radius, self.radius, self.radius);
        let max0 = self.center0 + Vec3::new(self.radius, self.radius, self.radius);
        let min1 = self.center1 - Vec3::new(self.radius, self.radius, self.radius);
        let max1 = self.center1 + Vec3::new(self.radius, self.radius, self.radius);

        let box0 = AABB::new(min0, max0);
        let box1 = AABB::new(min1, max1);

        Some(aabb::surrounding_box(&box0, &box1))
    }
}