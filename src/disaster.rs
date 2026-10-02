//! Artistic compressed timeline. The later survivors are not inhabitants of the crater.
use crate::material::MaterialId;
use crate::math::*;
#[derive(Clone, Copy, Debug, Default)]
pub struct Disaster {
    pub elapsed: Option<f32>,
}
pub const END: f32 = 30.;
pub const IMPACT: V = V::new(0., 2., -8.);
impl Disaster {
    pub fn at(seconds: f32) -> Self {
        Self {
            elapsed: Some(seconds.clamp(0., END)),
        }
    }
    pub fn active(self) -> bool {
        self.elapsed.is_some_and(|t| t < END)
    }
    pub fn aftermath(self) -> bool {
        self.elapsed.is_some_and(|t| t >= END)
    }
    pub fn stage(self) -> usize {
        match self.elapsed {
            None => 0,
            Some(t) if t < 3. => 0,
            Some(t) if t < 12. => 1,
            Some(t) if t < END => 2,
            _ => 3,
        }
    }
    pub fn tsunami(self) -> bool {
        self.elapsed.is_some_and(|t| (8.0..26.0).contains(&t))
    }
    pub fn water_height(self, x: f32, z: f32) -> f32 {
        if !self.tsunami() {
            return 0.45;
        }
        let t = self.elapsed.unwrap();
        let envelope = ((t - 8.) / 2.).clamp(0., 1.) * ((26. - t) / 3.).clamp(0., 1.);
        let mut height = 0.;
        for i in 0..3 {
            let center = 99. - (t - 8. - i as f32 * 2.7) * 4.2 + (x * 0.08).sin() * 1.3;
            let q = (z - center) / (4.8 + i as f32);
            let crest = (1. - q * q).max(0.).powi(2);
            height += crest * (8. - i as f32 * 1.5);
        }
        0.45 + height * envelope * ((z - 43.) / 10.).clamp(0., 1.)
    }
    pub fn water_normal(self, p: V) -> V {
        let dx = (self.water_height(p.x + 0.10, p.z) - self.water_height(p.x - 0.10, p.z)) / 0.20;
        let dz = (self.water_height(p.x, p.z + 0.10) - self.water_height(p.x, p.z - 0.10)) / 0.20;
        V::new(-dx, 1., -dz).norm()
    }
    pub fn burn(self) -> f32 {
        self.elapsed.map_or(0., |t| ((t - 3.) / 5.).clamp(0., 1.))
    }
    pub fn ash(self) -> f32 {
        self.elapsed.map_or(0., |t| ((t - 16.) / 10.).clamp(0., 1.))
    }
    pub fn label(self) -> &'static str {
        match self.elapsed {
            None => "LANZAR METEORITO",
            Some(t) if t < 3. => "METEORITO EN CAMINO",
            Some(t) if t < 6. => "IMPACTO",
            Some(t) if t < 8. => "BOSQUE EN LLAMAS",
            Some(t) if t < 26. => "TSUNAMIS",
            Some(t) if t < END => "CENIZA Y SILENCIO",
            _ => "REINICIAR BIOMA",
        }
    }
    pub fn meteor(self, ray: Ray, limit: f32) -> Option<V> {
        let t = self.elapsed?;
        if t >= 3. {
            return None;
        }
        let trajectory = V::new(-38., 65., -30.);
        let center = IMPACT + trajectory * (1. - t / 3.);
        let mut best = limit;
        let mut color = None;
        for i in 0..9 {
            let k = i as f32;
            let p = center + trajectory.norm() * k * 2.4;
            let radius = (2.6 - k * 0.22).max(0.5);
            let bounds = Bounds {
                lo: p - V::new(radius, radius, radius),
                hi: p + V::new(radius, radius, radius),
            };
            if let Some((distance, normal)) = bounds.hit(ray, best) {
                best = distance;
                color = Some(if i == 0 {
                    V::new(0.65, 0.22, 0.055) * (0.7 + normal.y.abs() * 0.3)
                } else {
                    V::new(1., 0.68 - k * 0.045, 0.08)
                });
            }
        }
        color
    }
    pub fn atmosphere(self, color: V, ray: Ray) -> V {
        let dust = (self.burn() * 0.8).max(self.ash());
        let gray = V::new(0.27, 0.28, 0.30) + V::new(0.13, 0.13, 0.13) * ray.d.y.max(0.);
        color.mix(V::new(0.45, 0.24, 0.11).mix(gray, self.ash()), dust)
    }
    pub fn surface(
        self,
        color: V,
        p: V,
        material: MaterialId,
        survivor: bool,
        combustible: bool,
        time: f32,
    ) -> V {
        let burn = self.burn();
        if burn == 0. || matches!(material, MaterialId::Water | MaterialId::Crystal) {
            return color;
        }
        let gray = color.dot(V::new(0.299, 0.587, 0.114));
        let charred = if material == MaterialId::Wood {
            V::new(0.055, 0.042, 0.033)
        } else {
            V::new(gray * 0.58, gray * 0.32, gray * 0.16)
        };
        let ash = if material == MaterialId::Wood {
            V::new(0.075, 0.077, 0.08)
        } else {
            V::new(gray * 0.65 + 0.12, gray * 0.65 + 0.12, gray * 0.65 + 0.13)
        };
        let mut out = color.mix(
            charred.mix(ash, self.ash()),
            if survivor { burn * 0.18 } else { burn },
        );
        if combustible && self.ash() < 1. {
            let turbulence =
                crate::render::noise(V::new(p.x * 1.3, p.y * 0.7 - time * 2., p.z * 1.3));
            let flame = ((turbulence - 0.28) * 2.).clamp(0., 1.).powi(2);
            out = out
                + V::new(1., 0.27 + flame * 0.4, 0.015) * flame * burn * (1. - self.ash()) * 1.5;
        }
        if let Some(t) = self.elapsed {
            let distance = (p.x - IMPACT.x).hypot(p.z - IMPACT.z);
            let ring = (1. - (distance - (t - 3.) * 28.).abs() / 3.).max(0.);
            if (3.0..6.0).contains(&t) {
                out = out + V::new(1., 0.65, 0.2) * ring;
            }
            let crater = (1. - distance / 8.).clamp(0., 1.) * burn;
            if material == MaterialId::Earth {
                out = out.mix(V::new(0.035, 0.03, 0.027), crater);
            }
        }
        out
    }
    pub fn flash(self) -> f32 {
        self.elapsed.map_or(0., |t| {
            if (3.0..3.7).contains(&t) {
                (1. - (t - 3.) / 0.7) * 0.8
            } else {
                0.
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tsunami_advances_recedes_and_precedes_survivors() {
        let a = Disaster::at(12.);
        let b = Disaster::at(16.);
        assert!(a.water_height(0., 82.2) > 7.);
        assert!(b.water_height(0., 65.4) > 7.);
        assert!(!a.aftermath() && !b.aftermath());
        assert_eq!(Disaster::at(28.).water_height(0., 65.4), 0.45);
        assert_eq!(Disaster::at(2.).stage(), 0);
        assert_eq!(Disaster::at(4.).stage(), 1);
        assert_eq!(Disaster::at(18.).stage(), 2);
        assert_eq!(Disaster::at(30.).stage(), 3);
    }
    #[test]
    fn timeline_has_flight_fire_and_ash_then_stops() {
        assert_eq!(Disaster::at(2.).burn(), 0.);
        assert!(Disaster::at(8.).burn() > 0.9);
        assert!(!Disaster::at(8.).aftermath());
        assert!(!Disaster::at(18.).aftermath());
        assert!(Disaster::at(30.).aftermath());
        assert_eq!(Disaster::at(30.).ash(), 1.);
        assert!(!Disaster::at(30.).active());
    }
}
