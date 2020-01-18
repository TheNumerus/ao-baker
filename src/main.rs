#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::window::WindowBuilder;
use glium::uniforms::UniformValue;
use glium::{implement_vertex, Surface, uniform, Display, Program};

use cgmath::prelude::*;
use cgmath::{Vector3, vec3, perspective, Matrix4, Deg, Point3, Quaternion, Matrix3};

use wavefront_obj::obj;
use wavefront_obj::obj::{Primitive, Object};

use std::fs::File;
use std::io::{Read, BufReader};
use std::time::{Instant};
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};

use rand::{thread_rng, Rng, prelude::*};

#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3]
}

implement_vertex!(Vertex, pos, color, normal);

const CENTER: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
const CAMERA_DIST: f32 = 3.0;
const UP_VECTOR: Vector3<f32> = vec3(0.0, 1.0, 0.0);
const ANGLE_SPREAD: f32 = 178.0;

const SIZE_X: u32 = 1280;
const SIZE_Y: u32 = 720;
const ASPECT_RATIO: f32 = 1280.0_f32 / 720.0;

const SAMPLES: u32 = 8;

struct VertexData {
    data: Vec<Vertex>,
    should_update: bool
}

impl VertexData {
    fn update(&mut self, data: Vec<Vertex>) {
        self.data = data;
        self.should_update = true;
    }
}

struct WorldData {
    circle: f32,
    camera_distance: f32,
    world_mat: Matrix4<f32>,
    shading_enabled: bool,
    is_paused: bool,
    ao_enabled: bool
}

impl WorldData {
    fn rotate_delta(&mut self, delta: f32) {
        if self.is_paused {
            return;
        }
        self.circle += delta;
        let x = self.camera_distance * self.circle.sin();
        let z = self.camera_distance * self.circle.cos();
        self.world_mat = Matrix4::look_at(Point3::new(x, 0.0, z), CENTER, UP_VECTOR);
    }
}

impl Default for WorldData {
    fn default() -> Self {
        Self {
            circle: 0.0,
            camera_distance: CAMERA_DIST,
            world_mat: Matrix4::look_at(Point3::new(0.0, 0.0, CAMERA_DIST), CENTER, UP_VECTOR),
            shading_enabled: true,
            is_paused: false,
            ao_enabled: true
        }
    }
}

fn make_display() -> (EventLoop<()>, Display) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("AO Baker".to_string())
        .with_inner_size((SIZE_X, SIZE_Y).into()).with_min_inner_size((400, 400).into());

    let cb = glium::glutin::ContextBuilder::new().with_depth_buffer(16).with_srgb(false);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    (event_loop, display)
}

fn make_program(display: &Display) -> Program {
    let vert = include_str!("test.vert");
    let frag = include_str!("test.frag");

    glium::Program::from_source(display, vert, frag, None).unwrap()
}

fn main() {
    let (event_loop, display) = make_display();
    let program = make_program(&display);

    let mut vertex_buffer = glium::VertexBuffer::new(&display, &[]).unwrap();

    let vertex_data = Arc::new(Mutex::new(VertexData{data: Vec::new(), should_update: false}));

    let transform = Matrix4::from_scale(1.0_f32);

    let mut wd = WorldData::default();

    let mut view = perspective(Deg(60.0), ASPECT_RATIO, 0.01_f32, 100.0_f32);

    let draw_param = glium::DrawParameters{
        depth: glium::Depth{
            test: glium::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    let mut is_focused = false;
    let mut time_earlier = Instant::now();
    let mut time = Instant::now();

    event_loop.run(move |event, _wt, control_flow | {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let delta = time.duration_since(time_earlier);
                time_earlier = time;
                time = Instant::now();
                wd.rotate_delta(delta.as_secs_f32());
                if vertex_data.lock().unwrap().should_update {
                    vertex_buffer = glium::VertexBuffer::new(&display, &vertex_data.lock().unwrap().data).unwrap();
                }
                let mut target = display.draw();
                target.clear_color(0.02, 0.02, 0.02, 1.0);
                target.clear_depth(1.0);
                let uniforms = uniform!(
                    model: Matrix4Wrapper(transform),
                    view: Matrix4Wrapper(view),
                    world: Matrix4Wrapper(wd.world_mat),
                    light: wd.shading_enabled,
                    ao: wd.ao_enabled
                );
                target.draw(&vertex_buffer, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program, &uniforms, &draw_param).unwrap();
                target.finish().unwrap();
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let aspect_ratio = new_size.width / new_size.height;
                view = perspective( Deg(60.0), aspect_ratio as f32, 0.01_f32, 100.0_f32);
            },
            Event::WindowEvent {
                event: WindowEvent::DroppedFile(file_path),
                ..
            } => {
                file_dropped(file_path, &display, Arc::clone(&vertex_data));
            },
            Event::WindowEvent {
                event: WindowEvent::Focused(focus),
                ..
            } => {
                is_focused = focus;
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(ch),
                ..
            } => {
                match ch {
                    'd' => wd.shading_enabled = !wd.shading_enabled,
                    'p' => wd.is_paused = !wd.is_paused,
                    'f' => wd.ao_enabled = !wd.ao_enabled,
                    _ => {}
                }
            },
            _ => {
                if is_focused {
                    *control_flow = ControlFlow::Poll;
                    display.gl_window().window().request_redraw();
                } else {
                    *control_flow = ControlFlow::Wait;
                }
            },
        }
    });
}

fn ray_triangle_intersect(orig: Vector3<f32>, dir: Vector3<f32>, vertices: [Vector3<f32>; 3]) -> bool {
    let v0v1 = vertices[1] - vertices[0];
    let v0v2 = vertices[2] - vertices[0];
    let pvec = dir.cross(v0v2);
    let det = v0v1.dot(pvec);
    if det < std::f32::EPSILON {
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

struct Matrix4Wrapper(cgmath::Matrix4<f32>);

impl glium::uniforms::AsUniformValue for Matrix4Wrapper {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4(self.0.into())
    }
}

fn file_dropped(file_path: PathBuf, display: &glium::Display, vertex_data: Arc<Mutex<VertexData>>) {
    if let Some(ext) = file_path.extension() {
        if ext == "obj" {
            println!("open {:?}", file_path.file_name());
            let obj = read_obj(file_path);
            let mut verts = Vec::with_capacity(obj.geometry[0].shapes.len() * 3);
            for shape in &obj.geometry[0].shapes {
                if let Primitive::Triangle(a, b, c) = shape.primitive {
                    for index in &[a, b, c] {
                        let vert_a = obj.vertices[index.0];
                        let norm_a = obj.normals[index.2.unwrap()];
                        let vert = Vertex{
                            color: [1.0; 3],
                            pos: [vert_a.x as f32, vert_a.y as f32, vert_a.z as f32],
                            normal: [norm_a.x as f32, norm_a.y as f32, norm_a.z as f32]
                        };
                        verts.push(vert);
                    }
                }
            }
            vertex_data.lock().unwrap().update(verts.to_owned());
            display.gl_window().window().request_redraw();
            thread::spawn(move || {
                let time = Instant::now();
                let mut rng = thread_rng();
                let spread = ANGLE_SPREAD / 180.0 * std::f32::consts::PI;
                for vert in &mut verts {
                    let mut hits = 0;
                    for _sample in 0..SAMPLES {
                        let line = get_random_ray(spread, vert.normal.into(), &mut rng);
                        for shape in &obj.geometry[0].shapes {
                            let shape = if let Primitive::Triangle(a, b, c) = shape.primitive {
                                let v0 = vec3(obj.vertices[a.0].x as f32, obj.vertices[a.0].y as f32, obj.vertices[a.0].z as f32);
                                let v1 = vec3(obj.vertices[b.0].x as f32, obj.vertices[b.0].y as f32, obj.vertices[b.0].z as f32);
                                let v2 = vec3(obj.vertices[c.0].x as f32, obj.vertices[c.0].y as f32, obj.vertices[c.0].z as f32);
                                [v0, v1, v2]
                            } else {
                                continue
                            };

                            let offset = Vector3::from(vert.normal) * 0.03;

                            let is_hit = ray_triangle_intersect(vec3(vert.pos[0], vert.pos[1], vert.pos[2]) + offset, line, shape);

                            if is_hit {
                                hits += 1;
                                break;
                            }
                        }
                    }
                    if hits != 0 {
                        let color = 1.0 - (hits as f32 / SAMPLES as f32);
                        vert.color = [color; 3];
                        continue;
                    }
                }
                vertex_data.lock().unwrap().update(verts);
                let time = time.elapsed();
                println!("comp finished in {} secs", time.as_secs_f64());
            });
        }
    }
}

fn read_obj(filename: PathBuf) -> Object {
    let file = File::open(filename).unwrap();
    let mut file_content = String::new();
    let mut reader = BufReader::new(file);
    reader.read_to_string(&mut file_content).unwrap();
    obj::parse(file_content).unwrap().objects[0].to_owned()
}

fn get_random_ray(angle_spread: f32, dir: Vector3<f32>, rng: &mut ThreadRng) -> Vector3<f32> {
    debug_assert!(angle_spread > 0.0);
    debug_assert!(angle_spread < std::f32::consts::PI);

    let angle = rng.gen_range((angle_spread / 2.0).cos(), 1.0);
    let rot = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
    let one_minus_z = (1.0 - (angle).powi(2)).sqrt();
    let vec = vec3(one_minus_z * rot.cos(), one_minus_z * rot.sin(), angle);

    let q = Quaternion::from_arc(vec3(0.0, 0.0, 1.0), dir, None);
    let mat = Matrix3::from(q);
    mat * vec
}