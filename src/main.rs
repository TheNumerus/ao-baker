#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::window::WindowBuilder;
use glium::{implement_vertex, Surface, uniform};

use cgmath::prelude::*;
use cgmath::{Vector3, vec3, perspective, Matrix4, Deg, Point3};

use wavefront_obj::obj;
use wavefront_obj::obj::Primitive;
use std::fs::File;
use std::io::{Read, BufReader};
use glium::uniforms::UniformValue;
use std::time::Instant;
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3]
}

implement_vertex!(Vertex, pos, color, normal);

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [ -0.5, -0.5, 0.0 ], color: [1.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] },
    Vertex { pos: [  0.5, -0.5, 0.0 ], color: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0] },
    Vertex { pos: [  0.0,  0.5, 0.0 ], color: [0.0, 0.0, 1.0], normal: [0.0, 1.0, 0.0] }
];

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

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("AO Baker".to_string())
        .with_inner_size((1280, 720).into()).with_min_inner_size((400, 400).into());

    let cb = glium::glutin::ContextBuilder::new().with_depth_buffer(16).with_srgb(false);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vert = include_str!("test.vert");
    let frag = include_str!("test.frag");

    let program = glium::Program::from_source(&display, vert, frag, None).unwrap();

    let mut vertex_buffer = glium::VertexBuffer::new(&display, &TRIANGLE).unwrap();

    let vertex_data = Arc::new(Mutex::new(VertexData{data: Vec::new(), should_update: false}));

    let transform = Matrix4::from_scale(0.7_f32);

    let mut circle = 0.0_f32;

    let mut world = Matrix4::look_at(Point3{x: 3.0 * circle.sin(), y: 0.0, z: 3.0 * circle.cos()}, Point3{x: 0.0, y: 0.0, z: 0.0}, vec3(0.0, 1.0, 0.0));

    let aspect_ratio = 1280.0_f32 / 720.0;
    let mut view = perspective(Deg(60.0), aspect_ratio, 0.01_f32, 100.0_f32);

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
    let time_last = time.duration_since(time);

    event_loop.run(move |event, _wt, control_flow | {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let delta = time.duration_since(time_earlier);
                time_earlier = time;
                time = Instant::now();
                circle += delta.as_secs_f64() as f32;
                world = Matrix4::look_at(Point3{x: 3.0 * circle.sin(), y: 0.0, z: 3.0 * circle.cos()}, Point3{x: 0.0, y: 0.0, z: 0.0}, vec3(0.0, 1.0, 0.0));
                if vertex_data.lock().unwrap().should_update {
                    vertex_buffer = glium::VertexBuffer::new(&display, &vertex_data.lock().unwrap().data).unwrap();
                }
                let mut target = display.draw();
                target.clear_color(0.1, 0.1, 0.1, 1.0);
                target.clear_depth(1.0);
                target.draw(&vertex_buffer, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program, &uniform!(model: Matrix4Wrapper(transform), view: Matrix4Wrapper(view), world: Matrix4Wrapper(world)), &draw_param).unwrap();
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
                file_dropped(file_path, &display, &mut vertex_buffer, Arc::clone(&vertex_data));
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

fn file_dropped(file_path: PathBuf, display: &glium::Display, vertex_buffer: &mut glium::VertexBuffer<Vertex>, vertex_data: Arc<Mutex<VertexData>>) {
    if let Some(ext) = file_path.extension() {
        if ext == "obj" {
            println!("open {:?}", file_path.file_name());
            let file = File::open(file_path).unwrap();
            let mut file_content = String::new();
            let mut reader = BufReader::new(file);
            reader.read_to_string(&mut file_content).unwrap();
            let obj = obj::parse(file_content).unwrap().objects[0].to_owned();
            let mut verts = Vec::with_capacity(obj.geometry[0].shapes.len() * 3);
            for shape in &obj.geometry[0].shapes {
                match shape.primitive {
                    Primitive::Line(_, _) | Primitive::Point(_) => panic!(),
                    Primitive::Triangle(a, b, c) => {
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
            }
            *vertex_buffer = glium::VertexBuffer::new(display, &verts).unwrap();
            display.gl_window().window().request_redraw();
            thread::spawn(move || {
                for vert in &mut verts {
                    for shape in &obj.geometry[0].shapes {
                        let shape = match shape.primitive {
                            Primitive::Line(_, _) | Primitive::Point(_) => panic!(),
                            Primitive::Triangle(a, b, c) => {
                                let v0 = vec3(obj.vertices[a.0].x as f32, obj.vertices[a.0].y as f32, obj.vertices[a.0].z as f32);
                                let v1 = vec3(obj.vertices[b.0].x as f32, obj.vertices[b.0].y as f32, obj.vertices[b.0].z as f32);
                                let v2 = vec3(obj.vertices[c.0].x as f32, obj.vertices[c.0].y as f32, obj.vertices[c.0].z as f32);
                                [v0, v1, v2]
                            }
                        };
                        let is_hit = ray_triangle_intersect(vec3(vert.pos[0], vert.pos[1], vert.pos[2]),
                                                            vec3(vert.normal[0], vert.normal[1], vert.normal[2]),
                                                            shape);
                        if is_hit {
                            vert.color = [0.0; 3];
                            break;
                        }
                    }
                }
                vertex_data.lock().unwrap().update(verts);
                println!("comp finished");
            });
        }
    }
}
