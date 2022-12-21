use super::vec::{Vec3, Point3};
use super::ray::Ray;
use super::mat::Material;

pub struct HitRecord<'a> {
    pub position: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    // explaination: the reference counted smart pointer, Rc<T> type keeps track of the number of references to a value to determine whether or not the value is still in use
    // pub material: Rc<dyn Material>
    pub material: &'a dyn Material
}

pub trait Hit: Sync {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

impl HitRecord<'_> {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) ->() {
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            (-1.0) * outward_normal
        }
    }
}

// explaination: smart pointer type of trait object, Box<T> pointer allow to store data on the heap, what remains on the stack is the pointer to the heap data
pub type World = Vec<Box<dyn Hit>>;

impl Hit for World {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut temp_rec = None;
        let mut cloest_so_far = t_max;

        for object in self {
            if let Some(rec) = object.hit(r, t_min, cloest_so_far) {
                cloest_so_far = rec.t;
                temp_rec = Some(rec);
            }
        }

        temp_rec
    }
}