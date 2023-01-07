mod vec;
mod ray;
mod translate;
mod rotate;
mod hit;
mod sphere;
mod rect;
mod cube;
mod tri;
mod mesh;
mod camera;
mod mat;
mod aabb;
mod bvh;
mod perlin;
mod texture;
mod medium;
mod onb;
mod pdf;

use std::{io::{stderr, Write}};
use rand::Rng;
use rayon::prelude::*;
use vec::{Vec3, Point3, Color};
use ray::Ray;
use translate::Translate;
use rotate::{Axis, Rotate};
use hit::{Hittable, HittableList, FlipNormal};
use sphere::{Sphere, MovingSphere};
use rect::{Plane, AARect};
use cube::Cube;
use tri::Triangle;
use mesh::Mesh;
use camera::Camera;
use mat::{Lambertian, Metal, Dielectric, DiffuseLight, ScatterRecord};
use bvh::BVH;
use texture::{ConstantTexture, CheckTexture, NoiseTexture, ImageTexture};
use medium::ConstantMedium;
use pdf::PDF;

fn ray_color(ray: &Ray, background: Color, world: &Box<dyn Hittable>, lights: &Box<dyn Hittable>, depth: u64) -> Color {
    if depth <= 0 {
        // if we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0)
    }

    // 0.001 t_min fixs shadow acne
    if let Some(rec) = world.hit(ray, 0.00001, f64::INFINITY) {
        // Color::new(1.0, 0.0, 0.0)

        // 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0))

        // Lambertian:
        // let target = rec.position + rec.normal + Vec3::random_in_unit_sphere().normalized();
        
        // Hemispherical scattering:
        // let target = rec.position + Vec3::random_in_hemisphere(rec.normal);

        // let r = Ray::new(rec.position, target - rec.position);
        // 0.5 * ray_color(&r, world, depth - 1)

        let emitted: Color = rec.material.emitted(&rec);

        // sample light
        // let on_light = Point3::new(255.0, 554.0, 277.0);
        // let mut to_light = on_light - rec.position;
        // let distance_squared = to_light.length().powi(2);
        // to_light = to_light.normalized();

        // if to_light.dot(rec.normal) < 0.0 {
        //     return emitted
        // }

        // let light_area = (343.0 - 213.0) * (332.0 - 227.0);
        // let light_cosine = f64::abs(to_light.y());
        // if light_cosine < 0.000001 {
        //     return emitted
        // }

        // let pdf_value = distance_squared / (light_cosine * light_area);
        // let scattered = Ray::new(rec.position, to_light, ray.time());

        // old method
        // if let Some((attenuation, scattered)) = rec.material.scatter(ray, &rec) {
        //     emitted + attenuation * ray_color(&scattered, background, world, lights, depth - 1)
        if let Some(srec) = rec.material.scatter_mc_method(ray, &rec) {

            match srec {
                ScatterRecord::Specular { specular_ray, attenuation } => {
                    return attenuation * ray_color(&specular_ray, background, world, lights, depth - 1)
                }
                ScatterRecord::Scatter { pdf, attenuation } => {
                    let hittable_pdf = PDF::hittable_pdf(rec.position, lights);
                    let mixture_pdf = PDF::mixture_pdf(&hittable_pdf, &pdf);
                    let scattered = Ray::new(rec.position, mixture_pdf.generate(), ray.time());
                    let pdf_value = mixture_pdf.value(scattered.direction());
                    return emitted + attenuation *  rec.material.scattering_pdf(ray, &rec, &scattered) * ray_color(&scattered, background, world, lights, depth - 1) / pdf_value
                }
           }

        } else {
            emitted
        }

    } else {
    // let unit_direction = ray.direction().normalized();
    // let t = 0.5 * (unit_direction.y() + 1.0);

    // //lerp white and blue with direction of y
    // (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
    background
    }
}

fn progress_ray_color(ray: &Ray, background: Color, world: &Box<dyn Hittable>, depth: u64) -> Color {
    if depth <= 0 {
        // if we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0)
    }

    // 0.001 t_min fixs shadow acne
    if let Some(rec) = world.hit(ray, 0.00001, f64::INFINITY) {
        // Color::new(1.0, 0.0, 0.0)

        // 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0))

        // Lambertian:
        let target = rec.position + rec.normal + Vec3::random_in_unit_sphere().normalized();
        
        // Hemispherical scattering:
        // let target = rec.position + Vec3::random_in_hemisphere(rec.normal);

        let r = Ray::new(rec.position, target - rec.position, 1.0);
        0.5 * progress_ray_color(&r, background, world, depth - 1)

    } else {
    // let unit_direction = ray.direction().normalized();
    // let t = 0.5 * (unit_direction.y() + 1.0);

    // //lerp white and blue with direction of y
    // (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
    background
    }
}

fn random_scene() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut rng = rand::thread_rng();
    let mut world: Vec<Box<dyn Hittable>> = Vec::new();

    let ground_mat = Lambertian::new(CheckTexture::new(ConstantTexture::new(Color::new(1.0, 1.0, 1.0)), ConstantTexture::new(Color::new(0.3, 0.3, 1.0))));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Box::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new((a as f64) + rng.gen_range(0.0..0.9),
                                     0.2,
                                     (b as f64) + rng.gen_range(0.0..0.9));

            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Lambertian::new(ConstantTexture::new(albedo));
                let center1 = center + Vec3::new(0.0, rng.gen_range(0.0..0.01), 0.0);
                let sphere = MovingSphere::new(center, center1, 0.0, 1.0, 0.2 ,sphere_mat);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Metal::new(albedo, fuzz);
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Dielectric::new(1.5);
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            }
        }
    }

    let mat1 = Dielectric::new(1.5);
    let mat2 = Lambertian::new(ConstantTexture::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));

    let mut lights = HittableList::default();

    ( Box::new(BVH::new( world, 0.0, 1.0)), Box::new(lights))
}

fn two_spehre() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();

    let top_mat = Lambertian::new(CheckTexture::new(ConstantTexture::new(Color::new(1.0, 1.0, 1.0)), ConstantTexture::new(Color::new(0.3, 0.3, 1.0))));
    let bottom_mat = Lambertian::new(CheckTexture::new(ConstantTexture::new(Color::new(1.0, 1.0, 1.0)), ConstantTexture::new(Color::new(0.3, 0.3, 1.0))));

    let top_sphere = Sphere::new(Point3::new(0.0, 10.0, 0.0), 10.0, top_mat);
    let bottom_sphere = Sphere::new(Point3::new(0.0, -10.0, 0.0), 10.0, bottom_mat);

    world.push(top_sphere);
    world.push(bottom_sphere);

    let mut lights = HittableList::default();

    (Box::new(world), Box::new(lights))
}

fn two_perlin_sphere() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();

    let top_mat = Lambertian::new(NoiseTexture::new(2.0));
    let bottom_mat = Lambertian::new(NoiseTexture::new(2.0));

    //hash goes wrong in negative field, move object to Fitst Quadrant for now
    let top_sphere = Sphere::new(Point3::new(1000.0, 2.0, 1000.0), 2.0, top_mat);
    let bottom_sphere = Sphere::new(Point3::new(1000.0, -1000.0, 1000.0), 1000.0, bottom_mat);

    world.push(top_sphere);
    world.push(bottom_sphere);

    let mut lights = HittableList::default();

    (Box::new(world), Box::new(lights))
}

fn earth() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let image = image::open("earthmap.jpg").expect("image not found").to_rgb8();
    let (width ,height) = image.dimensions();
    let data = image.into_raw();
    let texture = ImageTexture::new(data, width, height);
    let earth = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0, Lambertian::new(texture));
    let mut lights = HittableList::default();
    (Box::new(earth), Box::new(lights))
}

fn light_room() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();

    let bottom_mat = Lambertian::new(ConstantTexture::new(Color::new(0.7, 0.7, 0.7)));
    let top_mat = Lambertian::new(ConstantTexture::new(Color::new(0.0, 0.1843, 0.6549)));
    let emitted = DiffuseLight::new(ConstantTexture::new(Color::new(4.0, 4.0, 4.0)));
    
    let ground = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, bottom_mat);
    let sphere = Sphere::new(Point3::new(0.0, 2.0, 0.0), 2.0, top_mat);
    let plane = AARect::new(Plane::XY, 3.0, 5.0, 1.0, 3.0, -2.0, emitted);

    world.push(ground);
    world.push(sphere);
    world.push(plane.clone());

    let mut lights = HittableList::default();
    lights.push(plane);

    (Box::new(world), Box::new(lights))
}

fn cornell_box() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let red = Lambertian::new(ConstantTexture::new(Color::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(ConstantTexture::new(Color::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(ConstantTexture::new(Color::new(0.12, 0.45, 0.15)));
    let dielectric = Dielectric::new(1.5);
    let metal = Metal::new(Color::new(0.8, 0.85, 0.88), 0.0);
    let light = DiffuseLight::new(ConstantTexture::new(Color::new(15.0, 15.0, 15.0)));

    let rect_light = FlipNormal::new(AARect::new(Plane::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light));

    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, green));
    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, red));
    world.push(rect_light.clone());
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));
    world.push(AARect::new(Plane::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));

    //world.push(Sphere::new(Point3::new(190.0, 90.0, 190.0), 90.0, dielectric));
    world.push(
        Translate::new(
            Rotate::new(Axis::Y,
                        Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white),-18.0), Vec3::new(130.0, 0.0, 65.0)));
    world.push(
        Translate::new(
            Rotate::new(Axis::Y,
                        Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), metal),15.0), Vec3::new(265.0, 0.0, 295.0)));

    lights.push(rect_light);

    (Box::new(world), Box::new(lights))
}

fn cornell_box_with_smoke() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let red = Lambertian::new(ConstantTexture::new(Color::new(0.65, 0.05, 0.05)));
    let white = Lambertian::new(ConstantTexture::new(Color::new(0.73, 0.73, 0.73)));
    let green = Lambertian::new(ConstantTexture::new(Color::new(0.12, 0.45, 0.15)));
    let light = DiffuseLight::new(ConstantTexture::new(Color::new(15.0, 15.0, 15.0)));

    let rect_light = FlipNormal::new(AARect::new(Plane::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light));

    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, green));
    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, red));
    world.push(rect_light.clone());
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));
    world.push(AARect::new(Plane::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));

    let box1 = 
        Translate::new(
            Rotate::new(Axis::Y,
                        Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone()),-18.0), Vec3::new(130.0, 0.0, 65.0));
    let box2 =
        Translate::new(
            Rotate::new(Axis::Y,
                        Cube::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white),15.0), Vec3::new(265.0, 0.0, 295.0));

    world.push(ConstantMedium::new(box1, 0.01, ConstantTexture::new(Color::new(1.0, 1.0, 1.0))));
    world.push(ConstantMedium::new(box2, 0.01, ConstantTexture::new(Color::new(0.0, 0.0, 0.0))));

    lights.push(rect_light);

    (Box::new(world), Box::new(lights))
}

fn cornell_test() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let black = Lambertian::new(ConstantTexture::new(Color::new(0.00, 0.00, 0.00)));
    let white = Lambertian::new(ConstantTexture::new(Color::new(0.73, 0.73, 0.73)));
    let red = Lambertian::new(ConstantTexture::new(Color::new(0.65, 0.05, 0.05)));
    let green = Lambertian::new(ConstantTexture::new(Color::new(0.12, 0.45, 0.15)));
    let blue = Lambertian::new(ConstantTexture::new(Color::new(0.051, 0.459, 1.000)));
    let tomato = Lambertian::new(ConstantTexture::new(Color::new(1.0, 0.39, 0.28)));
    let violet = Lambertian::new(ConstantTexture::new(Color::new(0.93, 0.51, 0.93)));
    let turquoise = Lambertian::new(ConstantTexture::new(Color::new(0.25, 0.88, 0.82)));
    let azure = Lambertian::new(ConstantTexture::new(Color::new(0.94, 1.0, 1.0)));
    let light_mauve = Lambertian::new(ConstantTexture::new(Color::new(0.569, 0.380, 0.949)));
    let neutral_gray = Lambertian::new(ConstantTexture::new(Color::new(0.701, 0.820, 0.800)));
    let cinnamon_buff = Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.749, 0.431)));
    let jasper_red = Lambertian::new(ConstantTexture::new(Color::new(0.980, 0.168, 0.000)));
    let olympic_blue = Lambertian::new(ConstantTexture::new(Color::new(0.310, 0.560, 0.901)));
    let seashell_pink = Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.811, 0.768)));
    let benzol_green = Lambertian::new(ConstantTexture::new(Color::new(0.000, 0.850, 0.451)));
    let spinel_red= Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.302, 0.788)));
    let venice_green = Lambertian::new(ConstantTexture::new(Color::new(0.419, 1.000, 0.702)));
    let nile_blue = Lambertian::new(ConstantTexture::new(Color::new(0.749, 1.000, 0.902)));
    let deep_slate_olive = Lambertian::new(ConstantTexture::new(Color::new(0.090, 0.153, 0.074)));
    let lemon_yellow= Lambertian::new(ConstantTexture::new(Color::new(0.894, 0.941, 0.141)));
    let cotinga_purple = Lambertian::new(ConstantTexture::new(Color::new(0.204, 0.000, 0.349)));
    let helvetia_blue = Lambertian::new(ConstantTexture::new(Color::new(0.000, 0.341, 0.729)));
    let citron_yellow = Lambertian::new(ConstantTexture::new(Color::new(0.650, 0.831, 0.051)));
    let hermosa_pink = Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.702, 0.941)));
    let grayish_lavender= Lambertian::new(ConstantTexture::new(Color::new(0.749, 0.780, 0.800)));
    let carmine = Lambertian::new(ConstantTexture::new(Color::new(0.839, 0.000, 0.212)));
    let deep_slate_green = Lambertian::new(ConstantTexture::new(Color::new(0.059, 0.149, 0.122)));
    let coral_red = Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.451, 0.600)));
    let apricot_peach = Lambertian::new(ConstantTexture::new(Color::new(0.981, 0.811, 0.690)));
    let perano = Lambertian::new(ConstantTexture::new(Color::new(0.663, 0.750, 0.951)));
    let diamond = Lambertian::new(ConstantTexture::new(Color::new(0.732, 0.953, 1.000)));
    let dust_storm = Lambertian::new(ConstantTexture::new(Color::new(0.901, 0.804, 0.788)));
    let bone = Lambertian::new(ConstantTexture::new(Color::new(0.887, 0.851, 0.794)));
    let desire = Lambertian::new(ConstantTexture::new(Color::new(0.922, 0.238, 0.331)));
    let botticelli = Lambertian::new(ConstantTexture::new(Color::new(0.782, 0.873, 0.904)));
    let safety_orange = Lambertian::new(ConstantTexture::new(Color::new(1.000, 0.471, 0.0)));
    let han_purple = Lambertian::new(ConstantTexture::new(Color::new(0.318, 0.090, 0.979)));
    let soft_peach = Lambertian::new(ConstantTexture::new(Color::new(0.961, 0.933, 0.939)));
    let color_e60307 = Lambertian::new(ConstantTexture::new(Color::new(0.902, 0.012, 0.027)));
    let color_80cf00 = Lambertian::new(ConstantTexture::new(Color::new(0.502, 0.812, 0.002)));
    let color_cfb386 = Lambertian::new(ConstantTexture::new(Color::new(0.812, 0.702, 0.525)));

    let dielectric = Dielectric::new(1.1);
    let metal = Metal::new(Color::new(0.8, 0.85, 0.88), 0.01);
    let light0 = DiffuseLight::new(ConstantTexture::new(Color::new(1.0, 1.0, 0.88) * 4.8));
    let light1 = DiffuseLight::new(ConstantTexture::new(Color::new(1.0, 0.91, 0.99) * 1.5));
    let light2 = DiffuseLight::new(ConstantTexture::new(Color::new(0.88, 0.91, 0.99) * 1.3));
    let light3 = DiffuseLight::new(ConstantTexture::new(Color::new(1.0, 0.613, 0.604) * 25.2));
    let light4 = DiffuseLight::new(ConstantTexture::new(Color::new(0.841, 0.813, 0.974) * 58.9));

    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, perano));
    world.push(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, diamond));
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white));
    world.push(AARect::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white));
    world.push(AARect::new(Plane::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white));

    let rect_light0 = FlipNormal::new(AARect::new(Plane::XZ, 128.0, 428.0, 115.0, 270.0, 554.0, light0));
    let rect_light1 = AARect::new(Plane::YZ, 20.0, 45.0, 25.0, 455.0, 1.0, light1);
    let rect_light2 = FlipNormal::new(AARect::new(Plane::YZ, 20.0, 45.0, 25.0, 455.0, 554.0, light2));
    let rect_light3 = FlipNormal::new(AARect::new(Plane::XY, 208.0, 348.0, 373.0, 449.0, 554.0, light3));
    let rect_light4 = AARect::new(Plane::XY, 258.0, 298.0, 66.0, 90.0, 25.0, light4);
    let mirror = AARect::new(Plane::YZ, 30.0, 535.0, 65.0, 490.0, 555.0, metal);
    let spehre0 = Sphere::new(Point3::new(488.0, 455.0, 368.0), 49.0, dielectric);
    let cube0 = Cube::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(175.0, 175.0, 175.0), white);
    let tri0 = Triangle::new([Vec3::new(0.0, 0.0, 465.0), Vec3::new(555.0, 0.0, 465.0), Vec3::new(278.0, 455.0, 555.0)], metal);   
    let obj = Mesh::load_obj("baccante.obj", Vec3::new(278.0, 40.0, 258.0), 15.0, apricot_peach).unwrap();

    //world.push(rect_light0.clone());
    //world.push(rect_light1.clone());
    //world.push(rect_light2.clone());
    world.push(rect_light3.clone());
    //world.push(rect_light4.clone());
    //world.push(mirror);
    //world.push(spehre0);
    //world.push(Translate::new(Rotate::new(Axis::Y, cube0, 30.0), Vec3::new(278.0, 0.0, 156.0)));
    //world.push(tri0);  
    world.push(BVH::new(obj.tris.list, 0.0, 1.0));

    //lights.push(rect_light0);
    //lights.push(rect_light1);
    //lights.push(rect_light2);
    lights.push(rect_light3);
    //lights.push(rect_light4);

    (Box::new(world), Box::new(lights))
}

fn final_scene() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let mut rng = rand::thread_rng();
    let ground = Lambertian::new(ConstantTexture::new(Color::new(0.48, 0.83, 0.53)));
    let mut box_list1: Vec<Box<dyn Hittable>> = Vec::new();
    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = 100.0 * (rng.gen::<f64>() + 0.01);
            let z1 = z0 + w;
            box_list1.push(Box::new(Cube::new(Point3::new(x0, y0, z0), Point3::new(x1, y1, z1), ground.clone())));
        }
    }
    world.push(BVH::new(box_list1, 0.0, 1.0));

    let light = DiffuseLight::new(ConstantTexture::new(Color::new(7.0, 7.0, 7.0)));
    let rect_light = FlipNormal::new(AARect::new(Plane::XZ, 147.0, 412.0, 123.0, 423.0, 554.0, light));
    world.push(rect_light.clone());

    let center = Point3::new(400.0, 400.0, 200.0);
    world.push(MovingSphere::new(center, center + Point3::new(30.0, 0.0, 0.0), 0.0, 1.0, 50.0, Lambertian::new(ConstantTexture::new(Color::new(0.7, 0.3, 0.1)))));
    world.push(Sphere::new(Point3::new(260.0, 150.0, 45.0), 50.0, Dielectric::new(1.5)));
    world.push(Sphere::new(Point3::new(0.0, 150.0, 145.0), 50.0, Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)));

    let boundary = Sphere::new(Point3::new(360.0, 150.0, 145.0), 70.0, Dielectric::new(1.5));
    world.push(boundary.clone());
    world.push(ConstantMedium::new(boundary, 0.2, ConstantTexture::new(Color::new(0.2, 0.4, 0.9))));

    let boundary = Sphere::new(Point3::new(0.0, 0.0, 0.0), 5000.0, Dielectric::new(1.5));
    world.push(ConstantMedium::new(boundary, 0.0001, ConstantTexture::new(Color::new(1.0, 1.0, 1.0))));

    let image = image::open("earthmap.jpg").expect("image not found").to_rgb8();
    let (nx, ny) = image.dimensions();
    let data = image.into_raw();
    let texture = ImageTexture::new(data, nx, ny);
    world.push(Sphere::new(Point3::new(400.0, 200.0, 400.0), 100.0, Lambertian::new(texture)));
    world.push(Sphere::new(Point3::new(220.0, 280.0, 300.0), 80.0, Lambertian::new(NoiseTexture::new(0.1))));

    let white = Lambertian::new(ConstantTexture::new(Color::new(0.73, 0.73, 0.73)));
    let mut box_list2: Vec<Box<dyn Hittable>> = Vec::new();
    let ns = 1000;
    for _ in 0..ns {
        box_list2.push(Box::new(Sphere::new(Point3::new(165.0 * rng.gen::<f64>(), 165.0 * rng.gen::<f64>(), 165.0 * rng.gen::<f64>()), 10.0, white.clone())));
    }
    world.push(
        Translate::new(
            Rotate::new(Axis::Y, BVH::new(box_list2, 0.0, 0.1), 15.0),
                Point3::new(-100.0, 270.0, 395.0))
    );

    lights.push(rect_light);

    (Box::new(world), Box::new(lights))
}

pub fn progress_showcase() -> (Box<dyn Hittable>, Box<dyn Hittable>) {
    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    // let ground_mat = Lambertian::new(CheckTexture::new(ConstantTexture::new(Color::new(1.0, 1.0, 1.0)), ConstantTexture::new(Color::new(0.04, 0.01, 0.02))));
    // let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    // let white = Lambertian::new(ConstantTexture::new(Color::new(0.73, 0.73, 0.73)));
    // let green = Lambertian::new(ConstantTexture::new(Color::new(0.12, 0.45, 0.15)));
    // let tomato = Lambertian::new(ConstantTexture::new(Color::new(1.0, 0.39, 0.28)));
    // let violet = Lambertian::new(ConstantTexture::new(Color::new(0.93, 0.51, 0.93)));
    // let red = Lambertian::new(ConstantTexture::new(Color::new(0.65, 0.05, 0.05)));
    // let azure = Lambertian::new(ConstantTexture::new(Color::new(0.94, 1.0, 1.0)));
    // let dielectric = Dielectric::new(1.5);
    // let metal = Metal::new(Color::new(0.8, 0.85, 0.88), 0.0);
    // let glossy = Metal::new(Color::new(1.0, 0.4, 0.0), 0.3);
    // let light = DiffuseLight::new(ConstantTexture::new(Color::new(1.0, 1.0, 0.88) * 25.0));

    // let sphere_0 = Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, green);
    // let sphere_1 = Sphere::new(Vec3::new(-1.7, 1.0, 1.7), 1.0, metal);
    // let sphere_2 = Sphere::new(Vec3::new(1.7, 1.0, -1.7), 1.0, violet);
    // let sphere_3 = Sphere::new(Vec3::new(3.7, 0.0, 5.4), 3.4, dielectric);
    // let sphere_4 = Sphere::new(Vec3::new(3.7, 0.0, 5.4), -3.3, dielectric);
    // let sphere_5 = Sphere::new(Vec3::new(-3.3, 2.4, -2.9), 0.3, light);
    // let sphere_6 = Sphere::new(Vec3::new(-1.2, 0.4, -1.8), 0.5, glossy);

    // let plane_0 = AARect::new(Plane::YZ, 0.0, 0.7, 0.1, 0.3, 0.0, tomato);

    // let triangle = Triangle::new([Vec3::new(-2.4, 3.3, 4.6), Vec3::new(-3.2, 1.3, 2.1), Vec3::new(-0.8, 1.5, 3.4)], red);

    // world.push(sphere_0);
    // world.push(sphere_1);
    // world.push(sphere_2);
    // world.push(ground_sphere);
    // world.push(Translate::new(Rotate::new(Axis::Y, plane_0, 104.0), Vec3::new(0.5, 1.9, 1.7)));
    // world.push(triangle);
    // world.push(sphere_3);
    // world.push(sphere_4);
    // world.push(sphere_5);
    // world.push(sphere_6);

    // // let boundary = Sphere::new(Point3::new(0.0, 0.0, 0.0), 5000.0, Dielectric::new(1.5));
    // // world.push(ConstantMedium::new(boundary, 0.0001, ConstantTexture::new(Color::new(1.0, 1.0, 1.0))));

    // lights.push(sphere_5);

    (Box::new(world), Box::new(lights))
}

enum Scene {
    Random,
    TwoSphere,
    TwoPerlinSphere,
    Earth,
    LightRoom,
    CornellBox,
    CornellSmoke,
    CornellTest,
    FinalScene,
    Progress
}

fn main() {
    // image
    const ASPECT_RATIO: f64 = 1.0;
    const IMAGE_WIDTH: u64 = 1000;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 2000;
    const MAX_DEPTH: u64 = 100;

    // world
    // let mut world = World::new();
    // let mat_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    // let mat_center = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    // let mat_left = Rc::new(Dielectric::new(1.5));
    // let mat_left_inner = Rc::new(Dielectric::new(1.5));
    // let mat_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.1));

    // let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground);
    // let sphere_center = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center);
    // let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left);
    // let sphere_left_inner = Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.46, mat_left_inner);
    // let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right);

    // world.push(Box::new(sphere_ground));
    // world.push(Box::new(sphere_center));
    // world.push(Box::new(sphere_left));
    // world.push(Box::new(sphere_left_inner));
    // world.push(Box::new(sphere_right));

    // let world = random_scene();

    // camera
    // let lookfrom = Point3::new(13.0, 2.0, 3.0);
    // let lookat = Point3::new(0.0, 0.0, 0.0);
    // let vup = Vec3::new(0.0, 1.0, 0.0);
    // let dist_to_focus = 10.0;
    // let aperture = 0.1;
    // let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);
    // let viewport_height = 2.0;
    // let viewport_width = viewport_height * ASPECT_RATIO;
    // let focal_length = 1.0;

    // let origin = Point3::new(0.0, 0.0, 0.0);
    // let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    // let vertical = Vec3::new(0.0, viewport_height, 0.0);
    // let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    let scene: Scene = Scene::CornellTest;
    let (world, background, lights, camera) = match scene {
        Scene::Random => {
            let (world, lights) = final_scene();

            let backgournd = Color::new(0.7, 0.8, 1.0);

            let lookfrom = Point3::new(13.0, 2.0, 3.0);
            let lookat = Point3::new(0.0, 0.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::TwoSphere =>{
            let (world, lights) = two_spehre();

            let backgournd = Color::new(0.7, 0.8, 1.0);

            let lookfrom = Point3::new(13.0, 2.0, 3.0);
            let lookat = Point3::new(0.0, 0.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::TwoPerlinSphere => {
            let (world, lights) = two_perlin_sphere();

            let backgournd = Color::new(0.7, 0.8, 1.0);

            let lookfrom = Point3::new(1013.0, 2.0, 1003.0);
            let lookat = Point3::new(1000.0, 0.0, 1000.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::Earth => {
            let (world, lights) = earth();

            let backgournd = Color::new(0.7, 0.8, 1.0);

            let lookfrom = Point3::new(13.0, 2.0, 3.0);
            let lookat = Point3::new(0.0, 0.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::LightRoom => {
            let (world, lights) = light_room();

            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(26.0, 3.0, 6.0);
            let lookat = Point3::new(0.0, 2.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let camera = Camera::new(lookfrom, lookat, vup, 20.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::CornellBox => {
            let (world, lights) = cornell_box();
            
            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(278.0, 278.0, -800.0);
            let lookat = Point3::new(278.0, 278.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.05;
            let camera = Camera::new(lookfrom, lookat, vup, 40.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::CornellSmoke => {
            let (world, lights) = cornell_box_with_smoke();
            
            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(278.0, 278.0, -800.0);
            let lookat = Point3::new(278.0, 278.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.05;
            let camera = Camera::new(lookfrom, lookat, vup, 40.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::CornellTest => {
            let (world, lights) = cornell_test();
            
            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(278.0, 278.0, -800.0);
            let lookat = Point3::new(278.0, 278.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.01;
            let camera = Camera::new(lookfrom, lookat, vup, 40.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::FinalScene => {
            let (world, lights) = final_scene();
            
            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(478.0, 278.0, -600.0);
            let lookat = Point3::new(278.0, 278.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 10.0;
            let aperture = 0.01;
            let camera = Camera::new(lookfrom, lookat, vup, 40.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
        Scene::Progress => {
            let (world, lights) = progress_showcase();

            let backgournd = Color::new(0.0, 0.0, 0.0);

            let lookfrom = Point3::new(-3.3, 6.8, -9.8);
            let lookat = Point3::new(0.0, 1.0, 0.0);
            let vup = Vec3::new(0.0, 1.0, 0.0);
            let dist_to_focus = 12.0;
            let aperture = 0.2;
            let camera = Camera::new(lookfrom, lookat, vup, 40.0, ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

            (world, backgournd, lights, camera)
        }
    };

    println!("P3");
    println!("{} {}",IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    //let mut rng = rand::thread_rng();
    for j in (0..IMAGE_HEIGHT).rev() {
        //adding a progress indicator
        eprint!("\rScanlines remaining: {:3}", IMAGE_HEIGHT - j - 1);
        stderr().flush().unwrap();

        for i in 0..IMAGE_WIDTH {

            // let r = i as f64 / (IMAGE_WIDTH - 1) as f64;
            // let g = j as f64 / (IMAGE_HEIGHT - 1) as f64;
            // let b = 0.25;

            // let ir = (255.999 * r) as u64;
            // let ig = (255.999 * g) as u64;
            // let ib = (255.999 * b) as u64;

            // println!("{} {} {}", ir, ig, ib);
            
            // let pixel_color = Color::new(i as f64 / (IMAGE_WIDTH - 1) as f64,
            //                                    j as f64 / (IMAGE_HEIGHT - 1) as f64,
            //                                    0.25);
            
            // let u = (i as f64) / ((IMAGE_WIDTH - 1) as f64);
            // let v = (j as f64) / ((IMAGE_HEIGHT - 1) as f64);

            // let r = Ray::new(origin, lower_left_corner + u * horizontal + v * vertical - origin);
            // let pixel_color = ray_color(&r, &world);

            // let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            // for _ in 0..SAMPLES_PER_PIXEL {
            //     let random_u = rng.gen::<f64>();
            //     let random_v = rng.gen::<f64>();

            //     let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
            //     let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);
                
            //     let r = camera.get_ray(u, v);
            //     pixel_color += ray_color(&r, &world, MAX_DEPTH);
            // }          

            let pixel_color: Color = (0..SAMPLES_PER_PIXEL).into_par_iter().map(|_sample| {
                
                let mut rng = rand::thread_rng();
                let random_u = rng.gen::<f64>();
                let random_v = rng.gen::<f64>();

                let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
                let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);

                let r = camera.get_ray(u, v);

                // let unit_direction = r.direction().normalized();
                // let t = 0.5 * (unit_direction.y() + 1.0);
                // //lerp white and blue with direction of y
                // let backgournd = (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);

                ray_color(&r, background, &world, &lights, MAX_DEPTH)
                // progress_ray_color(&r, background, &world, MAX_DEPTH)
            })
            .sum();
            
            println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
    }
    eprintln!("Done.");
}
