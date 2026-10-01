use std::ops::{Add, Div, Mul, Neg, Sub};
#[derive(Clone, Copy, Debug, Default)]
pub struct V {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl V {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn norm(self) -> Self {
        self / self.length().max(1e-8)
    }
    pub fn at(self, i: usize) -> f32 {
        [self.x, self.y, self.z][i]
    }
    pub fn min(self, b: Self) -> Self {
        Self::new(self.x.min(b.x), self.y.min(b.y), self.z.min(b.z))
    }
    pub fn max(self, b: Self) -> Self {
        Self::new(self.x.max(b.x), self.y.max(b.y), self.z.max(b.z))
    }
    pub fn mix(self, b: Self, t: f32) -> Self {
        self * (1. - t) + b * t
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, k: f32) -> Self {
        Self::new(self.x * k, self.y * k, self.z * k)
    }
}
impl Div<f32> for V {
    type Output = Self;
    fn div(self, k: f32) -> Self {
        self * (1. / k)
    }
}
impl Neg for V {
    type Output = Self;
    fn neg(self) -> Self {
        self * (-1.)
    }
}
#[derive(Clone, Copy)]
pub struct Ray {
    pub o: V,
    pub d: V,
}
#[derive(Clone, Copy)]
pub struct Bounds {
    pub lo: V,
    pub hi: V,
}
impl Bounds {
    pub fn interval(self, r: Ray, limit: f32) -> Option<(f32, f32, V, V)> {
        let mut near = f32::NEG_INFINITY;
        let mut far = limit;
        let mut nn = V::default();
        let mut nf = V::default();
        for i in 0..3 {
            let o = r.o.at(i);
            let d = r.d.at(i);
            let lo = self.lo.at(i);
            let hi = self.hi.at(i);
            if d.abs() < 1e-8 {
                if o < lo || o > hi {
                    return None;
                }
                continue;
            }
            let mut a = (lo - o) / d;
            let mut b = (hi - o) / d;
            let mut n = match i {
                0 => V::new(-1., 0., 0.),
                1 => V::new(0., -1., 0.),
                _ => V::new(0., 0., -1.),
            };
            if a > b {
                std::mem::swap(&mut a, &mut b);
                n = -n;
            }
            if a > near {
                near = a;
                nn = n;
            }
            if b < far {
                far = b;
                nf = -n;
            }
            if far < near {
                return None;
            }
        }
        if far < 0.001 {
            None
        } else {
            Some((near, far, nn, nf))
        }
    }
    pub fn hit(self, r: Ray, limit: f32) -> Option<(f32, V)> {
        self.interval(r, limit).and_then(|(a, b, n, m)| {
            if a > 0.001 {
                Some((a, n))
            } else if b < limit {
                Some((b, m))
            } else {
                None
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn box_rays() {
        let b = Bounds {
            lo: V::new(-1., -1., -1.),
            hi: V::new(1., 1., 1.),
        };
        let (t, n) = b
            .hit(
                Ray {
                    o: V::new(0., 0., 3.),
                    d: V::new(0., 0., -1.),
                },
                100.,
            )
            .unwrap();
        assert_eq!(t, 2.);
        assert_eq!(n.z, 1.);
        assert!(b
            .hit(
                Ray {
                    o: V::new(2., 0., 3.),
                    d: V::new(0., 0., -1.)
                },
                100.
            )
            .is_none());
        assert_eq!(
            b.hit(
                Ray {
                    o: V::default(),
                    d: V::new(1., 0., 0.)
                },
                100.
            )
            .unwrap()
            .0,
            1.
        );
    }
}
