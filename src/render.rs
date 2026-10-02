use crate::disaster::Disaster;
use crate::material::{tint, MaterialId};
use crate::{
    math::*,
    scene::{Hit, Scene},
};
#[derive(Clone, Copy)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: V,
}
impl Camera {
    pub fn overview() -> Self {
        Self {
            yaw: 0.28,
            pitch: 0.48,
            distance: 208.,
            target: V::new(0., 5., 17.),
        }
    }
    pub fn preset(index: usize) -> Self {
        match index {
            2 => Self {
                yaw: 0.4,
                pitch: 0.55,
                distance: 36.,
                target: V::new(-12., 1.5, 13.),
            },
            3 => Self {
                yaw: -0.2,
                pitch: 0.25,
                distance: 28.,
                target: V::new(17., 2., 10.),
            },
            4 => Self {
                yaw: 0.3,
                pitch: 0.4,
                distance: 39.,
                target: V::new(-10., 2., -12.),
            },
            5 => Self {
                yaw: 0.3,
                pitch: 0.40,
                distance: 30.,
                target: V::new(22., 1., -14.),
            },
            6 => Self {
                yaw: 0.15,
                pitch: 0.20,
                distance: 10.,
                target: V::new(1., 3., 7.),
            },
            7 => Self {
                yaw: 0.15,
                pitch: 0.38,
                distance: 3.4,
                target: V::new(-23., crate::scene::ground(-23., -14.) + 0.15, -14.),
            },
            8 => Self {
                yaw: 0.40,
                pitch: 0.48,
                distance: 65.,
                target: V::new(-33., 14., -35.),
            },
            9 => Self {
                yaw: 0.10,
                pitch: 0.66,
                distance: 106.,
                target: V::new(0., -1., 70.),
            },
            10 => Self {
                yaw: 0.20,
                pitch: 0.66,
                distance: 31.,
                target: V::new(-20., -2., 75.),
            },
            11 => Self {
                yaw: 0.10,
                pitch: 0.60,
                distance: 31.,
                target: V::new(22., -2., 82.),
            },
            12 => Self {
                yaw: 0.12,
                pitch: 0.65,
                distance: 16.,
                target: V::new(-7., -1., 60.),
            },
            13 => Self {
                yaw: 0.45,
                pitch: 0.28,
                distance: 13.,
                target: V::new(-32., 3.2, 51.5),
            },
            14 => Self {
                yaw: 0.04,
                pitch: 0.10,
                distance: 58.,
                target: V::new(-29., 1.2, 55.),
            },
            15 => Self {
                yaw: 0.45,
                pitch: 0.72,
                distance: 52.,
                target: crate::disaster::IMPACT,
            },
            16 => Self {
                yaw: 0.30,
                pitch: 0.32,
                distance: 100.,
                target: V::new(0., 0., 67.),
            },
            _ => Self::overview(),
        }
    }
    pub fn eye(self) -> V {
        self.target
            + V::new(
                self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.cos() * self.pitch.cos(),
            ) * self.distance
    }
    pub fn ray(self, x: f32, y: f32, w: usize, h: usize) -> Ray {
        CameraFrame::new(self, w, h).ray(x, y)
    }
    pub fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance * (-amount * 0.08).exp()).clamp(1.2, 240.);
    }
    pub fn focus(self, bounds: Bounds) -> Self {
        let extent = (bounds.hi - bounds.lo).length();
        let right = (self.target - self.eye())
            .norm()
            .cross(V::new(0., 1., 0.))
            .norm();
        Self {
            target: (bounds.lo + bounds.hi) * 0.5 + right * (extent * 0.34),
            distance: (extent * 2.).clamp(2.8, 70.),
            pitch: self.pitch.clamp(0.14, 0.7),
            ..self
        }
    }
    pub fn interpolate(self, destination: Self, t: f32) -> Self {
        let t = t.clamp(0., 1.);
        let t = t * t * (3. - 2. * t);
        Self {
            target: self.target.mix(destination.target, t),
            yaw: self.yaw + (destination.yaw - self.yaw) * t,
            pitch: self.pitch + (destination.pitch - self.pitch) * t,
            distance: self.distance + (destination.distance - self.distance) * t,
        }
    }
    pub fn orbit(&mut self, horizontal: f32, vertical: f32) {
        self.yaw += horizontal;
        self.pitch = (self.pitch + vertical).clamp(0.04, 1.35);
    }
}
// Camera trigonometry and basis vectors are constant for the entire image.
#[derive(Clone, Copy)]
struct CameraFrame {
    origin: V,
    corner: V,
    dx: V,
    dy: V,
}
impl CameraFrame {
    fn new(cam: Camera, w: usize, h: usize) -> Self {
        let origin = cam.eye();
        let forward = (cam.target - origin).norm();
        let right = forward.cross(V::new(0., 1., 0.)).norm();
        let up = right.cross(forward);
        let half_width = 0.4 * w as f32 / h as f32;
        Self {
            origin,
            corner: forward - right * half_width + up * 0.4,
            dx: right * (2. * half_width / w as f32),
            dy: up * (-0.8 / h as f32),
        }
    }
    fn ray(self, x: f32, y: f32) -> Ray {
        Ray {
            o: self.origin,
            d: (self.corner + self.dx * x + self.dy * y).norm(),
        }
    }
}

fn hash(p: V) -> f32 {
    let x = (p.x.floor() as i32).wrapping_mul(73856093)
        ^ (p.y.floor() as i32).wrapping_mul(19349663)
        ^ (p.z.floor() as i32).wrapping_mul(83492791);
    let mut h = x as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    (h & 65535) as f32 / 65535.
}
pub(crate) fn noise(p: V) -> f32 {
    let b = V::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = p - b;
    let t = V::new(
        f.x * f.x * (3. - 2. * f.x),
        f.y * f.y * (3. - 2. * f.y),
        f.z * f.z * (3. - 2. * f.z),
    );
    let mut out = 0.;
    for x in 0..=1 {
        for y in 0..=1 {
            for z in 0..=1 {
                let weight = if x == 0 { 1. - t.x } else { t.x }
                    * if y == 0 { 1. - t.y } else { t.y }
                    * if z == 0 { 1. - t.z } else { t.z };
                out += hash(b + V::new(x as f32, y as f32, z as f32)) * weight;
            }
        }
    }
    out
}
fn sky(ray: Ray, _forest: bool) -> V {
    let elevation = ray.d.y.max(0.);
    let mut c = V::new(0.49, 0.73, 0.92).mix(V::new(0.10, 0.36, 0.75), elevation.sqrt());
    let cloud = (noise(ray.d * 7. + V::new(4., 0., 2.)) * 0.7 + noise(ray.d * 19.) * 0.3 - 0.58)
        .max(0.)
        * 2.;
    c = c.mix(V::new(0.92, 0.95, 0.97), cloud.min(0.6));
    c
}
pub fn reflect(direction: V, normal: V) -> V {
    direction - normal * (2. * direction.dot(normal))
}
// normal faces the incident ray; eta = incident IOR / transmitted IOR.
pub fn refract(direction: V, normal: V, eta: f32) -> Option<V> {
    let cosine = (-direction.dot(normal)).clamp(0., 1.);
    let k = 1. - eta * eta * (1. - cosine * cosine);
    if k < 0. {
        None
    } else {
        Some((direction * eta + normal * (eta * cosine - k.sqrt())).norm())
    }
}
fn fresnel(f0: f32, cosine: f32) -> f32 {
    f0 + (1. - f0) * (1. - cosine.clamp(0., 1.)).powi(5)
}
fn sky_color(scene: &Scene, ray: Ray, disaster: Disaster) -> V {
    disaster.atmosphere(sky(ray, scene.forest), ray)
}
fn ocean_hit(ray: Ray, limit: f32, disaster: Disaster) -> Option<(f32, V, V)> {
    if !disaster.tsunami() {
        if ray.d.y.abs() < 1e-6 {
            return None;
        }
        let t = (0.45 - ray.o.y) / ray.d.y;
        if t < 0.002 || t >= limit {
            return None;
        }
        let p = ray.o + ray.d * t;
        if p.x.abs() > 60.5 || p.z < 55. + (p.x * 0.1).sin() * 1.4 || p.z > 94.5 {
            None
        } else {
            Some((t, p, V::new(0., 1., 0.)))
        }
    } else {
        // Intersect the moving height field inside a tight volume, then refine the crossing.
        let volume = Bounds {
            lo: V::new(-60.5, 0.44, 43.),
            hi: V::new(60.5, 11., 94.5),
        };
        let (near, far, _, _) = volume.interval(ray, limit)?;
        let start = near.max(0.008);
        let end = far.min(limit);
        if end <= start {
            return None;
        }
        let steps = ((end - start) / 0.8).ceil().clamp(1., 220.) as usize;
        let delta = (end - start) / steps as f32;
        let value = |t: f32| {
            let p = ray.o + ray.d * t;
            p.y - disaster.water_height(p.x, p.z)
        };
        let mut previous_t = start;
        let mut previous = value(start);
        for i in 1..=steps {
            let t = start + delta * i as f32;
            let current = value(t);
            if (current >= 0.) != (previous >= 0.) {
                let (mut lo, mut hi) = (previous_t, t);
                for _ in 0..10 {
                    let mid = (lo + hi) * 0.5;
                    if (value(mid) >= 0.) == (previous >= 0.) {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let t = (lo + hi) * 0.5;
                let p = ray.o + ray.d * t;
                if p.z >= 55. + (p.x * 0.1).sin() * 1.4 || disaster.water_height(p.x, p.z) > 0.5 {
                    return Some((t, p, disaster.water_normal(p)));
                }
            }
            previous_t = t;
            previous = current;
        }
        None
    }
}
// Selection follows the same refracted path through the ocean as the displayed image.
pub fn pick(scene: &Scene, mut ray: Ray, disaster: Disaster) -> Option<Hit> {
    let hit = scene.hit(ray, 500.);
    if let Some((_, p, outward)) = ocean_hit(ray, hit.map_or(500., |h| h.t), disaster) {
        let entering = ray.d.dot(outward) < 0.;
        let n = if entering { outward } else { -outward };
        let ior = MaterialId::Water.get().ior;
        ray = Ray {
            o: p,
            d: refract(ray.d, n, if entering { 1. / ior } else { ior })?,
        };
        ray.o = ray.o + ray.d * 0.006;
        return scene.hit(ray, 500.);
    }
    hit
}
fn shade(scene: &Scene, ray: Ray, hit: Hit, time: f32, shadows: bool, disaster: Disaster) -> V {
    let cube = &scene.cubes[hit.index];
    let material = cube.material.get();
    let p = ray.o + ray.d * hit.t;
    let relief = V::new(
        (p.x * 17. + p.z * 4.).sin(),
        (p.y * 23. + p.x * 3.).sin(),
        (p.z * 19. + p.y * 5.).sin(),
    );
    let n = (hit.n + (relief - hit.n * relief.dot(hit.n)) * material.roughness).norm();
    let light = V::new(-0.5, 0.85, 0.55).norm();
    let color = tint(cube.color, material.sample(p, time));
    let diffuse = n.dot(light).max(0.);
    let shadow = if shadows && diffuse > 0. {
        0.3 + 0.7
            * scene.light_visibility(
                Ray {
                    o: p + hit.n * 0.004,
                    d: light,
                },
                35.,
            )
    } else {
        1.
    };
    let highlight = n
        .dot((light - ray.d).norm())
        .max(0.)
        .powi(material.shininess)
        * material.specular
        * shadow;
    let mut c = color * (0.40 + diffuse * shadow * 0.82) + V::new(1., 0.98, 0.94) * highlight;
    if scene.forest {
        let fog = (1. - (-hit.t * 0.0015).exp()).min(0.40);
        c = c.mix(V::new(0.49, 0.67, 0.78), fog);
    }
    disaster.surface(
        c,
        p,
        cube.material,
        matches!(
            cube.tag,
            crate::scene::Tag::Mammal | crate::scene::Tag::CrownBird
        ) || cube.tag.marine_survivor(),
        cube.material == MaterialId::Wood
            || (cube.material == MaterialId::Organic
                && matches!(
                    cube.tag,
                    crate::scene::Tag::Conifer
                        | crate::scene::Tag::Broadleaf
                        | crate::scene::Tag::Fern
                )),
        time,
    )
}
fn optical(
    scene: &Scene,
    ray: Ray,
    p: V,
    outward: V,
    id: MaterialId,
    local: V,
    time: f32,
    shadows: bool,
    disaster: Disaster,
    depth: u8,
) -> V {
    let material = id.get();
    if depth == 0 {
        return local;
    }
    let entering = ray.d.dot(outward) < 0.;
    let n = if entering { outward } else { -outward };
    let eta = if entering {
        1. / material.ior
    } else {
        material.ior
    };
    let transmitted = refract(ray.d, n, eta);
    let reflection_weight = if transmitted.is_none() {
        1.
    } else {
        fresnel(material.reflectivity, -ray.d.dot(n))
    };
    // Geometric normals keep picking and Snell refraction consistent; ripples perturb reflection only.
    let reflection_normal = if id == MaterialId::Water {
        (n + V::new(
            (p.x * 1.8 + time * 0.6).sin(),
            0.,
            (p.z * 2.7 - time * 0.4).cos(),
        ) * material.roughness)
            .norm()
    } else {
        n
    };
    let reflection_direction = reflect(ray.d, reflection_normal).norm();
    let reflected = trace_color(
        scene,
        Ray {
            o: p + reflection_direction * 0.006,
            d: reflection_direction,
        },
        time,
        shadows,
        disaster,
        depth - 1,
    );
    let transmission_weight = (1. - reflection_weight) * material.transparency;
    let mut out = local * ((1. - reflection_weight) * (1. - material.transparency))
        + reflected * reflection_weight;
    if let Some(direction) = transmitted {
        if transmission_weight > 0. {
            let outgoing = Ray {
                o: p + direction * 0.006,
                d: direction,
            };
            let behind = trace_color(scene, outgoing, time, shadows, disaster, depth - 1);
            let textured = material.sample(p, time);
            let transmitted_color = if id == MaterialId::Water {
                let distance = scene.hit(outgoing, 60.).map_or(30., |h| h.t);
                behind.mix(
                    textured.mix(V::new(0.15, 0.23, 0.25), disaster.ash()),
                    (0.10 + distance * 0.025).min(0.55),
                )
            } else {
                tint(behind, V::new(1., 1., 1.).mix(textured, 0.20))
            };
            out = out + transmitted_color * transmission_weight;
        }
    }
    let light = V::new(-0.5, 0.85, 0.55).norm();
    out + V::new(1., 0.99, 0.96)
        * reflection_normal
            .dot((light - ray.d).norm())
            .max(0.)
            .powi(material.shininess)
        * material.specular
        * 0.3
}
fn trace_color(
    scene: &Scene,
    ray: Ray,
    time: f32,
    shadows: bool,
    disaster: Disaster,
    depth: u8,
) -> V {
    let hit = scene.hit(ray, 500.);
    let distance = hit.map_or(500., |h| h.t);
    let ocean = ocean_hit(ray, distance, disaster);
    if let Some(meteor) = disaster.meteor(ray, ocean.map_or(distance, |h| h.0)) {
        return meteor;
    }
    if let Some((_, p, normal)) = ocean {
        let local = MaterialId::Water.get().sample(p, time);
        let water = optical(
            scene,
            ray,
            p,
            normal,
            MaterialId::Water,
            local,
            time,
            shadows,
            disaster,
            depth,
        );
        let foam = if disaster.tsunami() {
            ((p.y - 1.5) / 6.).clamp(0., 0.88) * (0.7 + noise(p * 2.) * 0.3)
        } else {
            0.
        };
        return water.mix(V::new(0.92, 0.95, 0.93), foam);
    }
    if let Some(hit) = hit {
        let local = shade(scene, ray, hit, time, shadows, disaster);
        let id = scene.cubes[hit.index].material;
        if id.get().reflectivity > 0. || id.get().transparency > 0. {
            optical(
                scene,
                ray,
                ray.o + ray.d * hit.t,
                hit.n,
                id,
                local,
                time,
                shadows,
                disaster,
                depth,
            )
        } else {
            local
        }
    } else {
        sky_color(scene, ray, disaster)
    }
}
pub fn render(
    scene: &Scene,
    cam: Camera,
    w: usize,
    h: usize,
    time: f32,
    full_quality: bool,
) -> Vec<u8> {
    render_cancellable(scene, cam, w, h, time, full_quality, None).unwrap()
}

pub fn render_cancellable(
    scene: &Scene,
    cam: Camera,
    w: usize,
    h: usize,
    time: f32,
    full_quality: bool,
    cancel: Option<(&std::sync::atomic::AtomicU64, u64)>,
) -> Option<Vec<u8>> {
    render_effects(
        scene,
        cam,
        w,
        h,
        time,
        full_quality,
        cancel,
        Disaster::default(),
    )
}
pub fn render_effects(
    scene: &Scene,
    cam: Camera,
    w: usize,
    h: usize,
    time: f32,
    full_quality: bool,
    cancel: Option<(&std::sync::atomic::AtomicU64, u64)>,
    disaster: Disaster,
) -> Option<Vec<u8>> {
    use std::sync::atomic::Ordering;
    let cancelled = || {
        cancel.is_some_and(|(generation, expected)| generation.load(Ordering::Relaxed) != expected)
    };
    let frame = CameraFrame::new(cam, w, h);
    let mut pixels = vec![0u8; w * h * 4];
    let workers = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(8);
    let rows = h.div_ceil(workers);
    std::thread::scope(|scope| {
        for (chunk_index, chunk) in pixels.chunks_mut(rows * w * 4).enumerate() {
            scope.spawn(move || {
                let y0 = chunk_index * rows;
                for (i, pixel) in chunk.chunks_exact_mut(4).enumerate() {
                    if i % (w * 4) == 0 && cancelled() {
                        return;
                    }
                    let x = i % w;
                    let y = y0 + i / w;
                    let samples = if full_quality { 2 } else { 1 };
                    let mut c = V::default();
                    for sample in 0..samples {
                        let (dx, dy) = if samples == 1 {
                            (0.5, 0.5)
                        } else if sample == 0 {
                            (0.25, 0.25)
                        } else {
                            (0.75, 0.75)
                        };
                        let ray = frame.ray(x as f32 + dx, y as f32 + dy);
                        let base = trace_color(
                            scene,
                            ray,
                            time,
                            true, // Keep shadow rays enabled in moving previews too.
                            disaster,
                            if full_quality { 4 } else { 2 },
                        );
                        c = c + base.mix(V::new(1., 0.92, 0.72), disaster.flash());
                    }
                    c = c / samples as f32;
                    let nx = x as f32 / w as f32 * 2. - 1.;
                    let ny = y as f32 / h as f32 * 2. - 1.;
                    c = c * (1. - 0.035 * (nx * nx + ny * ny));
                    pixel[0] = (c.x.clamp(0., 1.).sqrt() * 255.) as u8;
                    pixel[1] = (c.y.clamp(0., 1.).sqrt() * 255.) as u8;
                    pixel[2] = (c.z.clamp(0., 1.).sqrt() * 255.) as u8;
                    pixel[3] = 255;
                }
            });
        }
    });
    if cancelled() {
        None
    } else {
        Some(pixels)
    }
}
pub fn ppm(path: &str, pixels: &[u8], w: usize, h: usize) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(f, "P6\n{w} {h}\n255\n")?;
    for p in pixels.chunks_exact(4) {
        f.write_all(&p[..3])?;
    }
    Ok(())
}
// Bilinear preview: no nearest-neighbour blocks while orbiting.
pub fn upscale(src: &[u8], sw: usize, sh: usize, w: usize, h: usize) -> Vec<u8> {
    let mut dst = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 * (sw - 1) as f32 / (w - 1) as f32;
            let v = y as f32 * (sh - 1) as f32 / (h - 1) as f32;
            let x0 = u as usize;
            let y0 = v as usize;
            let x1 = (x0 + 1).min(sw - 1);
            let y1 = (y0 + 1).min(sh - 1);
            let fx = u - x0 as f32;
            let fy = v - y0 as f32;
            for c in 0..4 {
                let a = src[(y0 * sw + x0) * 4 + c] as f32 * (1. - fx)
                    + src[(y0 * sw + x1) * 4 + c] as f32 * fx;
                let b = src[(y1 * sw + x0) * 4 + c] as f32 * (1. - fx)
                    + src[(y1 * sw + x1) * 4 + c] as f32 * fx;
                dst[(y * w + x) * 4 + c] = (a * (1. - fy) + b * fy) as u8;
            }
        }
    }
    dst
}
#[cfg(test)]
mod texture_tests {
    use super::*;
    #[test]
    fn snell_reflection_and_total_internal_reflection() {
        let incoming = V::new(0.5, -(0.75f32).sqrt(), 0.);
        let normal = V::new(0., 1., 0.);
        let reflected = reflect(incoming, normal);
        assert!((reflected.x - 0.5).abs() < 1e-6 && reflected.y > 0.);
        let transmitted = refract(incoming, normal, 1. / 1.52).unwrap();
        assert!((transmitted.x - 0.5 / 1.52).abs() < 1e-6);
        assert!((transmitted.length() - 1.).abs() < 1e-6);
        let emerged = refract(transmitted, normal, 1.52).unwrap();
        assert!((emerged - incoming).length() < 1e-6);
        assert!(refract(V::new(0.9, -(0.19f32).sqrt(), 0.), normal, 1.52).is_none());
        assert!(fresnel(0.045, 0.1) > fresnel(0.045, 1.));
    }
    #[test]
    fn ocean_picking_matches_refracted_ray_and_reflection_hits_geometry() {
        let scene = crate::scene::biome();
        let ray = Camera::preset(10).ray(550., 378., 1100, 756);
        let (_, p, _) = ocean_hit(ray, 500., Disaster::default()).unwrap();
        let d = refract(ray.d, V::new(0., 1., 0.), 1. / MaterialId::Water.get().ior).unwrap();
        let expected = scene
            .hit(
                Ray {
                    o: p + d * 0.006,
                    d,
                },
                500.,
            )
            .unwrap();
        assert_eq!(
            pick(&scene, ray, Disaster::default()).unwrap().index,
            expected.index
        );
        // The reflected path can hit the coast rather than only return a sky color.
        let p = V::new(-32., 0.45, 65.);
        let incident = V::new(0., -0.15, -1.).norm();
        let direction = reflect(incident, V::new(0., 1., 0.));
        assert!(scene
            .hit(
                Ray {
                    o: p + direction * 0.006,
                    d: direction
                },
                100.
            )
            .is_some());
    }
    #[test]
    fn tsunami_surface_is_raised_geometry() {
        let disaster = Disaster::at(12.);
        let ray = Ray {
            o: V::new(0., 20., 82.2),
            d: V::new(0., -1., 0.),
        };
        let (_, p, n) = ocean_hit(ray, 100., disaster).unwrap();
        assert!(p.y > 7.);
        assert!((p.y - disaster.water_height(p.x, p.z)).abs() < 0.005);
        assert!((n.length() - 1.).abs() < 0.001);
        assert!((ocean_hit(ray, 100., Disaster::default()).unwrap().1.y - 0.45).abs() < 0.001);
    }
    #[test]
    fn orbit_preserves_target_and_distance() {
        let mut cam = Camera::preset(3);
        let old = cam;
        cam.orbit(0.2, 0.1);
        assert!((cam.yaw - old.yaw - 0.2).abs() < 1e-6);
        assert!((cam.target - old.target).length() < 1e-6);
        assert_eq!(cam.distance, old.distance);
        assert!((cam.eye() - old.eye()).length() > 1.);
        cam.orbit(0., 100.);
        assert_eq!(cam.pitch, 1.35);
        cam.orbit(0., -100.);
        assert_eq!(cam.pitch, 0.04);
    }
    #[test]
    fn focus_zoom_and_restore_camera() {
        let original = Camera::overview();
        let bounds = Bounds {
            lo: V::new(13., 2., 8.),
            hi: V::new(22., 7., 12.),
        };
        let mut focused = original.focus(bounds);
        assert!(focused.distance < original.distance);
        let before = focused.distance;
        focused.zoom(2.);
        assert!(focused.distance < before);
        focused.zoom(-2.);
        assert!((focused.distance - before).abs() < 0.0001);
        let restored = focused.interpolate(original, 1.);
        assert!((restored.target - original.target).length() < 0.0001);
        assert!((restored.distance - original.distance).abs() < 0.0001);
    }
    #[test]
    fn prepared_camera_matches_perspective() {
        for view in 1..=8 {
            let cam = Camera::preset(view);
            let eye = cam.eye();
            let forward = (cam.target - eye).norm();
            let right = forward.cross(V::new(0., 1., 0.)).norm();
            let up = right.cross(forward);
            for (x, y) in [(0., 0.), (550., 378.), (1099., 755.)] {
                let expected = (forward
                    + right * ((x / 1100. * 2. - 1.) * (1100. / 756.) * 0.4)
                    + up * ((1. - y / 756. * 2.) * 0.4))
                    .norm();
                assert!((cam.ray(x, y, 1100, 756).d - expected).length() < 1e-6);
            }
        }
    }
    #[test]
    fn obsolete_render_cancels_and_current_render_completes() {
        let scene = crate::scene::biome();
        let generation = std::sync::atomic::AtomicU64::new(2);
        assert!(render_cancellable(
            &scene,
            Camera::overview(),
            32,
            24,
            0.,
            true,
            Some((&generation, 1))
        )
        .is_none());
        let frame = render_cancellable(
            &scene,
            Camera::overview(),
            32,
            24,
            0.,
            true,
            Some((&generation, 2)),
        )
        .unwrap();
        assert_eq!(frame.len(), 32 * 24 * 4);
        assert!(frame.chunks_exact(4).all(|p| p[3] == 255));
    }
    #[test]
    fn interpolated_noise_is_continuous() {
        for i in -4..4 {
            let a = noise(V::new(i as f32 - 0.0001, 0.3, 0.6));
            let b = noise(V::new(i as f32 + 0.0001, 0.3, 0.6));
            assert!((a - b).abs() < 0.001);
        }
    }
}
