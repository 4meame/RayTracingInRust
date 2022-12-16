mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;

use std::io::{stderr, Write};
use rand::Rng;
use vec::{Vec3, Point3, Color};
use ray::Ray;
use hit::{Hit, World};
use sphere::Sphere;
use camera::Camera;

fn ray_color(ray: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // if we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0)
    }

    // 0.001 t_min fixs shadow acne
    if let Some(rec) = world.hit(ray, 0.00001, f64::INFINITY) {
        // 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0))

        // Lambertian:
        // let target = rec.position + rec.normal + Vec3::random_in_unit_sphere().normalized();
        
        // Hemispherical scattering:
        let target = rec.position + Vec3::random_in_hemisphere(rec.normal);
        let r = Ray::new(rec.position, target - rec.position);
        0.5 * ray_color(&r, world, depth - 1)
    } else {
    let unit_direction = ray.direction().normalized();
    let t = 0.5 * (unit_direction.y() + 1.0);
    //lerp white and blue with direction of y
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {
    // image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u64 = 512;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 100;
    const MAX_DEPTH: u64 = 5;

    // world
    let mut world = World::new();
    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // camera
    let camera = Camera::new();
    // let viewport_height = 2.0;
    // let viewport_width = viewport_height * ASPECT_RATIO;
    // let focal_length = 1.0;

    // let origin = Point3::new(0.0, 0.0, 0.0);
    // let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    // let vertical = Vec3::new(0.0, viewport_height, 0.0);
    // let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    println!("P3");
    println!("{} {}",IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    let mut rng = rand::thread_rng();
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

            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let random_u = rng.gen::<f64>();
                let random_v = rng.gen::<f64>();

                let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
                let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);
                
                let r = camera.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }

            println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
    }
    eprintln!("Done.");
}
