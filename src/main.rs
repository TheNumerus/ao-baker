#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use glium::glutin::event_loop::EventLoop;

use ao_baker::Window;

fn main() {
    let event_loop = EventLoop::new();
    let mut window = Window::new(&event_loop);
    event_loop.run(move |event, _wt, control_flow| window.event_handler(event, control_flow));
}
