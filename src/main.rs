use std::{
    fs::File,
    io::{BufWriter, Write},
};

const WIDTH: usize = 960;
const HEIGHT: usize = 640;
const EPS: f32 = 0.0001;

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn normalized(self) -> Self {
        self / self.len().max(EPS)
    }
    fn mix(self, b: Self, t: f32) -> Self {
        self * (1.0 - t) + b * t
    }
    fn clamp01(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }
}
impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}
impl std::ops::Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, s: f32) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s)
    }
}
impl std::ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}
impl Ray {
    fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
struct Material {
    color: Vec3,
    roughness: f32,
    reflectivity: f32,
}

#[derive(Clone, Copy)]
enum Primitive {
    Sphere {
        c: Vec3,
        r: f32,
        m: usize,
    },
    Ellipsoid {
        c: Vec3,
        r: Vec3,
        m: usize,
    },
    Capsule {
        a: Vec3,
        b: Vec3,
        r: f32,
        m: usize,
    },
    CylinderY {
        c: Vec3,
        r: f32,
        half: f32,
        m: usize,
    },
    Ground,
}

#[derive(Clone, Copy)]
struct Hit {
    t: f32,
    p: Vec3,
    n: Vec3,
    m: usize,
}

fn sphere_hit(ray: Ray, c: Vec3, r: f32) -> Option<(f32, Vec3)> {
    let oc = ray.origin - c;
    let b = oc.dot(ray.direction);
    let h = b * b - oc.dot(oc) + r * r;
    if h < 0.0 {
        return None;
    }
    let h = h.sqrt();
    let mut t = -b - h;
    if t < EPS {
        t = -b + h
    }
    if t < EPS {
        return None;
    }
    let p = ray.at(t);
    Some((t, (p - c) / r))
}

fn ellipsoid_hit(ray: Ray, c: Vec3, r: Vec3) -> Option<(f32, Vec3)> {
    let ro = Vec3::new(
        (ray.origin.x - c.x) / r.x,
        (ray.origin.y - c.y) / r.y,
        (ray.origin.z - c.z) / r.z,
    );
    let rd = Vec3::new(
        ray.direction.x / r.x,
        ray.direction.y / r.y,
        ray.direction.z / r.z,
    );
    let a = rd.dot(rd);
    let b = ro.dot(rd);
    let h = b * b - a * (ro.dot(ro) - 1.0);
    if h < 0.0 {
        return None;
    }
    let h = h.sqrt();
    let mut t = (-b - h) / a;
    if t < EPS {
        t = (-b + h) / a
    }
    if t < EPS {
        return None;
    }
    let p = ray.at(t) - c;
    let n = Vec3::new(p.x / (r.x * r.x), p.y / (r.y * r.y), p.z / (r.z * r.z)).normalized();
    Some((t, n))
}

fn capsule_hit(ray: Ray, pa: Vec3, pb: Vec3, radius: f32) -> Option<(f32, Vec3)> {
    let ba = pb - pa;
    let oa = ray.origin - pa;
    let baba = ba.dot(ba);
    let bard = ba.dot(ray.direction);
    let baoa = ba.dot(oa);
    let rdoa = ray.direction.dot(oa);
    let oaoa = oa.dot(oa);
    let aa = baba - bard * bard;
    let bb = baba * rdoa - baoa * bard;
    let cc = baba * oaoa - baoa * baoa - radius * radius * baba;
    let h = bb * bb - aa * cc;
    if h >= 0.0 && aa.abs() > EPS {
        let t = (-bb - h.sqrt()) / aa;
        let y = baoa + t * bard;
        if t > EPS && y > 0.0 && y < baba {
            let p = ray.at(t);
            let q = pa + ba * (y / baba);
            return Some((t, (p - q).normalized()));
        }
    }
    let cap = if baoa < 0.0 { pa } else { pb };
    sphere_hit(ray, cap, radius)
}

fn cylinder_y_hit(ray: Ray, c: Vec3, radius: f32, half: f32) -> Option<(f32, Vec3)> {
    let o = ray.origin - c;
    let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
    let b = o.x * ray.direction.x + o.z * ray.direction.z;
    let cc = o.x * o.x + o.z * o.z - radius * radius;
    let mut best: f32 = f32::INFINITY;
    let mut normal = Vec3::default();
    let h = b * b - a * cc;
    if h >= 0.0 && a > EPS {
        for t in [(-b - h.sqrt()) / a, (-b + h.sqrt()) / a] {
            let y = o.y + t * ray.direction.y;
            if t > EPS && y.abs() <= half && t < best {
                best = t;
                normal = Vec3::new(o.x + t * ray.direction.x, 0.0, o.z + t * ray.direction.z)
                    .normalized();
            }
        }
    }
    if ray.direction.y.abs() > EPS {
        for sy in [-1.0, 1.0] {
            let t = (sy * half - o.y) / ray.direction.y;
            let p = o + ray.direction * t;
            if t > EPS && p.x * p.x + p.z * p.z <= radius * radius && t < best {
                best = t;
                normal = Vec3::new(0.0, sy, 0.0);
            }
        }
    }
    if best.is_finite() {
        Some((best, normal))
    } else {
        None
    }
}

fn material(color: Vec3, roughness: f32, reflectivity: f32) -> Material {
    Material {
        color,
        roughness,
        reflectivity,
    }
}

struct Scene {
    objects: Vec<Primitive>,
    materials: Vec<Material>,
}

fn add_tree(scene: &mut Scene, x: f32, z: f32, h: f32, r: f32, seed: f32) {
    let base = Vec3::new(x, -0.9, z);
    let top = Vec3::new(x + 0.12 * (seed * 2.3).sin(), -0.9 + h, z);
    scene.objects.push(Primitive::Capsule {
        a: base,
        b: top,
        r,
        m: 1,
    });
    // Corona dicotómica de licopodio: ramas y masas de microfilos.
    for j in 0..8 {
        let a = j as f32 * 2.399 + seed;
        let up = if j % 2 == 0 { 0.58 } else { 0.25 };
        let start = top + Vec3::new(0.0, -0.25, 0.0);
        let end = top + Vec3::new(a.cos() * h * 0.18, up, a.sin() * h * 0.18);
        scene.objects.push(Primitive::Capsule {
            a: start,
            b: end,
            r: r * 0.32,
            m: 2,
        });
        scene.objects.push(Primitive::Ellipsoid {
            c: end,
            r: Vec3::new(r * 2.0, r * 1.15, r * 2.0),
            m: 3,
        });
    }
}

fn add_calamites(scene: &mut Scene, x: f32, z: f32, h: f32, seed: f32) {
    scene.objects.push(Primitive::CylinderY {
        c: Vec3::new(x, -0.9 + h * 0.5, z),
        r: 0.09,
        half: h * 0.5,
        m: 4,
    });
    for j in 1..7 {
        let y = -0.8 + h * j as f32 / 7.0;
        for k in 0..5 {
            let a = k as f32 * 1.256 + seed + j as f32 * 0.37;
            let start = Vec3::new(x, y, z);
            let end = start + Vec3::new(a.cos() * 0.62, 0.11, a.sin() * 0.62);
            scene.objects.push(Primitive::Capsule {
                a: start,
                b: end,
                r: 0.025,
                m: 5,
            });
            scene.objects.push(Primitive::Ellipsoid {
                c: end,
                r: Vec3::new(0.18, 0.045, 0.07),
                m: 5,
            });
        }
    }
}

fn add_fern(scene: &mut Scene, x: f32, z: f32, scale: f32, seed: f32) {
    for frond in 0..5 {
        let a = seed + frond as f32 * 1.256;
        let root = Vec3::new(x, -0.82, z);
        let tip = root + Vec3::new(a.cos() * scale, scale * 0.75, a.sin() * scale);
        scene.objects.push(Primitive::Capsule {
            a: root,
            b: tip,
            r: 0.025 * scale,
            m: 6,
        });
        for j in 2..8 {
            let t = j as f32 / 8.0;
            let p = root + (tip - root) * t;
            let tangent = Vec3::new(-a.sin(), 0.12, a.cos());
            for side in [-1.0, 1.0] {
                let q = p + tangent * (side * scale * 0.20 * (1.0 - t * 0.55));
                scene.objects.push(Primitive::Ellipsoid {
                    c: q,
                    r: Vec3::new(0.16 * scale, 0.035 * scale, 0.075 * scale),
                    m: 7,
                });
            }
        }
    }
}

fn add_arthropleura(scene: &mut Scene) {
    let count = 19;
    for i in 0..count {
        let f = i as f32 / (count - 1) as f32;
        let x = -2.25 + 4.15 * f;
        let z = -4.45 - 0.62 * f + 0.17 * (f * 5.8).sin();
        let y = -0.62 + 0.035 * (f * 9.0).sin();
        let c = Vec3::new(x, y, z);
        let taper = 1.0 - 0.30 * ((f - 0.48).abs() * 2.0).powf(1.7);
        let rr = Vec3::new(0.30 * taper, 0.18 * taper, 0.31 * taper);
        scene.objects.push(Primitive::Ellipsoid { c, r: rr, m: 8 });
        scene.objects.push(Primitive::Ellipsoid {
            c: c + Vec3::new(0.0, 0.115 * taper, 0.0),
            r: Vec3::new(0.24 * taper, 0.065 * taper, 0.26 * taper),
            m: 9,
        });
        if i % 2 == 0 {
            for side in [-1.0, 1.0] {
                let a = c + Vec3::new(0.0, -0.07, side * 0.18 * taper);
                let b = a + Vec3::new(-0.03, -0.19, side * 0.24);
                scene.objects.push(Primitive::Capsule {
                    a,
                    b,
                    r: 0.025,
                    m: 10,
                });
            }
        }
    }
    let head = Vec3::new(1.98, -0.59, -5.08);
    scene.objects.push(Primitive::Ellipsoid {
        c: head,
        r: Vec3::new(0.34, 0.20, 0.33),
        m: 11,
    });
    for side in [-1.0, 1.0] {
        scene.objects.push(Primitive::Sphere {
            c: head + Vec3::new(0.23, 0.09, side * 0.19),
            r: 0.035,
            m: 12,
        });
        scene.objects.push(Primitive::Capsule {
            a: head + Vec3::new(0.27, 0.02, side * 0.13),
            b: head + Vec3::new(0.58, 0.04, side * 0.30),
            r: 0.018,
            m: 10,
        });
    }
}

fn build_scene() -> Scene {
    let materials = vec![
        material(Vec3::new(0.18, 0.24, 0.17), 0.95, 0.03),
        material(Vec3::new(0.17, 0.16, 0.10), 0.9, 0.02),
        material(Vec3::new(0.12, 0.27, 0.13), 0.88, 0.01),
        material(Vec3::new(0.16, 0.39, 0.19), 0.92, 0.01),
        material(Vec3::new(0.22, 0.29, 0.17), 0.82, 0.02),
        material(Vec3::new(0.25, 0.46, 0.22), 0.88, 0.01),
        material(Vec3::new(0.18, 0.32, 0.13), 0.9, 0.01),
        material(Vec3::new(0.22, 0.50, 0.24), 0.92, 0.01),
        material(Vec3::new(0.29, 0.18, 0.10), 0.48, 0.10),
        material(Vec3::new(0.62, 0.37, 0.13), 0.38, 0.16),
        material(Vec3::new(0.18, 0.095, 0.05), 0.7, 0.03),
        material(Vec3::new(0.34, 0.20, 0.10), 0.45, 0.12),
        material(Vec3::new(0.025, 0.018, 0.01), 0.25, 0.35),
    ];
    let mut s = Scene {
        objects: vec![Primitive::Ground],
        materials,
    };
    let trees = [
        (-4.8, -9.0, 7.2, 0.34, 0.2),
        (-2.5, -12.5, 8.7, 0.40, 1.3),
        (1.1, -13.4, 9.3, 0.43, 2.1),
        (4.1, -10.2, 7.5, 0.36, 3.0),
        (-6.5, -16.5, 10.2, 0.48, 4.4),
        (6.8, -17.0, 10.8, 0.52, 5.2),
        (0.0, -20.0, 12.0, 0.58, 6.5),
    ];
    for &(x, z, h, r, q) in &trees {
        add_tree(&mut s, x, z, h, r, q);
    }
    for &(x, z, h, q) in &[
        (-3.5, -6.7, 3.5, 0.2),
        (3.1, -7.4, 4.2, 1.5),
        (5.2, -12.0, 5.2, 2.6),
        (-5.8, -12.7, 4.8, 3.7),
        (1.8, -9.2, 3.8, 4.9),
    ] {
        add_calamites(&mut s, x, z, h, q);
    }
    for &(x, z, sc, q) in &[
        (-3.6, -4.2, 1.15, 0.3),
        (3.5, -4.6, 1.35, 1.2),
        (-5.1, -6.1, 1.6, 2.0),
        (5.0, -6.5, 1.5, 2.8),
        (-1.4, -7.2, 1.1, 4.0),
        (2.0, -7.8, 1.25, 5.0),
    ] {
        add_fern(&mut s, x, z, sc, q);
    }
    // Tocones y troncos caídos añaden profundidad al pantano.
    s.objects.push(Primitive::Capsule {
        a: Vec3::new(-4.7, -0.68, -5.1),
        b: Vec3::new(-1.9, -0.74, -6.0),
        r: 0.22,
        m: 1,
    });
    s.objects.push(Primitive::Capsule {
        a: Vec3::new(2.7, -0.76, -7.0),
        b: Vec3::new(5.6, -0.69, -8.4),
        r: 0.20,
        m: 1,
    });
    add_arthropleura(&mut s);
    s
}

fn ground_material(p: Vec3) -> Material {
    let island = ((p.x * 0.43).sin() + (p.z * 0.31).cos() + (p.x * 0.17 + p.z * 0.22).sin()).abs();
    if island < 0.72 || (p.x > -2.7 && p.x < 2.7 && p.z < -6.0 && p.z > -11.0) {
        material(Vec3::new(0.045, 0.12, 0.105), 0.14, 0.58)
    } else {
        let moss = 0.5 + 0.5 * (p.x * 2.1).sin() * (p.z * 1.7).cos();
        material(
            Vec3::new(0.12 + 0.05 * moss, 0.105 + 0.07 * moss, 0.055),
            0.94,
            0.02,
        )
    }
}

fn intersect(scene: &Scene, ray: Ray, max_t: f32) -> Option<Hit> {
    let mut best = max_t;
    let mut out = None;
    for obj in &scene.objects {
        let h = match *obj {
            Primitive::Sphere { c, r, m } => sphere_hit(ray, c, r).map(|(t, n)| (t, n, m)),
            Primitive::Ellipsoid { c, r, m } => ellipsoid_hit(ray, c, r).map(|(t, n)| (t, n, m)),
            Primitive::Capsule { a, b, r, m } => capsule_hit(ray, a, b, r).map(|(t, n)| (t, n, m)),
            Primitive::CylinderY { c, r, half, m } => {
                cylinder_y_hit(ray, c, r, half).map(|(t, n)| (t, n, m))
            }
            Primitive::Ground => {
                if ray.direction.y.abs() > EPS {
                    let t = (-0.9 - ray.origin.y) / ray.direction.y;
                    if t > EPS {
                        Some((t, Vec3::new(0., 1., 0.), usize::MAX))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        if let Some((t, n, m)) = h {
            if t < best {
                best = t;
                out = Some(Hit {
                    t,
                    p: ray.at(t),
                    n,
                    m,
                });
            }
        }
    }
    out
}

fn hash(x: f32) -> f32 {
    (x.sin() * 43758.5453).fract().abs()
}

fn sky(dir: Vec3) -> Vec3 {
    let t = (dir.y * 0.5 + 0.5).clamp(0., 1.);
    let horizon = Vec3::new(0.53, 0.64, 0.48);
    let zenith = Vec3::new(0.14, 0.25, 0.24);
    let mut c = horizon.mix(zenith, t.powf(0.65));
    let sun = dir
        .dot(Vec3::new(-0.42, 0.72, 0.38).normalized())
        .max(0.)
        .powf(420.);
    c = c + Vec3::new(1.0, 0.68, 0.30) * sun * 1.8;
    c
}

fn shade(scene: &Scene, ray: Ray, hit: Hit) -> Vec3 {
    let mat = if hit.m == usize::MAX {
        ground_material(hit.p)
    } else {
        scene.materials[hit.m]
    };
    let light = Vec3::new(-0.42, 0.72, 0.38).normalized();
    let ndl = hit.n.dot(light).max(0.);
    let shadow_ray = Ray {
        origin: hit.p + hit.n * 0.003,
        direction: light,
    };
    let shadow = if intersect(scene, shadow_ray, 35.).is_some() {
        0.28
    } else {
        1.0
    };
    let ambient = Vec3::new(0.18, 0.28, 0.22);
    let warm = Vec3::new(1.0, 0.72, 0.42) * (ndl * shadow * 0.92);
    let view = -ray.direction;
    let half = (light + view).normalized();
    let spec = hit
        .n
        .dot(half)
        .max(0.)
        .powf(5. + (1. - mat.roughness) * 90.)
        * (1. - mat.roughness)
        * shadow;
    let grain = 0.90 + 0.14 * hash((hit.p.x * 47. + hit.p.y * 91. + hit.p.z * 63.).floor());
    let mut c = Vec3::new(
        mat.color.x * (ambient.x + warm.x),
        mat.color.y * (ambient.y + warm.y),
        mat.color.z * (ambient.z + warm.z),
    ) * grain
        + Vec3::new(1., 0.78, 0.48) * spec * 0.35;
    if mat.reflectivity > 0.0 {
        let rd = ray.direction - hit.n * (2. * ray.direction.dot(hit.n));
        let reflected = sky(rd);
        c = c.mix(
            reflected,
            mat.reflectivity * (0.55 + 0.45 * (1. - view.dot(hit.n).abs())),
        );
    }
    let fog = (1.0 - (-hit.t * 0.038).exp()).clamp(0., 0.68);
    c.mix(Vec3::new(0.31, 0.43, 0.35), fog)
}

fn main() -> std::io::Result<()> {
    let scene = build_scene();
    let camera = Vec3::new(0.0, 0.85, 3.65);
    let target = Vec3::new(0.0, 0.55, -7.1);
    let forward = (target - camera).normalized();
    let right = forward.cross(Vec3::new(0., 1., 0.)).normalized();
    let up = right.cross(forward);
    let aspect = WIDTH as f32 / HEIGHT as f32;
    let fov = (52.0f32.to_radians() * 0.5).tan();
    let mut pixels = vec![0u8; WIDTH * HEIGHT * 3];
    for y in 0..HEIGHT {
        if y % 80 == 0 {
            eprintln!("render {}%", y * 100 / HEIGHT);
        }
        for x in 0..WIDTH {
            let jitter = hash((x + y * WIDTH) as f32) * 0.38;
            let u = ((x as f32 + 0.5 + jitter) / WIDTH as f32 * 2. - 1.) * aspect * fov;
            let v = (1. - (y as f32 + 0.5 - jitter) / HEIGHT as f32 * 2.) * fov;
            let ray = Ray {
                origin: camera,
                direction: (forward + right * u + up * v).normalized(),
            };
            let color = if let Some(hit) = intersect(&scene, ray, 80.) {
                shade(&scene, ray, hit)
            } else {
                sky(ray.direction)
            };
            let c = Vec3::new(
                color.x.powf(1. / 2.2),
                color.y.powf(1. / 2.2),
                color.z.powf(1. / 2.2),
            )
            .clamp01();
            let i = (y * WIDTH + x) * 3;
            pixels[i] = (c.x * 255.) as u8;
            pixels[i + 1] = (c.y * 255.) as u8;
            pixels[i + 2] = (c.z * 255.) as u8;
        }
    }
    let file = File::create("carboniferous_swamp.ppm")?;
    let mut out = BufWriter::new(file);
    write!(out, "P6\n{} {}\n255\n", WIDTH, HEIGHT)?;
    out.write_all(&pixels)?;
    eprintln!("saved carboniferous_swamp.ppm");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn central_sphere_ray_hits() {
        let r = Ray {
            origin: Vec3::new(0., 0., 0.),
            direction: Vec3::new(0., 0., -1.),
        };
        let h = sphere_hit(r, Vec3::new(0., 0., -3.), 1.).unwrap();
        assert!((h.0 - 2.0).abs() < 0.0001);
    }
    #[test]
    fn scene_has_dense_vegetation() {
        let s = build_scene();
        assert!(s.objects.len() > 250);
    }
}
