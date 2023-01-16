use rand::Rng;
use std::f64;
use super::vec::{Vec3, Color};
use super::ray::Ray;
use super::hit::{HitRecord};
use super::texture::Texture;
use super::pdf::PDF;
use super::onb::ONB;

fn schlick_fresnel(u: f64) -> f64 {
    let m = (1.0 - u).clamp(0.0, 1.0);
    let m2 = m.powi(2);
    return m2 * m2 * m
}

fn GTR_1(n_dot_h: f64, a: f64) -> f64 {
    if a >= 1.0 {
        return 1.0 / f64::consts::PI
    } else {
        let a2 = a * a;
        let t = 1.0 + (a2 - 1.0) * n_dot_h * n_dot_h;
        return (a2 - 1.0) / (f64::consts::PI * a2.log2() * t);
    }
}

fn GTR_2(n_dot_h: f64, a: f64) -> f64 {
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * n_dot_h * n_dot_h;
    return a2 / (f64::consts::PI * t * t);
}

fn GTR_2_aniso(n_dot_h: f64, h_dot_x: f64, h_dot_y: f64, ax: f64, ay: f64) -> f64 {
    return 1.0 / (f64::consts::PI * ax * ay * f64::powi((h_dot_x / ax).powi(2) + (h_dot_y / ay).powi(2) + n_dot_h * n_dot_h, 2));
}

fn smithG_GGX(n_dot_v: f64, alphaG: f64) -> f64 {
    let a = alphaG * alphaG;
    let b = n_dot_v * n_dot_v;
    return 1.0 / (n_dot_v + f64::sqrt(a + b - a * b));
}

fn smithG_GGX_aniso(n_dot_v: f64, v_dot_x: f64, v_dot_y: f64, ax: f64, ay: f64) -> f64 {
    return 1.0 / (n_dot_v + f64::sqrt((v_dot_x * ax).powi(2) + (v_dot_y * ay).powi(2) + n_dot_v.powi(2)));
}

fn mon_to_lin(x: Vec3) -> Vec3 {
    return Vec3::new(x.x().powf(2.2), x.y().powf(2.2), x.z().powf(2.2));
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    return a * (1.0 - t) + b * t;
}

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

    //choose disney principled brdf
    fn brdf(&self, r_in: &Ray, r_out: &Ray, rec: &HitRecord) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
}

pub enum ScatterRecord<'a> {
    Specular { specular_ray: Ray, attenuation: Color },
    Scatter { pdf: PDF<'a>, attenuation: Color },
    Microfacet { pdf: PDF<'a> }
}

#[derive(Clone, Copy)]
pub struct PBR<T: Texture> {
    base_color: T,
    metallic: f64,
    subsurface: f64,
    specular: f64,
    roughness: f64,
    specular_tint: f64,
    anisotropic: f64,
    sheen: f64,
    sheen_tint: f64,
    clearcoat: f64,
    clearcoat_gloss: f64
}

impl<T: Texture> PBR<T> {
    pub fn new(base_color: T, metallic: f64, subsurface: f64, specular: f64, roughness: f64, specular_tint: f64, anisotropic: f64, sheen: f64, sheen_tint: f64, clearcoat: f64, clearcoat_gloss: f64) -> PBR<T> {
        PBR {
            base_color,
            roughness,
            metallic,
            subsurface,
            specular,
            specular_tint,
            anisotropic,
            sheen,
            sheen_tint,
            clearcoat,
            clearcoat_gloss,
        }
    }
}

impl<T: Texture> Material for PBR<T> {
    fn scatter_mc_method(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {

        let rec = ScatterRecord::Microfacet { 
            pdf: PDF::brdf_pdf(rec.normal)
        };

        Some(rec)
    }

    fn brdf(&self, r_in: &Ray, r_out: &Ray, rec: &HitRecord) -> Vec3 {
        //inverse direction
        let l= r_in.direction().normalized() * (-1.0);
        let v = r_out.direction().normalized();
        let onb = ONB::build_from_w(&rec.normal);
        let n = onb.w();
        let x = onb.u();
        let y = onb.v();
    
        let n_dot_v = n.dot(v);
        let n_dot_l = n.dot(l);
        if n_dot_l < 0.0 || n_dot_v < 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let h = (l + v).normalized();
        let n_dot_h = n.dot(h);
        let l_dot_h = l.dot(h);

        let cd_lin = mon_to_lin(self.base_color.mapping(rec.u, rec.v, &rec.position));
        //luminance approx
        let cd_lum = 0.3 * cd_lin.x() + 0.6 * cd_lin.y() + 0.1 * cd_lin.z();

        let c_tint = if cd_lum > 0.0 { cd_lin / cd_lum } else { Vec3::new(1.0, 1.0, 1.0) };
        let c_spec0 = (Vec3::new(1.0, 1.0, 1.0).mix(c_tint, self.specular_tint) * 0.08 * self.specular).mix(cd_lin, self.metallic);
        let c_sheen = Vec3::new(1.0, 1.0, 1.0).mix(c_tint, self.sheen_tint);

        // Diffuse fresnel - go from 1 at normal incidence to .5 at grazing
        // and mix in diffuse retro-reflection based on roughness
        let fresnel_l = schlick_fresnel(n_dot_l);
        let fresnel_v = schlick_fresnel(n_dot_v);
        let fresnel_diffuse_90 = 0.5 + 2.0 * l_dot_h * l_dot_h * self.roughness;
        let fresnel_diffuse = mix(1.0, fresnel_diffuse_90, fresnel_l) * mix(1.0, fresnel_diffuse_90, fresnel_v);

        // Based on Hanrahan-Krueger brdf approximation of isotropic bssrdf
        // 1.25 scale is used to (roughly) preserve albedo
        // Fss90 used to "flatten" retroreflection based on roughness
        let fresnel_subface_scatter_90 = l_dot_h * l_dot_h * self.roughness;
        let fresnel_subface_scatter = mix(1.0, fresnel_subface_scatter_90, fresnel_l) * mix(1.0, fresnel_subface_scatter_90, fresnel_v);
        let subface_scatter = 1.25 * (fresnel_subface_scatter * (1.0 / (n_dot_l + n_dot_v) - 0.5) + 0.5);

        // specular
        let aspect = (1.0 - self.anisotropic * 0.9).sqrt();
        let ax = (self.roughness.powi(2) / aspect).max(0.001);
        let ay = (self.roughness.powi(2) * aspect).max(0.001);
        let d_specular = GTR_2_aniso(n_dot_h, h.dot(x), h.dot(y), ax, ay);
        let fresnel_h = schlick_fresnel(l_dot_h);
        let f_specular = c_spec0.mix(Vec3::new(1.0, 1.0, 1.0), fresnel_h);
        let g_specular = smithG_GGX_aniso(n_dot_l, l.dot(x), l.dot(y), ax, ay) * smithG_GGX_aniso(n_dot_v, v.dot(x), v.dot(y), ax, ay);

        // sheen
        let fresnel_sheen = fresnel_h * self.sheen * c_sheen;

        // clearcoat (ior = 1.5 -> F0 = 0.04)
        let d_reflect = GTR_1(n_dot_h, mix(0.1, 0.001, self.clearcoat_gloss));
        let f_reflect = mix(0.04, 1.0, fresnel_h);
        let g_reflect = smithG_GGX(n_dot_l, 0.25) * smithG_GGX(n_dot_v, 0.25);

        return ((1.0 / f64::consts::PI) * mix(fresnel_diffuse, subface_scatter, self.subsurface) * cd_lin + fresnel_sheen)
                * (1.0 - self.metallic)
                + g_specular * f_specular * d_specular + Vec3::new(0.25, 0.25, 0.25) * self.clearcoat * g_reflect * f_reflect * d_reflect;
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