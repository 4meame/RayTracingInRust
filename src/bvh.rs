use std::cmp::Ordering;
use super::aabb;
use super::aabb::AABB;
use super::hit::{Hit, HitRecord};
use super::ray::Ray;

enum BVHNode {
    Branch { left: Box<BVH>, right: Box<BVH> },
    Leaf(Box<dyn Hit>)
}

pub struct BVH {
    tree: BVHNode,
    bbox: AABB
}

impl BVH {
    pub fn new(mut hit: Vec<Box<dyn Hit>>, time0: f64, time1: f64) -> BVH {
        fn box_compare(time0: f64, time1: f64, axis: usize) -> impl FnMut(&Box<dyn Hit>, &Box<dyn Hit>) -> Ordering {
            move |a, b| {
                let a_bbox = a.bounding_box(time0, time1);
                let b_bbox = b.bounding_box(time0, time1);
                if let (Some(a), Some(b)) = (a_bbox, b_bbox) {
                    let ac = a.min[axis] + a.max[axis];
                    let bc = b.min[axis] + b.max[axis];
                    ac.partial_cmp(&bc).unwrap()
                } else {
                    panic!("no bounding box in bvh node")
                }
            }
        }

        fn axis_range(hit: &Vec<Box<dyn Hit>>, time0: f64, time1: f64, axis: usize) -> f64 {
            let (min, max) = hit.iter().fold((f64::MAX, f64::MIN), |(bmin, bmax), hit| {
                if let Some(aabb) = hit.bounding_box(time0, time1) {
                    (bmin.min(aabb.min[axis]), bmax.max(aabb.max[axis]))
                } else {
                    (bmin, bmax)
                }
            });
            max - min
        }

        // find the axis and the greatest range for this set of objects by Closure and Iterator
        let mut axis_ranges: Vec<(usize, f64)> = (0..3).map(|a| (a, axis_range(&hit, time0, time1, a))).collect();
        // reversed comparison function, to sort descending:
        axis_ranges.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());     
        let axis = axis_ranges[0].0;

        // sort objects along it by  widest extension of bounding box along that axis
        hit.sort_unstable_by(box_compare(time0, time1, axis));

        let length = hit.len();
        match length {
            0 => panic!("no object in the scene"),
            1 => {
                let leaf = hit.pop().unwrap();
                if let Some(bbox) = leaf.bounding_box(time0, time1) {
                    BVH { tree: BVHNode::Leaf(leaf), bbox }
                } else {
                    panic!("no bounding box in bvh node")
                }
            },
            _ => {
                let right = BVH::new(hit.drain(length/2..).collect(), time0, time1);
                // half the hit moved
                let left = BVH::new(hit, time0, time1);
                let bbox = aabb::surrounding_box(&left.bbox, &right.bbox);
                BVH { tree: BVHNode::Branch { left: Box::new(left), right: Box::new(right) }, bbox }
            }
        }

    }
}

impl Hit for BVH {
    fn hit(&self, r: &Ray, t_min: f64, mut t_max: f64) -> Option<HitRecord> {
        if self.bbox.hit(r, t_min, t_max) {
            match &self.tree {
                BVHNode::Branch { left, right } => {
                    let left = left.hit(&r, t_min, t_max);
                    if let Some(l) = &left {t_max = l.t};
                    let right = right.hit(&r, t_min, t_max);
                    if right.is_some() { right } else { left }
                },
                BVHNode::Leaf(leaf) => leaf.hit(&r, t_min, t_max),
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(self.bbox)
    }
}