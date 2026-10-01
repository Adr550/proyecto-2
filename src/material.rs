//! The entire diorama uses exactly these five materials. Per-object colors are tints.
use crate::math::V;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum MaterialId {
    Earth,
    Wood,
    Organic,
    Water,
    Crystal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Texture {
    Strata,
    Grain,
    Cells,
    Ripples,
    Inclusions,
}
#[derive(Clone, Copy)]
pub struct Material {
    pub albedo: V,
    pub specular: f32,
    pub shininess: i32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
    pub roughness: f32,
    pub texture: Texture,
}
pub const MATERIALS: [Material; 5] = [
    Material {
        albedo: V::new(0.92, 0.88, 0.78),
        specular: 0.06,
        shininess: 12,
        transparency: 0.,
        reflectivity: 0.,
        ior: 1.,
        roughness: 0.09,
        texture: Texture::Strata,
    },
    Material {
        albedo: V::new(0.88, 0.76, 0.60),
        specular: 0.10,
        shininess: 24,
        transparency: 0.,
        reflectivity: 0.,
        ior: 1.,
        roughness: 0.13,
        texture: Texture::Grain,
    },
    Material {
        albedo: V::new(0.90, 0.96, 0.86),
        specular: 0.18,
        shininess: 38,
        transparency: 0.,
        reflectivity: 0.,
        ior: 1.,
        roughness: 0.06,
        texture: Texture::Cells,
    },
    Material {
        albedo: V::new(0.025, 0.22, 0.33),
        specular: 0.70,
        shininess: 100,
        transparency: 0.88,
        reflectivity: 0.07,
        ior: 1.333,
        roughness: 0.018,
        texture: Texture::Ripples,
    },
    Material {
        albedo: V::new(0.78, 0.92, 0.98),
        specular: 0.92,
        shininess: 160,
        transparency: 0.95,
        reflectivity: 0.045,
        ior: 1.52,
        roughness: 0.003,
        texture: Texture::Inclusions,
    },
];
impl MaterialId {
    pub fn get(self) -> &'static Material {
        &MATERIALS[self as usize]
    }
}
pub fn tint(a: V, b: V) -> V {
    V::new(a.x * b.x, a.y * b.y, a.z * b.z)
}
impl Material {
    pub fn sample(self, p: V, time: f32) -> V {
        use crate::render::noise;
        let factor = match self.texture {
            Texture::Strata => {
                0.83 + noise(p * 2.8) * 0.20 + (p.y * 2.1 + noise(p * 0.3) * 3.).sin() * 0.055
            }
            Texture::Grain => {
                0.84 + (p.x * 14. + p.z * 17. + noise(p * 1.8) * 5.).sin() * 0.10
                    + noise(p * 0.6) * 0.15
            }
            Texture::Cells => 0.80 + noise(p * 3.7) * 0.16 + noise(p * 18.) * 0.17,
            Texture::Ripples => {
                0.93 + (p.x * 1.8 + p.z * 0.9 + time * 0.6).sin()
                    * (p.z * 2.7 - time * 0.4).sin()
                    * 0.07
            }
            Texture::Inclusions => {
                0.94 + noise(p * 5.7) * 0.06
                    - (p.x * 2.4 + p.y * 3.1 + p.z * 0.7).sin().abs().powi(28) * 0.07
            }
        };
        self.albedo * factor
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn five_distinct_textures_and_valid_optical_parameters() {
        assert_eq!(MATERIALS.len(), 5);
        for (i, m) in MATERIALS.iter().enumerate() {
            assert!(MATERIALS[..i]
                .iter()
                .all(|other| other.texture != m.texture));
            for x in [
                m.albedo.x,
                m.albedo.y,
                m.albedo.z,
                m.specular,
                m.transparency,
                m.reflectivity,
            ] {
                assert!((0.0..=1.0).contains(&x));
            }
            assert!(m.ior >= 1. && m.shininess > 0);
        }
    }
}
