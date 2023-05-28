use crate::V2;
fn cross(a: V2, b: V2) -> f64 {
    a.x * b.y - a.y * b.x
}

fn orient(a: V2, b: V2, c: V2) -> f64 {
    cross(b - a, c - a)
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
