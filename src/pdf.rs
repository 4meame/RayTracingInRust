use std::f64;
use rand::Rng;
use super::hit::Hittable;
use super::vec::{Vec3, Point3};
use super::onb::ONB;

fn random_cosine_direction() -> Vec3 {
    let mut rng = rand::thread_rng();
    let r1 = rng.gen::<f64>();
    let r2 = rng.gen::<f64>();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * f64::consts::PI * r1;
    let x= f64::cos(phi) * r2.sqrt();
    let y = f64::sin(phi) * r2.sqrt();

    Vec3::new(x, y, z)
}

pub enum PDF<'a> {
    BRDF {},
    Cosine { uvw: ONB },
    Hittable { origin: Point3, hittable: &'a Box<dyn Hittable> },
    Mixture { p0: &'a PDF<'a>, p1: &'a PDF<'a> }
}

impl<'a> PDF<'a> {
    pub fn brdf_pdf(w: Vec3)-> PDF<'a> {
        PDF::BRDF {
            
        }
    }

    pub fn cosine_pdf(w: Vec3) -> PDF<'a> {
        PDF::Cosine {
            uvw: ONB::build_from_w(&w)
        }
    }

    pub fn hittable_pdf(origin: Point3, hittable: &'a Box<dyn Hittable>) -> PDF<'a> {
        PDF::Hittable { origin, hittable }
    }

    pub fn mixture_pdf(p0: &'a PDF, p1: &'a PDF) -> PDF<'a> {
        PDF::Mixture { p0, p1 }
    }

    pub fn value(&self, direction: Vec3) -> f64 {
        match self {
            PDF::BRDF {  } => todo!(),
            PDF::Cosine { uvw } => {
                let cosine = direction.normalized().dot(uvw.w());
                if cosine > 0.0 {
                    // importance sampling
                    cosine / f64::consts::PI
                } else {
                    0.0
                }
            },
            PDF::Hittable { origin, hittable } => {
                hittable.pdf_value(*origin, direction)
            },
            PDF::Mixture { p0, p1 } => {
                0.5 * p0.value(direction) + 0.5 * p1.value(direction)
            }
        }
    }

    pub fn generate(&self) -> Vec3 {
        match self {
            PDF::BRDF {  } => todo!(),
            PDF::Cosine { uvw } => {
                uvw.local(&random_cosine_direction())
            },
            PDF::Hittable { origin, hittable } => {
                hittable.random(*origin)
            },
            PDF::Mixture { p0, p1 } => {
                let mut rng = rand::thread_rng();
                if rng.gen::<bool>() {
                    p0.generate()
                } else {
                    p1.generate()
                }
            }
        }
    }
}