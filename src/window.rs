use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::window::WindowBuilder;
use glium::glutin::event::{Event, WindowEvent, MouseScrollDelta, DeviceEvent};

use crate::consts::*;
use crate::render::Renderer;
use crate::geo::generate_vector_buffer;
use crate::io::read_obj;
use crate::compute::{compute_ao, ComputeData};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Window {
    renderer: Renderer,
    is_mouse_pressed: bool,
    is_middle_mouse_pressed: bool,
    is_focused: bool,
    bake_in_progress: Arc<Mutex<bool>>,
    compute_data: ComputeData
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let wb = WindowBuilder::new()
            .with_title(APP_NAME.to_string())
            .with_inner_size((SIZE_X, SIZE_Y).into()).with_min_inner_size((400, 400).into());
        let renderer = Renderer::new(event_loop, wb);

        Window {
            renderer,
            is_mouse_pressed: false,
            is_middle_mouse_pressed: false,
            is_focused: false,
            bake_in_progress: Arc::new(Mutex::new(false)),
            compute_data: ComputeData::default()
        }
    }

    pub fn event_handler(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => self.renderer.redraw(),
            Event::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::MouseWheel{delta, ..} => {
                        // handle zoom
                        if let MouseScrollDelta::LineDelta(_, val) = delta {
                            self.renderer.world_data.adjust_zoom(val as i32);
                        }
                    },
                    WindowEvent::Resized(new_size) => self.renderer.update_aspect_ratio(new_size),
                    WindowEvent::DroppedFile(file_path) => {
                        self.file_dropped(file_path);
                    },
                    WindowEvent::ReceivedCharacter(ch) => {
                        match ch {
                            'd' | 'D' => self.renderer.world_data.toggle_shading(),
                            'p' | 'P' => self.renderer.world_data.toggle_paused(),
                            'f' | 'F' => self.renderer.world_data.toggle_ao(),
                            _ => {}
                        }
                    },
                    WindowEvent::MouseInput{button, state, ..} => {
                        if let glium::glutin::event::MouseButton::Left = button {
                            match state {
                                glium::glutin::event::ElementState::Pressed => self.is_mouse_pressed = true,
                                glium::glutin::event::ElementState::Released => self.is_mouse_pressed = false,
                            }
                        }
                        if let glium::glutin::event::MouseButton::Middle = button {
                            match state {
                                glium::glutin::event::ElementState::Pressed => self.is_middle_mouse_pressed = true,
                                glium::glutin::event::ElementState::Released => self.is_middle_mouse_pressed = false,
                            }
                        }
                    },
                    WindowEvent::Focused(focus) => self.is_focused = focus,
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{delta},
                ..
            } => {
                if self.is_focused && self.is_mouse_pressed {
                    self.renderer.world_data.rotate_manual(delta);
                } else if self.is_focused && self.is_middle_mouse_pressed {
                    self.renderer.world_data.pan_manual(delta);
                }
            },
            _ => {
                if self.is_focused {
                    *control_flow = ControlFlow::Poll;
                    self.renderer.request_redraw();
                } else {
                    *control_flow = ControlFlow::Wait;
                }
            }
        }
    }

    fn file_dropped(&mut self, file_path: PathBuf) {
        let ext = file_path.extension().unwrap_or_default();
        if ext != "obj" {
            return;
        }
        println!("opening {:?}", file_path.file_name());
        let name = format!("{} <{}>", APP_NAME, file_path.file_name().unwrap().to_str().unwrap());
        self.renderer.set_window_title(&name);
        {
            let mut lock = self.bake_in_progress.lock().unwrap();
            if *lock {
                return;
            } else {
                *lock = true;
            }
        }
        let obj = read_obj(file_path);
        let (verts, indices) = generate_vector_buffer(&obj);
        self.renderer.update_mesh_data(verts.to_owned(), indices);
        self.renderer.request_redraw();
        compute_ao(Arc::clone(&self.renderer.mesh_vdata), obj, verts, Arc::clone(&self.bake_in_progress), &self.compute_data);
    }
}
