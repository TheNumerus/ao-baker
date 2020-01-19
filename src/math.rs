use cgmath::{prelude::*, Vector3, vec3};
use rand::prelude::*;

pub fn get_random_ray(angle_spread: f32, rng: &mut ThreadRng) -> Vector3<f32> {
    debug_assert!(angle_spread > 0.0);
    debug_assert!(angle_spread < std::f32::consts::PI);

    let angle = rng.gen_range((angle_spread / 2.0).cos(), 1.0);
    let rot = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
    let one_minus_z = (1.0 - (angle).powi(2)).sqrt();
    let vec = vec3(one_minus_z * rot.cos(), one_minus_z * rot.sin(), angle);
    vec
}

pub fn ray_triangle_intersect(orig: Vector3<f32>, dir: Vector3<f32>, vertices: [Vector3<f32>; 3]) -> bool {
    let v0v1 = vertices[1] - vertices[0];
    let v0v2 = vertices[2] - vertices[0];
    let pvec = dir.cross(v0v2);
    let det = v0v1.dot(pvec);
    if det.abs() < std::f32::EPSILON {
        return false;
    }
    let inv_det = 1.0 / det;

    let tvec = orig - vertices[0];
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return false;
    }

    let qvec = tvec.cross(v0v1);
    let v = dir.dot(qvec) * inv_det;

    if v < 0.0 || (u + v) > 1.0 {
        return false;
    }

    let t = v0v2.dot(qvec) * inv_det;

    t >= 0.0
}
