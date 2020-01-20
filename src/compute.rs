use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::thread;

use crate::render::VertexData;
use crate::geo::Vertex;
use crate::consts::*;
use crate::math::*;

use wavefront_obj::obj::{Object, Primitive};

use rand::thread_rng;

use cgmath::{Vector3, Quaternion, vec3, Matrix3};

pub fn compute_ao(vertex_data: Arc<Mutex<VertexData>>, obj: Object, mut verts: Vec<Vertex>, bake_in_progress: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        let time = Instant::now();
        let mut rng = thread_rng();
        let spread = ANGLE_SPREAD / 180.0 * std::f32::consts::PI;

        for sample in 0..SAMPLES {
            let sample_time = Instant::now();
            let line = get_random_ray(spread, &mut rng);
            for vert in &mut verts {
                let mut is_hit = false;

                let offset = Vector3::from(vert.normal) * 0.001;
                let orig = Vector3::from(vert.pos) + offset;
                let q = Quaternion::from_arc(vec3(0.0, 0.0, 1.0), vert.normal.into(), None);
                let mat = Matrix3::from(q);

                let line = mat * line;
                for shape in &obj.geometry[0].shapes {
                    let shape = if let Primitive::Triangle(a, b, c) = shape.primitive {
                        let v0 = vec3(obj.vertices[a.0].x as f32, obj.vertices[a.0].y as f32, obj.vertices[a.0].z as f32);
                        let v1 = vec3(obj.vertices[b.0].x as f32, obj.vertices[b.0].y as f32, obj.vertices[b.0].z as f32);
                        let v2 = vec3(obj.vertices[c.0].x as f32, obj.vertices[c.0].y as f32, obj.vertices[c.0].z as f32);
                        [v0, v1, v2]
                    } else {
                        continue
                    };

                    is_hit = ray_triangle_intersect(orig, line, shape);

                    if is_hit {
                        break;
                    }
                }
                let old_color = vert.color[0];
                let new_color = if is_hit {
                    (old_color * sample as f32) / (sample as f32 + 1.0)
                } else {
                    (old_color * sample as f32 + 1.0) / (sample as f32 + 1.0)
                };
                vert.color = [new_color; 3];
            }
            let sample_time = sample_time.elapsed().as_secs_f64();
            let rays = verts.len() as f64;
            println!("{:.3} krays/s,  ETA: {:.1} secs", rays / sample_time / 1_000.0, sample_time * (SAMPLES - sample) as f64);
            vertex_data.lock().unwrap().update(verts.to_owned());
        }
        let time = time.elapsed();
        println!("comp finished in {} secs", time.as_secs_f64());
        *bake_in_progress.lock().unwrap() = false;
    });
}
