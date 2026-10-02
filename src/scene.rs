use crate::material::MaterialId;
use crate::math::*;
mod marine;
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tag {
    None,
    Conifer,
    Broadleaf,
    Fern,
    Tyrannosaurus,
    Triceratops,
    Edmontosaurus,
    Ankylosaurus,
    Enantiornithine,
    Scapherpeton,
    Mammal,
    CrownBird,
    Mosasaur,
    Plesiosaur,
    Ammonite,
    HorseshoeCrab,
    Bivalve,
    Nautiloid,
    Mineral,
}
impl Tag {
    pub fn marine_survivor(self) -> bool {
        matches!(self, Self::HorseshoeCrab | Self::Bivalve | Self::Nautiloid)
    }
}
#[derive(Clone)]
pub struct Cube {
    pub bounds: Bounds,
    pub color: V,
    pub material: MaterialId,
    pub tag: Tag,
    pub entity: usize,
}
pub struct Scene {
    pub cubes: Vec<Cube>,
    nodes: Vec<Node>,
    pub forest: bool,
    pub entities: Vec<Bounds>,
}
struct Node {
    bounds: Bounds,
    start: usize,
    end: usize,
    children: Option<(usize, usize)>,
}
#[derive(Clone, Copy)]
pub struct Hit {
    pub t: f32,
    pub n: V,
    pub index: usize,
}
impl Scene {
    fn new(forest: bool) -> Self {
        Self {
            cubes: vec![],
            nodes: vec![],
            forest,
            entities: Vec::new(),
        }
    }
    fn cube(&mut self, p: V, size: V, color: V, material: MaterialId, tag: Tag) {
        self.cubes.push(Cube {
            bounds: Bounds {
                lo: p - size * 0.5,
                hi: p + size * 0.5,
            },
            color,
            material,
            tag,
            entity: 0,
        });
    }
    fn register_animal(&mut self, start: usize) {
        let mut bounds = Bounds {
            lo: V::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            hi: V::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };
        let entity = self.entities.len() + 1;
        for cube in &mut self.cubes[start..] {
            bounds.lo = bounds.lo.min(cube.bounds.lo);
            bounds.hi = bounds.hi.max(cube.bounds.hi);
            cube.entity = entity;
        }
        self.entities.push(bounds);
    }
    pub fn animal_bounds(&self, index: usize) -> Option<Bounds> {
        if self.cubes.get(index)?.material == MaterialId::Crystal {
            return Some(self.cubes[index].bounds);
        }
        self.cubes
            .get(index)?
            .entity
            .checked_sub(1)
            .and_then(|id| self.entities.get(id))
            .copied()
    }
    fn build(&mut self, start: usize, end: usize) -> usize {
        let mut bounds = Bounds {
            lo: V::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            hi: V::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };
        for b in &self.cubes[start..end] {
            bounds.lo = bounds.lo.min(b.bounds.lo);
            bounds.hi = bounds.hi.max(b.bounds.hi);
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            start,
            end,
            children: None,
        });
        if end - start > 6 {
            let size = bounds.hi - bounds.lo;
            let axis = if size.x > size.y && size.x > size.z {
                0
            } else if size.y > size.z {
                1
            } else {
                2
            };
            self.cubes[start..end].sort_unstable_by(|a, b| {
                (a.bounds.lo.at(axis) + a.bounds.hi.at(axis))
                    .total_cmp(&(b.bounds.lo.at(axis) + b.bounds.hi.at(axis)))
            });
            let mid = (start + end) / 2;
            let l = self.build(start, mid);
            let r = self.build(mid, end);
            self.nodes[id].children = Some((l, r));
        }
        id
    }
    fn finish(mut self) -> Self {
        self.build(0, self.cubes.len());
        self
    }
    pub fn hit(&self, ray: Ray, limit: f32) -> Option<Hit> {
        self.trace(ray, limit, false)
    }
    // Shadow rays only need an obstruction, not the closest surface.
    #[cfg(test)]
    pub fn occluded(&self, ray: Ray, limit: f32) -> bool {
        self.trace(ray, limit, true).is_some()
    }
    pub fn light_visibility(&self, mut ray: Ray, mut limit: f32) -> f32 {
        // Any opaque obstruction suffices; only transparent paths need closest-hit traversal.
        let Some(first) = self.trace(ray, limit, true) else {
            return 1.;
        };
        if self.cubes[first.index].material.get().transparency == 0. {
            return 0.;
        }
        let mut visibility = 1.;
        for _ in 0..8 {
            let Some(hit) = self.hit(ray, limit) else {
                return visibility;
            };
            visibility *= self.cubes[hit.index].material.get().transparency;
            if visibility < 0.01 {
                return 0.;
            }
            let step = hit.t + 0.006;
            ray.o = ray.o + ray.d * step;
            limit -= step;
            if limit <= 0. {
                return visibility;
            }
        }
        visibility
    }
    fn trace(&self, ray: Ray, limit: f32, any: bool) -> Option<Hit> {
        let query = BoxQuery::new(ray);
        let mut best = limit;
        let mut result = None;
        let mut stack = [(0usize, 0f32); 64];
        stack[0].1 = query.entry(self.nodes[0].bounds, best)?;
        let mut len = 1;
        while len > 0 {
            len -= 1;
            let (id, entry) = stack[len];
            if entry > best {
                continue;
            }
            let node = &self.nodes[id];
            if let Some((l, r)) = node.children {
                let a = query.entry(self.nodes[l].bounds, best);
                let b = query.entry(self.nodes[r].bounds, best);
                match (a, b) {
                    (Some(a), Some(b)) => {
                        let (near, far) = if a < b {
                            ((l, a), (r, b))
                        } else {
                            ((r, b), (l, a))
                        };
                        stack[len] = far;
                        stack[len + 1] = near;
                        len += 2;
                    }
                    (Some(a), None) => {
                        stack[len] = (l, a);
                        len += 1;
                    }
                    (None, Some(b)) => {
                        stack[len] = (r, b);
                        len += 1;
                    }
                    _ => {}
                }
            } else {
                for i in node.start..node.end {
                    if query.entry(self.cubes[i].bounds, best).is_none() {
                        continue;
                    }
                    if let Some((t, n)) = self.cubes[i].bounds.hit(ray, best) {
                        if t < best {
                            let hit = Hit { t, n, index: i };
                            if any {
                                return Some(hit);
                            }
                            best = t;
                            result = Some(hit);
                        }
                    }
                }
            }
        }
        result
    }
}
struct BoxQuery {
    ray: Ray,
    inverse: V,
}
impl BoxQuery {
    fn new(ray: Ray) -> Self {
        Self {
            ray,
            inverse: V::new(1. / ray.d.x, 1. / ray.d.y, 1. / ray.d.z),
        }
    }
    fn entry(&self, bounds: Bounds, limit: f32) -> Option<f32> {
        let mut near = f32::NEG_INFINITY;
        let mut far = limit;
        for axis in 0..3 {
            let origin = self.ray.o.at(axis);
            let lo = bounds.lo.at(axis);
            let hi = bounds.hi.at(axis);
            if self.ray.d.at(axis).abs() < 1e-8 {
                if origin < lo || origin > hi {
                    return None;
                }
            } else {
                let a = (lo - origin) * self.inverse.at(axis);
                let b = (hi - origin) * self.inverse.at(axis);
                near = near.max(a.min(b));
                far = far.min(a.max(b));
            }
        }
        if far >= near && far >= 0.001 {
            Some(near)
        } else {
            None
        }
    }
}
struct Rng(u32);
impl Rng {
    fn f(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0 as f32 / u32::MAX as f32
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}
pub fn river(x: f32) -> f32 {
    (x * 0.115).sin() * 5.5 - 1.5
}
pub fn ground(x: f32, z: f32) -> f32 {
    let clearings = [
        (-13.0, 14.0),
        (-22.0, 16.0),
        (17.0, 10.0),
        (-9.0, -10.0),
        (-16.0, -12.0),
        (-5.0, -15.0),
        (22.0, -14.0),
        (21.0, -21.0),
    ];
    if clearings
        .iter()
        .any(|(cx, cz)| (x - cx).powi(2) + (z - cz).powi(2) < 42.0)
    {
        return 2.0;
    }

    let bank = ((z - river(x)).abs() - 3.2).max(0.);
    let hill = ((x * 0.095).sin() * (z * 0.08).cos() + 1.) * 0.65;
    let peak1 = 29.0 * (-((x + 35.0) / 16.0).powi(2) - ((z + 38.0) / 12.0).powi(2)).exp();
    let peak2 = 24.0 * (-((x - 32.0) / 19.0).powi(2) - ((z + 39.0) / 13.0).powi(2)).exp();
    let ridge = 13.0 * (-((x + 54.0) / 9.0).powi(2) - ((z + 5.0) / 23.0).powi(2)).exp();
    let relief = (peak1 + peak2 + ridge) * (0.9 + 0.10 * (x * 0.38 + z * 0.24).sin());
    1. + (bank * 0.04 + hill + relief).floor()
}
fn tree(s: &mut Scene, x: f32, z: f32, h: f32, broad: bool, r: &mut Rng) {
    let y = ground(x, z);
    let wood = V::new(0.30, 0.23, 0.14);
    let green = if broad {
        V::new(0.29, 0.39, 0.18)
    } else {
        V::new(0.20, 0.30, 0.17)
    };
    let tag = if broad { Tag::Broadleaf } else { Tag::Conifer };
    for k in 0..(h / 0.65) as usize {
        s.cube(
            V::new(x, y + k as f32 * 0.65 + 0.32, z),
            V::new(0.64, 0.66, 0.64),
            wood * r.range(0.9, 1.15),
            MaterialId::Wood,
            tag,
        );
    }
    if broad {
        for branch in 0..4 {
            let a = branch as f32 * 1.57;
            for j in 1..5 {
                s.cube(
                    V::new(
                        x + a.cos() * j as f32 * 0.45,
                        y + h * 0.60 + j as f32 * 0.27,
                        z + a.sin() * j as f32 * 0.45,
                    ),
                    V::new(0.46, 0.46, 0.46),
                    wood,
                    MaterialId::Wood,
                    tag,
                );
            }
        }
        for ix in -3..=3 {
            for iy in -2..=2 {
                for iz in -3..=3 {
                    let p = V::new(ix as f32, iy as f32 * 1.5, iz as f32);
                    if p.length() < 3.5 && r.f() > 0.08 {
                        s.cube(
                            V::new(
                                x + ix as f32 * 0.83,
                                y + h + iy as f32 * 0.75,
                                z + iz as f32 * 0.83,
                            ),
                            V::new(0.90, 0.90, 0.90),
                            green * r.range(0.75, 1.30),
                            MaterialId::Organic,
                            tag,
                        );
                    }
                }
            }
        }
    } else {
        for layer in 0..9 {
            let radius = 3. - layer as f32 * 0.28;
            for ix in -3i32..=3 {
                for iz in -3i32..=3 {
                    if ((ix * ix + iz * iz) as f32).sqrt() > radius {
                        continue;
                    }
                    s.cube(
                        V::new(
                            x + ix as f32 * 0.72,
                            y + h * 0.38 + layer as f32 * h * 0.075,
                            z + iz as f32 * 0.72,
                        ),
                        V::new(0.80, 0.66, 0.80),
                        green * r.range(0.80, 1.25),
                        MaterialId::Organic,
                        tag,
                    );
                }
            }
        }
    }
}
fn fern(s: &mut Scene, x: f32, z: f32, scale: f32) {
    let y = ground(x, z);
    for j in 0..6 {
        let a = j as f32 * std::f32::consts::TAU / 6.;
        for k in 1..6 {
            let d = k as f32 * 0.18 * scale;
            let p = V::new(
                x + a.cos() * d,
                y + (0.24 + (k as f32 / 6. * 3.14).sin() * 0.48) * scale,
                z + a.sin() * d,
            );
            s.cube(
                p,
                V::new(0.25, 0.18, 0.25) * scale,
                V::new(0.31, 0.40, 0.17),
                MaterialId::Organic,
                Tag::Fern,
            );
        }
    }
}
struct Model<'a> {
    scene: &'a mut Scene,
    origin: V,
    scale: f32,
    reverse: bool,
    tag: Tag,
    color: V,
}
impl Model<'_> {
    fn block(&mut self, p: V, step: f32, c: V) {
        let p = if self.reverse {
            V::new(-p.x, p.y, -p.z)
        } else {
            p
        };
        self.scene.cube(
            self.origin + p * self.scale,
            V::new(step, step, step) * self.scale,
            c,
            MaterialId::Organic,
            self.tag,
        );
    }
    fn ellipsoid(&mut self, c: V, r: V, color: V) {
        let step = if matches!(self.tag, Tag::Scapherpeton | Tag::Mammal | Tag::CrownBird) {
            0.045
        } else if matches!(
            self.tag,
            Tag::Enantiornithine
                | Tag::Ammonite
                | Tag::HorseshoeCrab
                | Tag::Bivalve
                | Tag::Nautiloid
        ) {
            0.10
        } else {
            0.22
        };
        let nx = (r.x / step).ceil() as i32;
        let ny = (r.y / step).ceil() as i32;
        let nz = (r.z / step).ceil() as i32;
        for x in -nx..=nx {
            for y in -ny..=ny {
                for z in -nz..=nz {
                    let p = V::new(x as f32 * step, y as f32 * step, z as f32 * step);
                    let q = V::new(p.x / r.x, p.y / r.y, p.z / r.z);
                    if q.dot(q) > 1. {
                        continue;
                    } // Only the shell needs primary-ray intersection.
                    let interior = [
                        V::new(step, 0., 0.),
                        V::new(-step, 0., 0.),
                        V::new(0., step, 0.),
                        V::new(0., -step, 0.),
                        V::new(0., 0., step),
                        V::new(0., 0., -step),
                    ]
                    .iter()
                    .all(|d| {
                        let t = p + *d;
                        let q = V::new(t.x / r.x, t.y / r.y, t.z / r.z);
                        q.dot(q) < 1.
                    });
                    if interior {
                        continue;
                    }
                    let speck = 1. + ((x * 7 + y * 11 + z * 3) % 7) as f32 * 0.012;
                    self.block(c + p, step * 1.01, color * speck);
                }
            }
        }
    }
    fn segment(&mut self, a: V, b: V, r: f32, c: V) {
        let spacing = (r * 1.5).clamp(0.025, 0.20);
        let steps = ((b - a).length() / spacing).ceil() as usize;
        for i in 0..=steps {
            let p = a + (b - a) * (i as f32 / steps.max(1) as f32);
            self.ellipsoid(p, V::new(r, r, r), c);
        }
    }
    fn tail(&mut self, start: V, length: f32, r: f32) {
        for i in 0..18 {
            let t = i as f32 / 17.;
            self.ellipsoid(
                start + V::new(-length * t, -0.4 * t, (t * 3.).sin() * 0.25),
                V::new(0.36, r * (1. - t * 0.88), r * (1. - t * 0.88)),
                self.color,
            );
        }
    }
    fn eyes(&mut self, x: f32, y: f32, z: f32) {
        let eye_scale = if self.tag == Tag::Enantiornithine {
            0.40
        } else {
            1.0
        };
        for side in [-1., 1.] {
            self.block(
                V::new(x, y, z * side),
                0.17 * eye_scale,
                V::new(0.66, 0.48, 0.19),
            );
            self.block(
                V::new(x + 0.04 * eye_scale, y, z * side + side * 0.08 * eye_scale),
                0.09 * eye_scale,
                V::new(0.02, 0.025, 0.015),
            );
        }
    }
}
fn animal(s: &mut Scene, x: f32, z: f32, scale: f32, reverse: bool, tag: Tag) {
    let start = s.cubes.len();
    let y = ground(x, z) + 0.05;
    let color = match tag {
        Tag::Tyrannosaurus => V::new(0.40, 0.35, 0.24),
        Tag::Triceratops => V::new(0.49, 0.37, 0.24),
        Tag::Edmontosaurus => V::new(0.41, 0.46, 0.29),
        _ => V::new(0.37, 0.39, 0.27),
    };
    let mut m = Model {
        scene: s,
        origin: V::new(x, y, z),
        scale,
        reverse,
        tag,
        color,
    };
    match tag {
        Tag::Tyrannosaurus => {
            m.ellipsoid(V::new(0., 2.8, 0.), V::new(2., 1.1, 0.77), color);
            m.ellipsoid(V::new(1.9, 3.6, 0.), V::new(0.7, 0.95, 0.61), color);
            m.ellipsoid(V::new(2.85, 4.15, 0.), V::new(1.0, 0.62, 0.68), color);
            m.ellipsoid(
                V::new(3.0, 3.70, 0.),
                V::new(0.88, 0.20, 0.53),
                color * 0.75,
            );
            m.eyes(2.65, 4.35, 0.63);
            for side in [-1., 1.] {
                m.segment(
                    V::new(-0.65, 2.2, side * 0.64),
                    V::new(-0.1, 1.25, side * 0.78),
                    0.44,
                    color,
                );
                m.segment(
                    V::new(-0.1, 1.25, side * 0.78),
                    V::new(-0.4, 0.35, side * 0.78),
                    0.26,
                    color,
                );
                m.ellipsoid(
                    V::new(0.0, 0.16, side * 0.78),
                    V::new(0.70, 0.20, 0.38),
                    color * 0.7,
                );
                m.segment(
                    V::new(1.2, 2.65, side * 0.66),
                    V::new(1.45, 2.05, side * 0.72),
                    0.13,
                    color,
                );
                for finger in [0., 0.16] {
                    m.segment(
                        V::new(1.45, 2.05, side * 0.72 + finger),
                        V::new(1.75, 2.10, side * 0.72 + finger),
                        0.07,
                        V::new(0.69, 0.62, 0.43),
                    );
                }
            }
            m.tail(V::new(-1.7, 2.7, 0.), 5.0, 0.69);
            for i in 0..12 {
                m.block(V::new(-1.5 + i as f32 * 0.27, 3.75, 0.), 0.22, color * 0.60);
            }
        }
        Tag::Triceratops => {
            m.ellipsoid(V::new(0., 1.7, 0.), V::new(2., 1.25, 1.0), color);
            for a in [-1.1, 1.] {
                for side in [-1., 1.] {
                    m.segment(
                        V::new(a, 1.7, side * 0.73),
                        V::new(a + 0.1, 0.28, side * 0.80),
                        0.30,
                        color * 0.85,
                    );
                    m.ellipsoid(
                        V::new(a + 0.25, 0.15, side * 0.80),
                        V::new(0.48, 0.20, 0.37),
                        color * 0.65,
                    );
                }
            }
            m.ellipsoid(V::new(2.25, 1.60, 0.), V::new(1.25, 0.65, 0.67), color);
            m.ellipsoid(
                V::new(1.45, 2.40, 0.),
                V::new(0.30, 1.28, 1.23),
                color * 0.90,
            );
            m.ellipsoid(
                V::new(1.73, 2.53, 0.),
                V::new(0.17, 0.93, 0.89),
                V::new(0.60, 0.39, 0.24),
            );
            m.eyes(2.30, 1.90, 0.63);
            let horn = V::new(0.76, 0.70, 0.49);
            for side in [-1., 1.] {
                for i in 0..8 {
                    let t = i as f32 / 7.;
                    m.ellipsoid(
                        V::new(2.15 + t * 1.15, 2.10 + t * 0.85, side * 0.52),
                        V::new(0.19 - t * 0.13, 0.19 - t * 0.13, 0.19 - t * 0.13),
                        horn,
                    );
                }
            }
            m.segment(V::new(3., 1.95, 0.), V::new(3.2, 2.48, 0.), 0.14, horn);
            m.ellipsoid(V::new(3.4, 1.3, 0.), V::new(0.28, 0.29, 0.35), color * 0.5);
            m.tail(V::new(-1.6, 1.5, 0.), 2.4, 0.50);
        }
        Tag::Edmontosaurus => {
            m.ellipsoid(V::new(0., 2., 0.), V::new(2.2, 1.15, 0.85), color);
            m.segment(V::new(1.5, 2.35, 0.), V::new(2.5, 2.95, 0.), 0.51, color);
            m.ellipsoid(V::new(3.05, 3.1, 0.), V::new(0.84, 0.44, 0.41), color);
            m.ellipsoid(
                V::new(3.68, 2.87, 0.),
                V::new(0.43, 0.15, 0.47),
                color * 0.63,
            );
            m.eyes(2.85, 3.3, 0.40);
            for side in [-1., 1.] {
                m.segment(
                    V::new(-0.85, 2., side * 0.63),
                    V::new(-0.5, 0.25, side * 0.70),
                    0.38,
                    color * 0.85,
                );
                m.segment(
                    V::new(1.25, 1.9, side * 0.52),
                    V::new(1.58, 0.20, side * 0.58),
                    0.24,
                    color * 0.80,
                );
                m.ellipsoid(
                    V::new(-0.2, 0.14, side * 0.70),
                    V::new(0.57, 0.18, 0.33),
                    color * 0.6,
                );
                m.block(V::new(1.7, 0.14, side * 0.58), 0.33, color * 0.55);
            }
            m.tail(V::new(-1.6, 2., 0.), 4.0, 0.66);
            for i in 0..15 {
                m.block(V::new(-1.8 + i as f32 * 0.27, 3.07, 0.), 0.27, color * 0.62);
            }
        }
        Tag::Ankylosaurus => {
            m.ellipsoid(V::new(0., 1.15, 0.), V::new(2., 0.95, 1.15), color);
            for x in [-1.1, 1.] {
                for side in [-1., 1.] {
                    m.segment(
                        V::new(x, 1.15, side * 0.75),
                        V::new(x + 0.1, 0.20, side * 0.9),
                        0.32,
                        color * 0.83,
                    );
                }
            }
            m.ellipsoid(V::new(2.05, 1.05, 0.), V::new(0.75, 0.40, 0.58), color);
            m.eyes(2.25, 1.22, 0.52);
            for x in -5i32..=5 {
                for z in -3i32..=3 {
                    if x.abs() + z.abs() > 7 {
                        continue;
                    }
                    let y = 1.8 - (z as f32 * 0.07).abs() - (x as f32 * 0.04).abs();
                    m.ellipsoid(
                        V::new(x as f32 * 0.34, y, z as f32 * 0.30),
                        V::new(0.22, 0.25, 0.21),
                        V::new(0.61, 0.58, 0.38),
                    );
                }
            }
            m.tail(V::new(-1.5, 1.0, 0.), 2.8, 0.36);
            m.ellipsoid(
                V::new(-4.4, 0.65, 0.),
                V::new(0.66, 0.38, 0.76),
                V::new(0.56, 0.52, 0.34),
            );
        }
        _ => {}
    }
    s.register_animal(start);
}
fn bird(s: &mut Scene, p: V) {
    let start = s.cubes.len();
    let c = V::new(0.33, 0.27, 0.18);
    let mut m = Model {
        scene: s,
        origin: p,
        scale: 1.,
        reverse: false,
        tag: Tag::Enantiornithine,
        color: c,
    };
    m.ellipsoid(V::new(0., 0.35, 0.), V::new(0.45, 0.30, 0.25), c);
    m.ellipsoid(V::new(0.30, 0.65, 0.), V::new(0.22, 0.23, 0.21), c);
    m.block(V::new(0.52, 0.62, 0.), 0.16, V::new(0.51, 0.42, 0.24));
    m.eyes(0.36, 0.71, 0.18);
    for side in [-1., 1.] {
        m.segment(
            V::new(0., 0.35, side * 0.2),
            V::new(-0.35, 0.30, side * 0.9),
            0.13,
            c,
        );
        m.segment(
            V::new(-0.35, 0.30, side * 0.9),
            V::new(-0.80, 0.30, side * 1.1),
            0.10,
            c * 0.68,
        );
        m.segment(
            V::new(0.1, 0.20, side * 0.13),
            V::new(0.13, 0., side * 0.13),
            0.07,
            V::new(0.54, 0.43, 0.25),
        );
    }
    m.segment(
        V::new(-0.3, 0.35, 0.),
        V::new(-0.90, 0.40, 0.),
        0.10,
        c * 0.7,
    );
    s.register_animal(start);
}
pub const BIRD_SITES: [(f32, f32, f32); 6] = [
    (1., 7., 0.73),
    (-30., 12., 2.8),
    (33., -7., 3.5),
    (-15., -24., 1.8),
    (38., 24., 0.15),
    (-40., -12., 4.0),
];
pub const AMPHIBIAN_SITES: [(f32, f32); 4] = [(-23., -14.), (-15., -20.), (5., 4.5), (28., 2.0)];
fn amphibian(s: &mut Scene, x: f32, z: f32) {
    let start = s.cubes.len();
    let c = V::new(0.20, 0.24, 0.12);
    let mut m = Model {
        scene: s,
        origin: V::new(x, ground(x, z), z),
        scale: 1.0,
        reverse: false,
        tag: Tag::Scapherpeton,
        color: c,
    };
    m.ellipsoid(V::new(0., 0.16, 0.), V::new(0.32, 0.11, 0.13), c);
    m.ellipsoid(V::new(0.30, 0.17, 0.), V::new(0.17, 0.09, 0.15), c);
    for i in 0..12 {
        let t = i as f32 / 11.;
        m.ellipsoid(
            V::new(-0.25 - t * 0.55, 0.14 - t * 0.06, t * t * 0.12),
            V::new(0.08, 0.08 * (1. - t * 0.75), 0.09 * (1. - t * 0.75)),
            c,
        );
    }
    for x in [-0.18, 0.20] {
        for side in [-1., 1.] {
            m.segment(
                V::new(x, 0.14, side * 0.08),
                V::new(x - 0.09, 0.03, side * 0.23),
                0.04,
                c,
            );
        }
    }
    for side in [-1., 1.] {
        m.block(
            V::new(0.34, 0.22, side * 0.10),
            0.04,
            V::new(0.02, 0.025, 0.015),
        );
    }
    s.register_animal(start);
}
pub const SURVIVOR_SITE: V = V::new(-13., 2., 14.);
fn mammal(s: &mut Scene, x: f32, z: f32) {
    let start = s.cubes.len();
    let c = V::new(0.32, 0.23, 0.16);
    let mut m = Model {
        scene: s,
        origin: V::new(x, ground(x, z), z),
        scale: 1.,
        reverse: false,
        tag: Tag::Mammal,
        color: c,
    };
    m.ellipsoid(V::new(0., 0.28, 0.), V::new(0.53, 0.26, 0.25), c);
    m.ellipsoid(V::new(0.48, 0.39, 0.), V::new(0.26, 0.23, 0.20), c);
    m.ellipsoid(V::new(0.69, 0.31, 0.), V::new(0.21, 0.09, 0.12), c * 0.8);
    for side in [-1., 1.] {
        m.ellipsoid(
            V::new(0.38, 0.63, side * 0.14),
            V::new(0.10, 0.15, 0.06),
            c * 0.8,
        );
        m.block(
            V::new(0.60, 0.44, side * 0.17),
            0.05,
            V::new(0.01, 0.01, 0.01),
        );
        for x in [-0.3, 0.3] {
            m.segment(
                V::new(x, 0.23, side * 0.15),
                V::new(x + 0.1, 0.06, side * 0.25),
                0.08,
                c * 0.65,
            );
        }
    }
    m.segment(
        V::new(-0.43, 0.25, 0.),
        V::new(-1.2, 0.11, 0.22),
        0.065,
        c * 0.65,
    );
    s.register_animal(start);
}
fn survivor_bird(s: &mut Scene, x: f32, z: f32) {
    let start = s.cubes.len();
    let c = V::new(0.36, 0.31, 0.22);
    let mut m = Model {
        scene: s,
        origin: V::new(x, ground(x, z), z),
        scale: 1.,
        reverse: false,
        tag: Tag::CrownBird,
        color: c,
    };
    m.ellipsoid(V::new(0., 0.46, 0.), V::new(0.38, 0.32, 0.25), c);
    m.ellipsoid(V::new(0.29, 0.77, 0.), V::new(0.20, 0.20, 0.18), c);
    m.block(V::new(0.50, 0.73, 0.), 0.15, V::new(0.45, 0.35, 0.19));
    for side in [-1., 1.] {
        m.ellipsoid(
            V::new(-0.1, 0.45, side * 0.22),
            V::new(0.29, 0.20, 0.075),
            c * 0.65,
        );
        m.segment(
            V::new(0., 0.27, side * 0.12),
            V::new(0.10, 0.04, side * 0.14),
            0.055,
            c * 0.6,
        );
        m.block(
            V::new(0.35, 0.81, side * 0.16),
            0.04,
            V::new(0.01, 0.01, 0.01),
        );
    }
    m.segment(
        V::new(-0.25, 0.5, 0.),
        V::new(-0.56, 0.58, 0.),
        0.10,
        c * 0.6,
    );
    s.register_animal(start);
}
pub fn crater_height(x: f32, z: f32) -> f32 {
    let p = crate::disaster::IMPACT;
    let r = (x - p.x).hypot(z - p.z);
    if r <= 12. {
        -8. + 10. * (r / 12.).powi(2)
    } else {
        ground(x, z) + (1. - ((r - 13.5) / 2.5).abs()).max(0.) * 3.8
    }
}
fn excavate_crater(s: &mut Scene) {
    let p = crate::disaster::IMPACT;
    s.cubes.retain(|c| {
        let center = (c.bounds.lo + c.bounds.hi) * 0.5;
        (center.x - p.x).hypot(center.z - p.z) > 16.
    });
    for x in -16..=16 {
        for z in -24..=8 {
            let (xf, zf) = (x as f32, z as f32);
            let r = (xf - p.x).hypot(zf - p.z);
            if r > 16. {
                continue;
            }
            let height = (crater_height(xf, zf) * 2.).floor() * 0.5;
            s.cube(
                V::new(xf, (height - 11.) * 0.5, zf),
                V::new(1.001, height + 11., 1.001),
                if r < 11. {
                    V::new(0.14, 0.12, 0.11)
                } else {
                    V::new(0.30, 0.24, 0.18)
                },
                MaterialId::Earth,
                Tag::None,
            );
        }
    }
    // Fallen trunks and ejecta around the excavated bowl, using existing materials.
    for i in 0..34 {
        let angle = i as f32 * 2.39996;
        let radius = 17. + (i % 7) as f32 * 1.3;
        let x = p.x + angle.cos() * radius;
        let z = p.z + angle.sin() * radius;
        let base = ground(x, z);
        s.cube(
            V::new(x, base + 0.5, z),
            if i % 3 == 0 {
                V::new(3.5, 0.7, 0.7)
            } else {
                V::new(0.9, 1.0 + (i % 3) as f32 * 0.4, 1.2)
            },
            V::new(0.19, 0.15, 0.12),
            if i % 3 == 0 {
                MaterialId::Wood
            } else {
                MaterialId::Earth
            },
            Tag::None,
        );
    }
}
pub fn impacted(original: &Scene) -> Scene {
    let mut s = Scene::new(true);
    s.cubes = original.cubes.clone();
    s.entities = original.entities.clone();
    excavate_crater(&mut s);
    s.finish()
}
pub fn ruins(original: &Scene) -> Scene {
    let mut s = aftermath(original);
    s.cubes.retain(|c| c.entity == 0);
    s.entities.clear();
    s.nodes.clear();
    s.finish()
}
pub fn aftermath(original: &Scene) -> Scene {
    let mut s = Scene::new(true);
    s.cubes = original
        .cubes
        .iter()
        .filter(|c| {
            c.entity == 0
                && !(c.material == MaterialId::Organic
                    && matches!(c.tag, Tag::Conifer | Tag::Broadleaf | Tag::Fern))
        })
        .cloned()
        .collect();
    // Preserve complete surviving marine individuals and rebuild their entity IDs.
    for entity in 1..=original.entities.len() {
        let start = s.cubes.len();
        s.cubes.extend(
            original
                .cubes
                .iter()
                .filter(|c| c.entity == entity && c.tag.marine_survivor())
                .cloned(),
        );
        if s.cubes.len() > start {
            s.register_animal(start);
        }
    }
    // The final tableau represents a later landscape, not animals surviving at ground zero.
    for (x, z) in [
        (-13., 14.),
        (-16., 15.),
        (16., 10.),
        (-29., -16.),
        (30., 23.),
        (29., -28.),
    ] {
        mammal(&mut s, x, z);
    }
    for (x, z) in [
        (-11., 15.),
        (-15., 12.),
        (20., 12.),
        (-24., -14.),
        (35., 18.),
        (31., -27.),
    ] {
        survivor_bird(&mut s, x, z);
    }
    excavate_crater(&mut s);
    s.finish()
}
pub fn biome() -> Scene {
    let mut s = Scene::new(true);
    let mut r = Rng(819);
    // 81 x 65 units: floodplain, channel, forest and wet backwaters.
    for x in -60i32..=60 {
        for z in -48i32..=48 {
            let (xf, zf) = (x as f32, z as f32);
            let d = (zf - river(xf)).abs();
            let pond = ((xf + 23.) / 8.).powi(2) + ((zf + 20.) / 5.).powi(2) < 1.;
            let water = d < 2.4 || pond;
            let height = if water { 0.0 } else { ground(xf, zf) };
            for layer in -4..(height.ceil() as i32) {
                s.cube(
                    V::new(xf, layer as f32 + 0.5, zf),
                    V::new(1.001, 1.001, 1.001),
                    V::new(0.30, 0.25, 0.17) * r.range(0.97, 1.03),
                    MaterialId::Earth,
                    Tag::None,
                );
            }
            if water {
                s.cube(
                    V::new(xf, 0.35, zf),
                    V::new(1., 0.25, 1.),
                    V::new(0.16, 0.34, 0.30) * r.range(0.90, 1.1),
                    MaterialId::Water,
                    Tag::None,
                );
            } else {
                let c = if height > 9. {
                    V::new(0.40, 0.39, 0.34)
                } else if d < 4. {
                    V::new(0.47, 0.41, 0.27)
                } else {
                    V::new(0.31, 0.34, 0.18)
                };
                s.cube(
                    V::new(xf, height - 0.18, zf),
                    V::new(1.001, 0.38, 1.001),
                    c * r.range(0.98, 1.02),
                    MaterialId::Earth,
                    Tag::None,
                );
            }
        }
    }
    let animal_positions = [
        (-13., 14.),
        (-22., 16.),
        (17., 10.),
        (-9., -10.),
        (-16., -12.),
        (-5., -15.),
        (22., -14.),
        (21., -21.),
    ];
    for _ in 0..420 {
        let x = r.range(-57., 57.);
        let z = r.range(-44., 44.);
        if ground(x, z) > 15.0 || (z - river(x)).abs() < 5.5 {
            continue;
        }
        if animal_positions
            .iter()
            .any(|(a, b)| ((x - a).powi(2) + (z - b).powi(2)).sqrt() < 6.5)
        {
            continue;
        }
        if z > 7. && x.abs() < 29. && r.f() < 0.90 {
            continue;
        }
        if ((x + 23.) / 9.).powi(2) + ((z + 20.) / 6.).powi(2) < 1. {
            continue;
        }
        let h = r.range(6., 11.5);
        let broad = r.f() < 0.65;
        tree(&mut s, x, z, h, broad, &mut r);
    }
    for _ in 0..490 {
        let x = r.range(-58., 58.);
        let z = r.range(-45., 45.);
        if (z - river(x)).abs() > 3.5 && ((x + 23.) / 9.).powi(2) + ((z + 20.) / 6.).powi(2) > 1. {
            fern(&mut s, x, z, r.range(0.55, 1.4));
        }
    }
    animal(&mut s, -13., 14., 1., false, Tag::Triceratops);
    animal(&mut s, -22., 16., 0.66, false, Tag::Triceratops);
    animal(&mut s, 17., 10., 1.10, true, Tag::Tyrannosaurus);
    animal(&mut s, -9., -10., 1.15, false, Tag::Edmontosaurus);
    animal(&mut s, -16., -12., 0.95, false, Tag::Edmontosaurus);
    animal(&mut s, -5., -15., 0.60, false, Tag::Edmontosaurus);
    animal(&mut s, 22., -14., 1.0, true, Tag::Ankylosaurus);
    animal(&mut s, 21., -21., 0.72, true, Tag::Ankylosaurus);
    for i in 0..9 {
        s.cube(
            V::new(-1. + i as f32 * 0.60, ground(0., 7.) + 0.4, 7.),
            V::new(0.64, 0.64, 0.64),
            V::new(0.30, 0.24, 0.16),
            MaterialId::Wood,
            Tag::None,
        );
    }
    for (x, z, h) in BIRD_SITES {
        let y = ground(x, z) + h;
        if h > 0.8 {
            s.cube(
                V::new(x - 0.45, ground(x, z) + h * 0.5 - 0.15, z),
                V::new(0.42, h - 0.3, 0.42),
                V::new(0.30, 0.24, 0.16),
                MaterialId::Wood,
                Tag::None,
            );
            s.cube(
                V::new(x, y - 0.25, z),
                V::new(1.5, 0.45, 0.65),
                V::new(0.30, 0.24, 0.16),
                MaterialId::Wood,
                Tag::None,
            );
        }
        bird(&mut s, V::new(x, y, z));
    }
    for (x, z) in AMPHIBIAN_SITES {
        amphibian(&mut s, x, z);
    }
    for _ in 0..85 {
        let x = r.range(-58., 58.);
        let z = river(x) + (if r.f() < 0.5 { -1. } else { 1. }) * r.range(2.6, 3.5);
        s.cube(
            V::new(x, 0.75, z),
            V::new(0.65, 0.65, 0.65),
            V::new(0.43, 0.43, 0.32) * r.range(0.8, 1.2),
            MaterialId::Earth,
            Tag::None,
        );
    }
    marine::coast(&mut s);
    marine::fauna(&mut s);
    // Separate cubic mineral crystals: each has closed entry/exit interfaces.
    for (x, z, width, height) in [
        (-32., 51.5, 2.8, 4.5),
        (-35., 51., 1.6, 2.7),
        (-29.3, 52., 1.6, 2.4),
    ] {
        let base = marine::seabed(x, z);
        s.cube(
            V::new(x, base - 0.1, z),
            V::new(width + 0.5, 0.4, width + 0.5),
            V::new(0.36, 0.31, 0.27),
            MaterialId::Earth,
            Tag::None,
        );
        s.cube(
            V::new(x, base + height * 0.5, z),
            V::new(width, height, width),
            V::new(0.90, 0.97, 1.),
            MaterialId::Crystal,
            Tag::Mineral,
        );
    }
    s.finish()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn birds_are_dispersed_and_mountains_rise_above_valley() {
        for (i, a) in BIRD_SITES.iter().enumerate() {
            for b in &BIRD_SITES[i + 1..] {
                assert!((a.0 - b.0).hypot(a.1 - b.1) > 12.0);
            }
        }
        assert!(ground(-35., -38.) > 20.);
        assert!(ground(17., 10.) < 4.);
    }
    #[test]
    fn bvh_matches_brute_force() {
        let s = biome();
        let mut rng = Rng(21);
        for _ in 0..90 {
            let ray = Ray {
                o: V::new(rng.range(-35., 35.), rng.range(5., 30.), 45.),
                d: V::new(rng.range(-0.5, 0.5), rng.range(-0.9, -0.1), -1.).norm(),
            };
            let a = s.hit(ray, 200.).map(|h| h.t);
            let b = s
                .cubes
                .iter()
                .filter_map(|c| c.bounds.hit(ray, 200.).map(|p| p.0))
                .min_by(f32::total_cmp);
            match (a, b) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 0.0001),
                (None, None) => {}
                _ => panic!("BVH mismatch"),
            }
        }
    }
    #[test]
    fn shadow_and_parallel_queries_match_reference() {
        let s = biome();
        let mut rng = Rng(77);
        for i in 0..120 {
            let ray = Ray {
                o: V::new(
                    rng.range(-60., 60.),
                    rng.range(-2., 35.),
                    rng.range(-48., 48.),
                ),
                d: match i % 4 {
                    0 => V::new(0., -1., 0.),
                    1 => V::new(1., 0., 0.),
                    2 => V::new(0., 0., 1.),
                    _ => V::new(-0.5, 0.85, 0.55).norm(),
                },
            };
            for limit in [0.1, 3., 35., 500.] {
                let expected = s
                    .cubes
                    .iter()
                    .filter_map(|c| c.bounds.hit(ray, limit).map(|h| h.0))
                    .min_by(f32::total_cmp);
                assert_eq!(s.occluded(ray, limit), expected.is_some());
                match (s.hit(ray, limit), expected) {
                    (Some(a), Some(b)) => assert!((a.t - b).abs() < 0.0001),
                    (None, None) => {}
                    _ => panic!("parallel/inside BVH mismatch"),
                }
            }
        }
    }
    #[test]
    fn individual_selection_and_aftermath_fauna() {
        let original = biome();
        let triceratops: Vec<_> = original
            .cubes
            .iter()
            .enumerate()
            .filter(|(_, c)| c.tag == Tag::Triceratops)
            .collect();
        let ids: std::collections::BTreeSet<_> =
            triceratops.iter().map(|(_, c)| c.entity).collect();
        assert_eq!(ids.len(), 2);
        assert!(!ids.contains(&0));
        for (index, cube) in triceratops.iter().step_by(97) {
            let bounds = original.animal_bounds(*index).unwrap();
            assert!(bounds.lo.x <= cube.bounds.lo.x && bounds.hi.x >= cube.bounds.hi.x);
            assert!((bounds.hi - bounds.lo).length() < 16.);
        }
        let after = aftermath(&original);
        for tag in [
            Tag::Tyrannosaurus,
            Tag::Triceratops,
            Tag::Edmontosaurus,
            Tag::Ankylosaurus,
            Tag::Enantiornithine,
        ] {
            assert!(!after.cubes.iter().any(|c| c.tag == tag));
        }
        for tag in [Tag::Mammal, Tag::CrownBird] {
            assert!(after.cubes.iter().any(|c| c.tag == tag && c.entity > 0));
        }
        assert!(after.cubes.iter().any(|c| c.material == MaterialId::Wood));
        assert!(
            !after.cubes.iter().any(|c| c.material == MaterialId::Organic
                && matches!(c.tag, Tag::Conifer | Tag::Broadleaf | Tag::Fern))
        );
        assert_eq!(after.entities.len(), 25);
    }
    #[test]
    fn coast_extinctions_preserve_surviving_individuals() {
        let original = biome();
        let after = aftermath(&original);
        for tag in [
            Tag::Mosasaur,
            Tag::Plesiosaur,
            Tag::Ammonite,
            Tag::HorseshoeCrab,
            Tag::Bivalve,
            Tag::Nautiloid,
        ] {
            let before: Vec<_> = original.cubes.iter().filter(|c| c.tag == tag).collect();
            assert!(!before.is_empty());
            let remaining: Vec<_> = after
                .cubes
                .iter()
                .enumerate()
                .filter(|(_, c)| c.tag == tag)
                .collect();
            if tag.marine_survivor() {
                assert_eq!(before.len(), remaining.len());
                for (index, cube) in remaining {
                    let bounds = after.animal_bounds(index).unwrap();
                    assert!(bounds.lo.x <= cube.bounds.lo.x && bounds.hi.x >= cube.bounds.hi.x);
                    assert!(bounds.lo.z > 50.);
                }
            } else {
                assert!(remaining.is_empty());
            }
        }
        for (view, tag) in [(10, Tag::Mosasaur), (11, Tag::Plesiosaur)] {
            let camera = crate::render::Camera::preset(view);
            let hit = original
                .hit(camera.ray(550., 378., 1100, 756), 500.)
                .unwrap();
            assert_eq!(original.cubes[hit.index].tag, tag);
            assert!(original.animal_bounds(hit.index).is_some());
        }
        assert!(marine::seabed(0., 49.) > marine::seabed(0., 70.));
    }
    #[test]
    fn scene_uses_only_five_materials_and_crystals_survive() {
        let scene = biome();
        let mut present = [false; 5];
        for c in &scene.cubes {
            present[c.material as usize] = true;
        }
        assert!(present.iter().all(|x| *x));
        let after = aftermath(&scene);
        assert_eq!(
            scene
                .cubes
                .iter()
                .filter(|c| c.material == MaterialId::Crystal)
                .count(),
            3
        );
        assert_eq!(
            after
                .cubes
                .iter()
                .filter(|c| c.material == MaterialId::Crystal)
                .count(),
            3
        );
        // A normal ray through an isolated crystal loses light but does not cast an opaque shadow.
        let mut glass = Scene::new(false);
        glass.cube(
            V::default(),
            V::new(2., 2., 2.),
            V::new(1., 1., 1.),
            MaterialId::Crystal,
            Tag::Mineral,
        );
        let glass = glass.finish();
        let visibility = glass.light_visibility(
            Ray {
                o: V::new(0., 0., 3.),
                d: V::new(0., 0., -1.),
            },
            10.,
        );
        assert!(visibility > 0.8 && visibility < 1.);
    }
    #[test]
    fn crater_has_depth_and_intermediate_scene_has_no_survivors() {
        let original = biome();
        let damaged = impacted(&original);
        let p = crate::disaster::IMPACT;
        let hit = damaged
            .hit(
                Ray {
                    o: V::new(p.x, 30., p.z),
                    d: V::new(0., -1., 0.),
                },
                100.,
            )
            .unwrap();
        assert!((30. - hit.t + 8.).abs() < 0.01);
        assert!(crater_height(13.5, p.z) > ground(13.5, p.z) + 3.);
        let empty = ruins(&original);
        assert!(empty.cubes.iter().all(|c| c.entity == 0));
        let final_scene = aftermath(&original);
        assert!(final_scene.cubes.iter().any(|c| c.tag == Tag::Mammal));
        assert!(final_scene.cubes.iter().any(|c| c.tag == Tag::CrownBird));
    }
    #[test]
    fn moving_preview_keeps_cast_shadows() {
        let mut clear = Scene::new(false);
        clear.cube(
            V::new(0., -0.5, 0.),
            V::new(20., 1., 20.),
            V::new(0.6, 0.6, 0.6),
            MaterialId::Earth,
            Tag::None,
        );
        let mut blocked = Scene::new(false);
        blocked.cubes = clear.cubes.clone();
        blocked.cube(
            V::new(0., 1., 0.),
            V::new(2., 2., 2.),
            V::new(0.3, 0.3, 0.3),
            MaterialId::Earth,
            Tag::None,
        );
        let cam = crate::render::Camera {
            yaw: 0.,
            pitch: 1.35,
            distance: 8.,
            target: V::new(1.5, 0., -1.5),
        };
        let a = crate::render::render(&clear.finish(), cam, 1, 1, 0., false);
        let b = crate::render::render(&blocked.finish(), cam, 1, 1, 0., false);
        assert!(
            b[0] as i32 + 15 < a[0] as i32,
            "moving render lost its shadow: {a:?} {b:?}"
        );
    }
    #[test]
    fn all_selected_taxa_present() {
        let s = biome();
        for tag in [
            Tag::Tyrannosaurus,
            Tag::Triceratops,
            Tag::Edmontosaurus,
            Tag::Ankylosaurus,
            Tag::Enantiornithine,
            Tag::Scapherpeton,
        ] {
            assert!(s.cubes.iter().any(|c| c.tag == tag));
        }
        assert!(s.cubes.len() > 30000);
    }
}
