#![feature(unsafe_destructor)]

extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use graphics::GraphicsEngine;
use std::io;
use std::time;

mod graphics;

fn main() {
    let context = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    context.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    context.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    context.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (window, events) = context.create_window(640, 480, "mithril - testbed", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_all_polling(true);
    window.make_current();

    let mut graphics_engine = GraphicsEngine::new(&window);

    let mut timer = io::Timer::new().unwrap();
    let period = timer.periodic(time::Duration::milliseconds(17));
    while !window.should_close() {
        context.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                    break;
                }

                _ => {
                    // do nothing
                }
            }
        }

        graphics_engine.draw();

        window.swap_buffers();
        period.recv().unwrap();
    }
}
