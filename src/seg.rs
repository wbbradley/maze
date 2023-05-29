use crate::V2;
use std::f64::consts::TAU;

fn cross(a: V2, b: V2) -> f64 {
    a.x * b.y - a.y * b.x
}

fn orient(a: V2, b: V2, c: V2) -> f64 {
    cross(b - a, c - a)
}

fn shrink((a, b): (V2, V2), scale: f64) -> (V2, V2) {
    let mid = (a + b) * 0.5;
    (V2::lerp(mid, a, scale), V2::lerp(mid, b, scale))
}

pub(crate) fn intersection_with_width(
    a: V2,
    b: V2,
    c: V2,
    d: V2,
    width: f64,
    shrink_factor: f64,
) -> bool {
    let (a, b) = shrink((a, b), 0.9);
    let (a1, b1) = shrink(displace_by(a, b, TAU / 4.0, width), shrink_factor);
    let (a2, b2) = shrink(displace_by(a, b, -TAU / 4.0, width), shrink_factor);
    let (c1, d1) = shrink(displace_by(c, d, TAU / 4.0, width), shrink_factor);
    let (c2, d2) = shrink(displace_by(c, d, -TAU / 4.0, width), shrink_factor);
    let ab_combos = [(a, b), (a1, b1), (a2, b2)];
    let cd_combos = [(c, d), (c1, d1), (c2, d2)];
    for (a, b) in ab_combos {
        for (c, d) in cd_combos {
            if intersection(a, b, c, d) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn displace_by(a: V2, b: V2, radians: f64, offset: f64) -> (V2, V2) {
    let ab_norm = {
        let d = (b - a).normalise();
        let cr = radians.cos();
        let sr = radians.sin();
        V2 {
            x: d.x * cr - d.y * sr,
            y: d.x * sr + d.y * cr,
        } * offset
    };

    (a + ab_norm, b + ab_norm)
}

pub(crate) fn intersection(a: V2, b: V2, c: V2, d: V2) -> bool {
    let oa = orient(c, d, a);
    let ob = orient(c, d, b);
    let oc = orient(a, b, c);
    let od = orient(a, b, d);
    // Proper intersection exists iff opposite signs
    oa * ob < 0.0 && oc * od < 0.0
}

#[test]
fn test_intersection() {
    let a = V2 { x: 0.0, y: 0.0 };
    let b = V2 { x: 1.0, y: 0.0 };
    let c = V2 { x: 2.0, y: 0.0 };
    let d = V2 { x: 3.0, y: 0.0 };
    assert!(!intersection(a, b, c, d));
    let a = V2 { x: 0.0, y: 0.0 };
    let b = V2 { x: 1.0, y: 0.0 };
    let c = V2 { x: 0.5, y: 1.0 };
    let d = V2 { x: 0.5, y: -1.0 };
    assert!(intersection(a, b, c, d));
}
