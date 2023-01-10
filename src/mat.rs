use rand::Rng;
use std::f64;
use super::vec::{Vec3, Color};
use super::ray::Ray;
use super::hit::{HitRecord};
use super::texture::Texture;
use super::pdf::PDF;

pub trait Material: Sync {
    // old method
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        None
    }

    // mc method
    fn scatter_mc_method(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
    }

    fn emitted(&self, rec: &HitRecord) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    fn brdf(&self) -> f64 {
        0.0
    }
}

pub enum ScatterRecord<'a> {
    Specular { specular_ray: Ray, attenuation: Color },
    Scatter { pdf: PDF<'a>, attenuation: Color },
    Microfacet { pdf: PDF<'a>, attenuation: Color }
}

#[derive(Clone, Copy)]
pub struct PBR<T: Texture> {
    albedo: T
}

impl<T: Texture> PBR<T> {
    pub fn new(albedo: T) -> PBR<T> {
        PBR {
            albedo
        }
    }
}

impl<T: Texture> Material for PBR<T> {
    fn scatter_mc_method(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {

        let rec = ScatterRecord::Microfacet { 
            pdf: PDF::brdf_pdf(rec.normal),
            attenuation: self.albedo.mapping(rec.u, rec.v, &rec.position)
        };

        Some(rec)
}

    fn brdf(&self) -> f64 {
        1.0
    }
}

#[derive(Clone, Copy)]
pub struct Lambertian<T: Texture> {
    albedo: T
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Lambertian<T> {
        Lambertian {
            albedo
        }
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere().normalized();
        if scatter_direction.near_zero() {
            // Catch degenerate scatter direction
            scatter_direction = rec.normal;
        }

        let scattered = Ray::new(rec.position, scatter_direction, _r_in.time());

        Some((self.albedo.mapping(rec.u, rec.v, &rec.position), scattered))
    }

    fn scatter_mc_method(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        // let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere();
        
        // if scatter_direction.near_zero() {
        //     // catch degenerate scatter direction
        //     scatter_direction = rec.normal;
        // }

        // let scattered = Ray::new(rec.position, scatter_direction, _r_in.time());

        // // importance sampling
        // let pdf = rec.normal.dot(scattered.direction()) / f64::consts::PI;

        let rec = ScatterRecord::Scatter { 
            pdf: PDF::cosine_pdf(rec.normal),
            attenuation: self.albedo.mapping(rec.u, rec.v, &rec.position)
        };

        Some(rec)
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = rec.normal.dot(scattered.direction().normalized()).max(0.0);
        cosine / f64::consts::PI
    }
}


#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Color,
    fuzz: f64
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal {
            albedo,
            fuzz
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r_in.direction().reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.position, reflected + self.fuzz * Vec3::random_in_unit_sphere(), r_in.time());

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }

    fn scatter_mc_method(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let reflected = r_in.direction().reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.position, reflected + self.fuzz * Vec3::random_in_unit_sphere(), r_in.time());

        if scattered.direction().dot(rec.normal) > 0.0 {
            let rec = ScatterRecord::Specular { 
                specular_ray: scattered,
                attenuation: self.albedo
            };
            Some(rec)
        } else {
            None
        }
    }
}


#[derive(Clone, Copy)]
pub struct Dielectric {
    ir: f64
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Dielectric { 
            ir: index_of_refraction
        }
    }

    fn reflectance(cosine: f64, index_of_refraction: f64) -> f64 {
        // use Schlick's approximation
        let r0 = ((1.0 - index_of_refraction) / (1.0 + index_of_refraction)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().normalized();
        
        let cos_theta = ((-1.0) * unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            unit_direction.reflect(rec.normal)
        } else {
            unit_direction.refract(rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.position, direction, r_in.time());
        Some((Color::new(1.0, 1.0, 1.0), scattered))
    }

    fn scatter_mc_method(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().normalized();
        
        let cos_theta = ((-1.0) * unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            unit_direction.reflect(rec.normal)
        } else {
            unit_direction.refract(rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.position, direction, r_in.time());

        let rec = ScatterRecord::Specular { 
            specular_ray: scattered,
            attenuation
        };

        Some(rec)
    }
}

#[derive(Clone, Copy)]
pub struct DiffuseLight<T: Texture> {
    emit: T
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(emit: T) -> DiffuseLight<T> {
        DiffuseLight { 
            emit
         }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<(Color, Ray)> {
        None
    }

    fn emitted(&self, rec: &HitRecord) -> Color {
        if rec.front_face {
            self.emit.mapping(rec.u, rec.v, &rec.position)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Isotropic<T: Texture> {
    albedo: T
}

impl<T: Texture> Isotropic<T> {
    pub fn new(albedo: T) -> Isotropic<T> { 
        Isotropic { 
            albedo 
        } 
    }
}

impl<T: Texture> Material for Isotropic<T> {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let scattered = Ray::new(rec.position, Vec3::random_in_unit_sphere(), r_in.time());
        Some((self.albedo.mapping(rec.u, rec.v, &rec.position), scattered))
    }
}