use glium::implement_vertex;

use wavefront_obj::obj::{Object, Primitive};

use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3]
}

implement_vertex!(Vertex, pos, color, normal);

pub fn generate_vector_buffer(obj: &Object) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts = Vec::with_capacity(obj.geometry[0].shapes.len() * 3);
    let mut indices_vec = Vec::new();
    let mut map = HashMap::new();
    let mut i = 0;
    for shape in &obj.geometry[0].shapes {
        if let Primitive::Triangle(a, b, c) = shape.primitive {
            for index in &[a, b, c] {
                if !map.contains_key(&(index.0, index.2.unwrap())) {
                    map.insert((index.0, index.2.unwrap()), i);
                    let vert_a = obj.vertices[index.0];
                    let norm_a = obj.normals[index.2.unwrap()];
                    let vert = Vertex {
                        color: [1.0; 3],
                        pos: [vert_a.x as f32, vert_a.y as f32, vert_a.z as f32],
                        normal: [norm_a.x as f32, norm_a.y as f32, norm_a.z as f32]
                    };
                    verts.push(vert);
                    indices_vec.push(i);
                    i += 1;
                } else {
                    indices_vec.push(map[&(index.0, index.2.unwrap())]);
                }
            }
        }
    }
    (verts, indices_vec)
}