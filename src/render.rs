use glium::{Display, Program, VertexBuffer, IndexBuffer, DrawParameters, Surface, uniform};
use glium::uniforms::UniformValue;
use glium::glutin::window::WindowBuilder;
use glium::glutin::event_loop::EventLoop;
use glium::index::PrimitiveType;
use glium::glutin::dpi::LogicalSize;

use std::time::Instant;
use std::sync::{Arc, Mutex};

use crate::consts::*;
use crate::world_data::WorldData;
use crate::geo::Vertex;

use cgmath::{perspective, Deg, Matrix4};

pub struct Renderer {
    display: Display,
    program: Program,
    mesh_vbuffer: VertexBuffer<Vertex>,
    pub mesh_vdata: Arc<Mutex<VertexData>>,
    mesh_indices: IndexBuffer<u32>,
    pub world_data: WorldData,
    view_matrix: Matrix4<f32>,
    delta_timer: DeltaTimer,
    draw_parameters: DrawParameters<'static>
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>, wb: WindowBuilder) -> Self {
        let cb = glium::glutin::ContextBuilder::new().with_depth_buffer(16).with_srgb(false);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let program = make_program(&display);

        let mesh_vbuffer = glium::VertexBuffer::new(&display, &[]).unwrap();
        let mesh_vdata = Arc::new(Mutex::new(VertexData{data: Vec::new(), should_update: false}));
        let mesh_indices = glium::index::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[]).unwrap();

        let world_data = WorldData::default();

        let view_matrix = perspective(Deg(60.0), ASPECT_RATIO, 0.01_f32, 100.0_f32);

        let delta_timer = DeltaTimer::new();

        let draw_parameters = DrawParameters {
            depth: glium::Depth{
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        Renderer {
            display,
            program,
            mesh_vbuffer,
            mesh_vdata,
            mesh_indices,
            world_data,
            view_matrix,
            delta_timer,
            draw_parameters
        }
    }

    pub fn redraw(&mut self) {
        self.world_data.rotate_delta(self.delta_timer.next_delta());
        {
            let lock = self.mesh_vdata.lock().unwrap();
            if lock.should_update {
                self.mesh_vbuffer = glium::VertexBuffer::new(&self.display, &lock.data).unwrap();
            }
        }
        let mut target = self.display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);
        target.clear_depth(1.0);
        let uniforms = uniform!(
            view: Matrix4Wrapper(self.view_matrix),
            world: Matrix4Wrapper(*self.world_data.world_mat()),
            light: self.world_data.shading_enabled,
            ao: self.world_data.ao_enabled
        );

        target.draw(
            &self.mesh_vbuffer,
            &self.mesh_indices,
            &self.program,
            &uniforms,
            &self.draw_parameters
        ).unwrap();

        target.finish().unwrap();
    }

    pub fn request_redraw(&self) {
        self.display.gl_window().window().request_redraw();
    }

    pub fn update_aspect_ratio(&mut self, new_size: LogicalSize) {
        let aspect_ratio = new_size.width / new_size.height;
        self.view_matrix = perspective( Deg(60.0), aspect_ratio as f32, 0.01_f32, 100.0_f32);
    }

    pub fn update_mesh_data(&mut self, data: Vec<Vertex>, indices: Vec<u32>) {
        self.mesh_indices = glium::index::IndexBuffer::new(&self.display, PrimitiveType::TrianglesList, &indices).unwrap();
        self.mesh_vdata.lock().unwrap().update(data);
    }
}

pub struct DeltaTimer {
    time: Instant
}

impl DeltaTimer {
    pub fn new() -> Self {
        DeltaTimer {
            time: Instant::now()
        }
    }

    pub fn next_delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.time);
        self.time = now;
        delta.as_secs_f32()
    }
}

pub struct VertexData {
    pub data: Vec<Vertex>,
    pub should_update: bool
}

impl VertexData {
    pub fn update(&mut self, data: Vec<Vertex>) {
        self.data = data;
        self.should_update = true;
    }
}

pub fn make_program(display: &Display) -> Program {
    let vert = include_str!("test.vert");
    let frag = include_str!("test.frag");

    glium::Program::from_source(display, vert, frag, None).unwrap()
}

pub struct Matrix4Wrapper(pub cgmath::Matrix4<f32>);

impl glium::uniforms::AsUniformValue for Matrix4Wrapper {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4(self.0.into())
    }
}
