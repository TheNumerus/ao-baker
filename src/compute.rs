use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use std::thread;
use std::ops::{IndexMut, Index};
use std::io::Write;

use crate::render::VertexData;
use crate::geo::Vertex;
use crate::consts::*;

use wavefront_obj::obj::{Object, Primitive, Shape, Vertex as ObjVertex};

use rand::thread_rng;
use rand::prelude::*;

use cgmath::{Vector3, Quaternion, vec3, Matrix3, prelude::*};

pub fn compute_ao(vertex_data: Arc<Mutex<VertexData>>, obj: Object, mut verts: Vec<Vertex>, bake_in_progress: Arc<AtomicBool>, compute_data: &ComputeData, bake_stopper: Arc<AtomicBool>) {
    let compute_data = compute_data.clone();
    thread::spawn(move || {
        //let time = Instant::now();
        let mut time = 0.0_f64;
        let mut rng = thread_rng();
        let spread = ANGLE_SPREAD / 180.0 * std::f32::consts::PI;

        let mut stdout =  std::io::stdout();

        let grid = AABBGrid::new(&verts, &obj.geometry[0].shapes, &obj.vertices);

        for sample in 0..compute_data.samples {
            if bake_stopper.load(Ordering::SeqCst) {
                bake_stopper.store(false, Ordering::SeqCst);
                print!("\nBake aborted after {} samples", sample);
                break;
            }
            let sample_time = Instant::now();
            let line = get_random_ray(spread, &mut rng);
            for vert in &mut verts {
                let mut is_hit = false;

                let offset = Vector3::from(vert.normal) * 0.001;
                let orig = Vector3::from(vert.pos) + offset;
                let q = Quaternion::from_arc(vec3(0.0, 0.0, 1.0), vert.normal.into(), None);
                let mat = Matrix3::from(q);

                let line = mat * line;
                let cells = grid.traverse(&orig, &line);
                'cells: for cell in &cells {
                    let cell = &grid[*cell];
                    let cell = match cell {
                        Some(val) => val,
                        None => continue
                    };
                    for shape in cell {
                        let shape = &obj.geometry[0].shapes[*shape];
                        let shape = if let Primitive::Triangle(a, b, c) = shape.primitive {
                            let v0 = vec3(obj.vertices[a.0].x as f32, obj.vertices[a.0].y as f32, obj.vertices[a.0].z as f32);
                            let v1 = vec3(obj.vertices[b.0].x as f32, obj.vertices[b.0].y as f32, obj.vertices[b.0].z as f32);
                            let v2 = vec3(obj.vertices[c.0].x as f32, obj.vertices[c.0].y as f32, obj.vertices[c.0].z as f32);
                            [v0, v1, v2]
                        } else {
                            continue
                        };

                        is_hit = ray_triangle_intersect(orig, line, shape).is_some();

                        if is_hit {
                            break 'cells;
                        }
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
            time += sample_time.elapsed().as_secs_f64();

            let rays =  (((sample + 1) as usize) * verts.len()) as f64;

            print!("\x1B[2KAverage {:.3} krays/s,  ETA: {:.1} secs\r", rays / time / 1_000.0, time * (compute_data.samples as f64 / sample as f64) - time);
            stdout.flush().unwrap();
            let mut lock = vertex_data.lock().unwrap();
            lock.update(verts.to_owned());
        }
        println!("\ncomp finished in {:.3} secs", time);
        vertex_data.lock().unwrap().update(verts.to_owned());
        bake_in_progress.store(false, Ordering::SeqCst);
    });
}

fn find_extrema(verts: &[Vertex]) -> [f32; 6] {
    let mut min_x = std::f32::MAX;
    let mut min_y = std::f32::MAX;
    let mut min_z = std::f32::MAX;

    let mut max_x = std::f32::MIN;
    let mut max_y = std::f32::MIN;
    let mut max_z = std::f32::MIN;

    for vert in verts {
        if vert.pos[0] < min_x {
            min_x = vert.pos[0];
        }
        if vert.pos[1] < min_y {
            min_y = vert.pos[1];
        }
        if vert.pos[2] < min_z {
            min_z = vert.pos[2];
        }

        if vert.pos[0] > max_x {
            max_x = vert.pos[0];
        }
        if vert.pos[1] > max_y {
            max_y = vert.pos[1];
        }
        if vert.pos[2] > max_z {
            max_z = vert.pos[2];
        }
    }
    [min_x, min_y, min_z, max_x, max_y, max_z]
}

fn find_extrema_triangle(a: &ObjVertex, b: &ObjVertex, c: &ObjVertex) -> [f32; 6] {
    let mut min_x = a.x;
    let mut min_y = a.y;
    let mut min_z = a.z;

    let mut max_x = a.x;
    let mut max_y = a.y;
    let mut max_z = a.z;

    for vert in &[b, c] {
        if vert.x < min_x {
            min_x = vert.x;
        }
        if vert.y < min_y {
            min_y = vert.y;
        }
        if vert.z < min_z {
            min_z = vert.z;
        }

        if vert.x > max_x {
            max_x = vert.x;
        }
        if vert.y > max_y {
            max_y = vert.y;
        }
        if vert.z > max_z {
            max_z = vert.z;
        }
    }
    [min_x as f32, min_y as f32, min_z as f32, max_x as f32, max_y as f32, max_z as f32]
}

fn map_pos_to_grid(pos: f32, divs: usize, min: f32, max: f32) -> usize {
    if pos >= max {
        divs - 1
    } else if pos <= min {
        0
    } else {
        let delta = max - min;
        let index = ((pos - min) * (divs as f32 / delta)).floor();
        index as usize
    }
}

#[derive(Clone, Debug)]
struct AABBGrid {
    bounds: BoundBox,
    x_divs: usize,
    y_divs: usize,
    z_divs: usize,
    grid: Vec<Option<Vec<usize>>>,
    max_dist: f32
}

impl AABBGrid {
    fn new(verts: &[Vertex], shapes: &[Shape], obj_verts: &[ObjVertex]) -> Self {
        let time = Instant::now();

        let extrema = find_extrema(verts);

        let size_x = extrema[3] - extrema[0];
        let size_y = extrema[4] - extrema[1];
        let size_z = extrema[5] - extrema[2];

        let sizes = [size_x, size_y, size_z];

        let mut dim = [1; 3];
        let cube_root = (shapes.len() as f32 * 4.0 / (size_x * size_y * size_z)).powf(1.0 / 3.0);
        for i in 0..3 {
            dim[i] = (cube_root * sizes[i]).floor() as usize;
            if dim[i] > 128 {
                dim[i] = 128;
            } else if dim[i] < 1 {
                dim[i] = 1;
            }
        }

        let max_dist = ((size_x.powi(2) + size_y.powi(2)).sqrt() + size_z.powi(2)).sqrt() + 1.0;

        let grid = vec![None; dim[0] * dim[1] * dim[2]];

        let mut aabb_grid = AABBGrid {
            bounds: BoundBox {
                min: [extrema[0], extrema[1], extrema[2]],
                max: [extrema[3], extrema[4], extrema[5]]
            },
            x_divs: dim[0],
            y_divs: dim[1],
            z_divs: dim[2],
            grid,
            max_dist
        };

        for (index, shape) in shapes.iter().enumerate() {
            if let Primitive::Triangle(a, b, c) = shape.primitive {
                let vert_extrema = find_extrema_triangle(&obj_verts[a.0], &obj_verts[b.0], &obj_verts[c.0]);
                let min_index_x = map_pos_to_grid(vert_extrema[0], dim[0], extrema[0], extrema[3]);
                let min_index_y = map_pos_to_grid(vert_extrema[1], dim[1], extrema[1], extrema[4]);
                let min_index_z = map_pos_to_grid(vert_extrema[2], dim[2], extrema[2], extrema[5]);
                let max_index_x = map_pos_to_grid(vert_extrema[3], dim[0], extrema[0], extrema[3]);
                let max_index_y = map_pos_to_grid(vert_extrema[4], dim[1], extrema[1], extrema[4]);
                let max_index_z = map_pos_to_grid(vert_extrema[5], dim[2], extrema[2], extrema[5]);
                for x in min_index_x..=max_index_x {
                    for y in min_index_y..=max_index_y {
                        for z in min_index_z..=max_index_z {
                            match &mut aabb_grid[(x, y, z)] {
                                Some(vec) => {
                                    vec.push(index);
                                },
                                None => {
                                    aabb_grid[(x, y, z)] = Some(vec![index]);
                                }
                            }
                        }
                    }
                }
            }
        }

        let time = time.elapsed().as_secs_f64();
        println!("precompute took {:.03} secs", time);

        aabb_grid
    }

    fn traverse(&self, origin: &Vector3<f32>, dir: &Vector3<f32>) -> Vec<(usize, usize, usize)> {
        let mut vec = Vec::with_capacity(self.x_divs + self.y_divs + self.z_divs);


        let inv_dir: Vector3<f32> = 1.0 / dir;
        let sign = [dir.x > 0.0, dir.y > 0.0, dir.z > 0.0];

        let t_hit = match self.bounds.intersect(&origin, &inv_dir, &sign) {
            Some(v) => v,
            None => return vec
        };

        let origin = [origin.x, origin.y, origin.z];
        let inv_dir = [inv_dir.x, inv_dir.y, inv_dir.z];
        let mut deltas = [0.0; 3];
        let mut next_cross = [0.0; 3];

        let cell_sizes = [
            (self.bounds.max[0] - self.bounds.min[0]) / self.x_divs as f32,
            (self.bounds.max[1] - self.bounds.min[1]) / self.y_divs as f32,
            (self.bounds.max[2] - self.bounds.min[2]) / self.z_divs as f32
        ];

        let res = [self.x_divs as i32, self.y_divs as i32, self.z_divs as i32];

        let mut exit = [0; 3];
        let mut cell = [0; 3];
        let mut step = [0; 3];

        for i in 0..3 {
            let ray_orig_cell = (origin[i] + dir[i] * t_hit) - self.bounds.min[i];
            cell[i] = (ray_orig_cell / cell_sizes[i]).floor().max(0.0).min(res[i] as f32 - 1.0) as i32;
            if sign[i] {
                deltas[i] = cell_sizes[i] * inv_dir[i];
                next_cross[i] = t_hit + ((cell[i] + 1) as f32 * cell_sizes[i] - ray_orig_cell) * inv_dir[i];
                exit[i] = res[i];
                step[i] = 1;
            } else {
                deltas[i] = -cell_sizes[i] * inv_dir[i];
                next_cross[i] = t_hit + (cell[i] as f32 * cell_sizes[i] - ray_orig_cell) * inv_dir[i];
                exit[i] = -1;
                step[i] = -1;
            }
        }

        loop {
            vec.push((cell[0] as usize, cell[1] as usize, cell[2] as usize));
            let k = (((next_cross[0] < next_cross[1]) as usize) << 2) +
                    (((next_cross[0] < next_cross[2]) as usize) << 1) +
                    ((next_cross[1] < next_cross[2]) as usize);

            let axis = MAP[k];

            if self.max_dist < next_cross[axis] {
                break
            }

            cell[axis] += step[axis];

            if cell[axis] == exit[axis] {
                break
            }
            next_cross[axis] += deltas[axis];
        }

        vec
    }
}

impl Index<(usize, usize, usize)> for AABBGrid {
    type Output = Option<Vec<usize>>;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let (x, y, z) = index;
        let index = x + y * self.x_divs + z * self.x_divs * self.y_divs;
        &self.grid[index]
    }
}

impl IndexMut<(usize, usize, usize)> for AABBGrid {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        let (x, y, z) = index;
        let index = x + y * self.x_divs + z * self.x_divs * self.y_divs;
        &mut self.grid[index]
    }
}

pub fn get_random_ray(angle_spread: f32, rng: &mut ThreadRng) -> Vector3<f32> {
    debug_assert!(angle_spread > 0.0);
    debug_assert!(angle_spread < std::f32::consts::PI);

    let angle = rng.gen_range((angle_spread / 2.0).cos(), 1.0);
    let rot = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
    let one_minus_z = (1.0 - (angle).powi(2)).sqrt();
    vec3(one_minus_z * rot.cos(), one_minus_z * rot.sin(), angle)
}

pub fn ray_triangle_intersect(orig: Vector3<f32>, dir: Vector3<f32>, vertices: [Vector3<f32>; 3]) -> Option<f32> {
    let v0v1 = vertices[1] - vertices[0];
    let v0v2 = vertices[2] - vertices[0];
    let pvec = dir.cross(v0v2);
    let det = v0v1.dot(pvec);
    if det.abs() < std::f32::EPSILON {
        return None;
    }
    let inv_det = 1.0 / det;

    let tvec = orig - vertices[0];
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = tvec.cross(v0v1);
    let v = dir.dot(qvec) * inv_det;

    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = v0v2.dot(qvec) * inv_det;

    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}

#[derive(Copy, Clone, Debug)]
struct BoundBox {
    min: [f32; 3],
    max: [f32; 3]
}

impl BoundBox {
    pub fn intersect(&self, orig: &Vector3<f32>, inv_dir: &Vector3<f32>, sign: &[bool; 3]) -> Option<f32> {
        let sign = [sign[0] as usize, sign[1] as usize, sign[2] as usize];
        let bounds = [self.min, self.max];

        let mut tmin  = (bounds[1 - sign[0]][0] - orig.x) * inv_dir.x;
        let mut tmax  = (bounds[sign[0]][0] - orig.x) * inv_dir.x;
        let tymin = (bounds[1 - sign[1]][1] - orig.y) * inv_dir.y;
        let tymax = (bounds[sign[1]][1] - orig.y) * inv_dir.y;

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let tzmin = (bounds[1 - sign[2]][2] - orig.z) * inv_dir.z;
        let tzmax = (bounds[sign[2]][2] - orig.z) * inv_dir.z;

        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        Some(tmin)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ComputeData {
    max_ray_dist: f32,
    samples: u32
}

impl Default for ComputeData {
    fn default() -> Self {
        ComputeData{
            max_ray_dist: std::f32::MAX,
            samples: SAMPLES
        }
    }
}
