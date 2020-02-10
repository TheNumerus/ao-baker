use glium::{Display, Program, VertexBuffer, IndexBuffer, DrawParameters, Surface, uniform};
use glium::uniforms::UniformValue;
use glium::glutin::window::WindowBuilder;
use glium::glutin::event_loop::EventLoop;
use glium::index::PrimitiveType;
use glium::glutin::dpi::LogicalSize;
use glium::texture::{Texture2d, RawImage2d};

use std::time::Instant;
use std::sync::{Arc, Mutex};

use crate::consts::*;
use crate::world_data::WorldData;
use crate::geo::{Vertex, VertexUV};

use cgmath::{perspective, Deg, Matrix4, Matrix3};

use rusttype::FontCollection;

mod tooltips;

pub struct Renderer {
    display: Display,
    program: Program,
    program_tooltip: Program,
    mesh_vbuffer: VertexBuffer<Vertex>,
    pub mesh_vdata: Arc<Mutex<VertexData>>,
    mesh_indices: IndexBuffer<u32>,
    pub world_data: WorldData,
    view_matrix: Matrix4<f32>,
    delta_timer: DeltaTimer,
    draw_parameters: DrawParameters<'static>,
    quad_vbuffer: VertexBuffer<VertexUV>,
    tooltip_textures: Vec<Texture2d>,
    tooltip_transform: Matrix3<f32>,
    grid_vbuffer: VertexBuffer<Vertex>,
    grid_program: Program
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>, wb: WindowBuilder) -> Self {
        let cb = glium::glutin::ContextBuilder::new().with_depth_buffer(16).with_srgb(false);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let (program, program_tooltip, grid_program) = Self::make_programs(&display);

        let mesh_vbuffer = glium::VertexBuffer::new(&display, &[]).unwrap();
        let mesh_vdata = Arc::new(Mutex::new(VertexData{data: Vec::new(), should_update: false}));
        let mesh_indices = glium::index::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[]).unwrap();

        let quad_vbuffer = glium::VertexBuffer::new(&display, &QUAD).unwrap();

        let world_data = WorldData::default();

        let view_matrix = perspective(Deg(60.0), ASPECT_RATIO, 0.01_f32, 100.0_f32);

        let delta_timer = DeltaTimer::new();

        let draw_parameters = DrawParameters {
            depth: glium::Depth{
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        let collection = FontCollection::from_bytes(FONT_BYTES).unwrap();
        let font = collection.into_font().unwrap();

        let mut tooltip_textures = Vec::new();
        for tooltip in &TOOLTIPS {
            let (tooltip_width, tooltip_data) = tooltips::texture_data_from_str(&font, 64.0, tooltip);
            let tooltip_image = RawImage2d::from_raw_rgba_reversed(&tooltip_data, (tooltip_width as u32, 64));
            tooltip_textures.push(Texture2d::new(&display, tooltip_image).unwrap());
        }

        let (size_x, size_y) = display.get_framebuffer_dimensions();
        let ratio = size_x as f32 / size_y as f32;

        let tooltip_transform = Matrix3::new(
            0.1 / ratio, 0.0, -1.0,
            0.0, 0.1, -1.0,
            0.0, 0.0, 1.0,
        );

        let grid_vbuffer = Self::get_grid_buffer(&display);

        Renderer {
            display,
            program,
            program_tooltip,
            mesh_vbuffer,
            mesh_vdata,
            mesh_indices,
            world_data,
            view_matrix,
            delta_timer,
            draw_parameters,
            quad_vbuffer,
            tooltip_textures,
            tooltip_transform,
            grid_vbuffer,
            grid_program,
        }
    }

    fn get_grid_buffer(display: &Display) -> VertexBuffer<Vertex> {
        let mut vec = Vec::with_capacity(21 * 4 * 2);
        let line_width = 0.01;
        let normal = [0.0, 1.0, 0.0];
        let color = [0.0; 3];

        for i in -10..=10 {
            vec.push(Vertex{normal, color, pos: [i as f32 + line_width, 0.0, 10.0]});
            vec.push(Vertex{normal, color, pos: [i as f32 - line_width, 0.0, -10.0]});
            vec.push(Vertex{normal, color, pos: [i as f32 - line_width, 0.0, 10.0]});

            vec.push(Vertex{normal, color, pos: [i as f32 + line_width, 0.0, 10.0]});
            vec.push(Vertex{normal, color, pos: [i as f32 + line_width, 0.0, -10.0]});
            vec.push(Vertex{normal, color, pos: [i as f32 - line_width, 0.0, -10.0]});

            vec.push(Vertex{normal, color, pos: [10.0, 0.0, i as f32 + line_width]});
            vec.push(Vertex{normal, color, pos: [10.0, 0.0, i as f32 - line_width]});
            vec.push(Vertex{normal, color, pos: [-10.0, 0.0, i as f32 - line_width]});

            vec.push(Vertex{normal, color, pos: [10.0, 0.0, i as f32 + line_width]});
            vec.push(Vertex{normal, color, pos: [-10.0, 0.0, i as f32 - line_width,]});
            vec.push(Vertex{normal, color, pos: [-10.0, 0.0, i as f32 + line_width]});
        }

        glium::VertexBuffer::new(display, &vec).unwrap()
    }

    pub fn redraw(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);
        target.clear_depth(1.0);

        self.world_data.rotate_delta(self.delta_timer.next_delta());
        {
            let lock = self.mesh_vdata.lock().unwrap();
            if lock.should_update {
                self.mesh_vbuffer = glium::VertexBuffer::new(&self.display, &lock.data).unwrap();
            }
        }

        let grid_uniforms = uniform!(
            view: Matrix4Wrapper(self.view_matrix),
            world: Matrix4Wrapper(*self.world_data.world_mat())
        );

        target.draw(
            &self.grid_vbuffer,
            glium::index::NoIndices(PrimitiveType::TrianglesList),
            &self.grid_program,
            &grid_uniforms,
            &self.draw_parameters
        ).unwrap();

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

        for (index, tooltip) in self.tooltip_textures.iter().enumerate() {
            let mut tooltip_transform = self.tooltip_transform.to_owned();
            let (size_x, size_y) = self.display.get_framebuffer_dimensions();
            let ratio = size_x as f32 / size_y as f32;

            tooltip_transform.y.z = -1.0 + (index as f32) * 0.1;
            tooltip_transform.x.x = (0.1 / ratio) * tooltip.width() as f32 / 64.0;

            let tooltip_uniforms = uniform!(
                font_texture: tooltip,
                transform: Matrix3Wrapper(tooltip_transform)
            );

            target.draw(
                &self.quad_vbuffer,
                glium::index::NoIndices(PrimitiveType::TrianglesList),
                &self.program_tooltip,
                &tooltip_uniforms,
                &self.draw_parameters
            ).unwrap();
        }

        target.finish().unwrap();
    }

    pub fn request_redraw(&self) {
        self.display.gl_window().window().request_redraw();
    }

    pub fn set_window_title(&self, title: &str) {
        self.display.gl_window().window().set_title(title);
    }

    pub fn update_aspect_ratio(&mut self, new_size: LogicalSize) {
        let aspect_ratio = new_size.width / new_size.height;
        self.view_matrix = perspective( Deg(60.0), aspect_ratio as f32, 0.01_f32, 100.0_f32);
    }

    pub fn update_mesh_data(&mut self, data: Vec<Vertex>, indices: Vec<u32>) {
        self.mesh_indices = glium::index::IndexBuffer::new(&self.display, PrimitiveType::TrianglesList, &indices).unwrap();
        self.mesh_vdata.lock().unwrap().update(data);
    }

    fn make_programs(display: &Display) -> (Program, Program, Program) {
        let vert = include_str!("shaders/mesh.vert");
        let frag = include_str!("shaders/mesh.frag");

        let vert_tooltip = include_str!("shaders/tooltip.vert");
        let frag_tooltip = include_str!("shaders/tooltip.frag");

        let vert_grid = include_str!("shaders/grid.vert");
        let frag_grid = include_str!("shaders/grid.frag");

        let program_mesh = glium::Program::from_source(
            display,
            vert,
            frag,
            None
        ).unwrap();

        let program_tooltip = glium::Program::from_source(
            display,
            vert_tooltip,
            frag_tooltip,
            None
        ).unwrap();

        let grid_program = glium::Program::from_source(
            display,
            vert_grid,
            frag_grid,
            None
        ).unwrap();

        (program_mesh, program_tooltip, grid_program)
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

pub struct Matrix4Wrapper(pub cgmath::Matrix4<f32>);

impl glium::uniforms::AsUniformValue for Matrix4Wrapper {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4(self.0.into())
    }
}

pub struct Matrix3Wrapper(pub cgmath::Matrix3<f32>);

impl glium::uniforms::AsUniformValue for Matrix3Wrapper {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat3(self.0.into())
    }
}
