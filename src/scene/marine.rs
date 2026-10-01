use super::*;
pub(super) fn coast(s: &mut Scene) {
    for x in -60..=60 {
        for z in 49..=94 {
            let (x, z) = (x as f32, z as f32);
            let h = seabed(x, z);
            let sand = V::new(0.65, 0.56, 0.36);
            s.cube(
                V::new(x, (h - 10.) * 0.5, z),
                V::new(1.001, h + 10., 1.001),
                sand,
                MaterialId::Earth,
                Tag::None,
            );
        }
    }
}
pub fn seabed(x: f32, z: f32) -> f32 {
    let coast = 55. + (x * 0.10).sin() * 1.4;
    if z < coast {
        ground(x, 48.) * ((coast - z) / (coast - 48.)).clamp(0., 1.)
    } else {
        (-0.6 * (z - coast)).max(-7.)
    }
}
fn model(s: &mut Scene, p: V, tag: Tag, c: V) -> Model<'_> {
    Model {
        scene: s,
        origin: p,
        scale: 1.,
        reverse: false,
        tag,
        color: c,
    }
}
fn reptile(s: &mut Scene, p: V, long_neck: bool) {
    let start = s.cubes.len();
    let c = if long_neck {
        V::new(0.31, 0.40, 0.36)
    } else {
        V::new(0.19, 0.30, 0.32)
    };
    let mut m = model(
        s,
        p,
        if long_neck {
            Tag::Plesiosaur
        } else {
            Tag::Mosasaur
        },
        c,
    );
    m.ellipsoid(V::default(), V::new(3.1, 0.85, 1.1), c);
    m.ellipsoid(V::new(0., -0.43, 0.), V::new(2.7, 0.48, 0.92), c * 1.45);
    for x in [-1.8, 1.5] {
        for side in [-1., 1.] {
            m.segment(
                V::new(x, -0.2, side * 0.7),
                V::new(x - 1.0, -0.45, side * 2.8),
                0.27,
                c,
            );
            m.ellipsoid(
                V::new(x - 0.65, -0.38, side * 1.9),
                V::new(0.8, 0.16, 0.8),
                c,
            );
        }
    }
    if long_neck {
        for i in 0..20 {
            let t = i as f32 / 19.;
            m.ellipsoid(
                V::new(2.3 + t * 4.9, t * 0.75, 0.),
                V::new(0.35, 0.37 - t * 0.17, 0.37 - t * 0.17),
                c,
            );
        }
        m.ellipsoid(V::new(7.4, 0.75, 0.), V::new(0.7, 0.31, 0.34), c);
        m.eyes(7.55, 0.85, 0.30);
        m.tail(V::new(-2.6, 0., 0.), 2., 0.48);
    } else {
        m.ellipsoid(V::new(3.0, 0., 0.), V::new(1.5, 0.60, 0.65), c);
        m.ellipsoid(V::new(4.15, -0.13, 0.), V::new(1.3, 0.32, 0.43), c);
        m.eyes(3.7, 0.32, 0.57);
        for i in 0..8 {
            for side in [-1., 1.] {
                m.block(
                    V::new(3.4 + i as f32 * 0.22, -0.30, side * 0.39),
                    0.11,
                    V::new(0.8, 0.78, 0.62),
                );
            }
        }
        m.tail(V::new(-2.5, 0., 0.), 5., 0.75);
        m.segment(V::new(-7., -0.3, 0.), V::new(-7.8, 1.2, 0.), 0.30, c);
        m.segment(V::new(-7., -0.3, 0.), V::new(-7.6, -1.3, 0.), 0.25, c);
    }
    s.register_animal(start);
}
fn cephalopod(s: &mut Scene, p: V, nautiloid: bool) {
    let start = s.cubes.len();
    let c = if nautiloid {
        V::new(0.69, 0.57, 0.40)
    } else {
        V::new(0.61, 0.35, 0.18)
    };
    let mut m = model(
        s,
        p,
        if nautiloid {
            Tag::Nautiloid
        } else {
            Tag::Ammonite
        },
        c,
    );
    // A continuous expanding spiral, with a narrow central umbilicus.
    for i in 0..100 {
        let t = i as f32 / 99.;
        let a = t * std::f32::consts::TAU * 2.2;
        let r = 0.08 + t * 0.80;
        let stripe = if i % 7 < 2 { 0.68 } else { 1. };
        m.ellipsoid(
            V::new(r * a.cos(), r * a.sin() + 0.95, 0.),
            V::new(0.09 + t * 0.12, 0.09 + t * 0.12, 0.12 + t * 0.17),
            c * stripe,
        );
    }
    let flesh = V::new(0.47, 0.29, 0.18);
    m.ellipsoid(V::new(0.60, 0.45, 0.), V::new(0.35, 0.20, 0.24), flesh);
    for i in 0..6 {
        let side = (i as f32 - 2.5) * 0.12;
        m.segment(
            V::new(0.7, 0.43, side),
            V::new(1.45, 0.2 + side.abs(), side * 2.),
            0.055,
            flesh,
        );
    }
    m.block(V::new(0.77, 0.55, 0.23), 0.08, V::new(0.02, 0.025, 0.015));
    s.register_animal(start);
}
fn horseshoe(s: &mut Scene, x: f32, z: f32) {
    let start = s.cubes.len();
    let c = V::new(0.29, 0.24, 0.16);
    let mut m = model(s, V::new(x, seabed(x, z) + 0.10, z), Tag::HorseshoeCrab, c);
    m.ellipsoid(V::new(0.15, 0.16, 0.), V::new(0.62, 0.22, 0.67), c);
    m.ellipsoid(V::new(-0.47, 0.12, 0.), V::new(0.38, 0.16, 0.44), c * 0.88);
    m.segment(
        V::new(-0.70, 0.10, 0.),
        V::new(-1.75, 0.07, 0.),
        0.045,
        c * 0.7,
    );
    for side in [-1., 1.] {
        for i in 0..5 {
            let x = -0.45 + i as f32 * 0.18;
            m.segment(
                V::new(x, 0.04, side * 0.22),
                V::new(x - 0.13, 0.0, side * 0.62),
                0.04,
                c * 0.65,
            );
        }
        m.block(
            V::new(0.40, 0.32, side * 0.39),
            0.065,
            V::new(0.035, 0.03, 0.02),
        );
    }
    s.register_animal(start);
}
fn bivalve(s: &mut Scene, x: f32, z: f32) {
    let start = s.cubes.len();
    let c = V::new(0.62, 0.55, 0.42);
    let mut m = model(s, V::new(x, seabed(x, z) + 0.06, z), Tag::Bivalve, c);
    m.ellipsoid(V::new(0., 0.13, 0.), V::new(0.43, 0.12, 0.32), c * 0.75);
    m.ellipsoid(V::new(0., 0.30, 0.), V::new(0.43, 0.14, 0.32), c);
    for i in -3..=3 {
        m.segment(
            V::new(-0.28, 0.35, 0.),
            V::new(0.25, 0.36, i as f32 * 0.07),
            0.025,
            c * 0.8,
        );
    }
    s.register_animal(start);
}
pub(super) fn fauna(s: &mut Scene) {
    reptile(s, V::new(-20., -2.2, 75.), false);
    reptile(s, V::new(21., -2.5, 82.), true);
    for (x, z) in [(-6., 70.), (3., 77.), (32., 70.), (-34., 82.)] {
        cephalopod(s, V::new(x, -3., z), false);
    }
    for (x, z) in [(-4., 64.), (12., 72.), (35., 85.)] {
        cephalopod(s, V::new(x, -2.4, z), true);
    }
    for (x, z) in [(-7., 57.), (-12., 58.), (1., 58.), (21., 59.)] {
        horseshoe(s, x, z);
    }
    for (x, z) in [
        (-9., 58.),
        (-5., 59.),
        (2., 60.),
        (10., 61.),
        (23., 60.),
        (31., 62.),
    ] {
        bivalve(s, x, z);
    }
}
