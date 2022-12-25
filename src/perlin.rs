use rand::Rng;
use super::vec::{Vec3, Point3};

fn generate_float() -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut f = Vec::with_capacity(256);
    for _ in 0..256 {
        f.push(rng.gen_range(0.0..1.0))
    }
    f
}

fn generate_vector() -> Vec<Vec3> {
    let mut v = Vec::with_capacity(256);
    for _ in 0..256 {
        v.push(Vec3::random_in_unit_sphere())
    }
    v
}

fn permute(a: &mut [usize], n: usize) {
    let mut rng = rand::thread_rng();
    for i in (0..n as usize).rev() {
        let target = rng.gen_range(0..=i);
        // swap the two elements in the slice
        a.swap(i, target);
    }
}

fn generate_perm() -> Vec<usize> {
    let mut p = Vec::with_capacity(256);
    for i in 0..256 {
        p.push(i);
    }
    permute(&mut p, 256);
    p
}

fn perlin_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let uu = u * u * (3.0 - 2.0 * u);
    let vv = v * v * (3.0 - 2.0 * v);
    let ww = w * w * (3.0 - 2.0 * w);
    let mut accum: f64 = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu)) *
                    (j as f64 * vv + (1 - j) as f64 * (1.0 - vv)) *
                    (k as f64 * ww + (1 - k) as f64 * (1.0 - ww)) *
                    c[i][j][k].dot(weight);
            }
        }
    }
    accum
}

pub struct Perlin {
    rd_vec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>
}

impl Perlin {
    pub fn new() -> Perlin {
        Perlin {
            rd_vec: generate_vector(),
            perm_x: generate_perm(),
            perm_y: generate_perm(),
            perm_z: generate_perm()
        }
    }

    pub fn perlin(&self, p: &Point3, scale: f64) -> f64 {
        // let i = (4 * p.x().abs() as usize) & 255;
        // let j = (4 * p.y().abs() as usize) & 255;
        // let k = (4 * p.z().abs() as usize) & 255;

        let mut u = scale * p.x() - f64::floor(scale * p.x());
        let mut v = scale * p.y() - f64::floor(scale * p.y());
        let mut w = scale * p.z() - f64::floor(scale * p.z());
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);
        let i = f64::floor(scale * p.x()) as usize;
        let j = f64::floor(scale * p.y()) as usize;
        let k = f64::floor(scale * p.z()) as usize;

        // self.rd_vec[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
        
        let mut c = [[[Vec3::new(0.0, 0.0, 0.0); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = 
                    self.rd_vec[
                    self.perm_x[(i + di) & 255] ^ 
                    self.perm_y[(j + dj) & 255] ^ 
                    self.perm_z[(k + dk) & 255]
                    ]
                }
            }
        };

        perlin_interp(&c, u, v, w)
    }

    pub fn turb(&self, p: &Vec3, scale: f64, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * self.perlin(&temp_p, scale);
            weight *= 0.5;
            temp_p *= 2.0;
        }
        f64::abs(accum)
    }
}