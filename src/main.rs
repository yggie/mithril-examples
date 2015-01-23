#![feature(unsafe_destructor)]
#![allow(unstable)]

extern crate gl;
extern crate glfw;

use glfw::{Action, Context, Glfw, Key, Window, WindowEvent};
use graphics::GraphicsEngine;
use std::io;
use std::time;
use std::sync;

mod graphics;

fn main() {
    let mut app = Application::new();

    app.run(time::Duration::milliseconds(17));
}

pub struct Application<'a> {
    context: glfw::Glfw,
    window: glfw::Window,
    graphics: GraphicsEngine<'a>,
    events_receiver: sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    timer: io::Timer,
    left_mouse_button_down: bool,
}

impl<'a> Application<'a> {
    fn new() -> Application<'a> {
        let context = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        context.window_hint(glfw::WindowHint::ContextVersion(3, 2));
        context.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
        context.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

        let (window, events) = context.create_window(640, 480, "mithril - testbed", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        window.set_all_polling(true);
        window.make_current();

        return Application{
            context: context,
            graphics: GraphicsEngine::new(&window),
            window: window,
            timer: io::Timer::new().unwrap(),
            events_receiver: events,
            left_mouse_button_down: false,
        };
    }

    fn run(&mut self, duration: time::Duration) {
        let period = self.timer.periodic(duration);

        {
            let asset_ref = self.graphics.new_asset_from_file("assets/cube.obj");
            self.graphics.create_object_from_asset(asset_ref.clone());
            let obj = self.graphics.create_object_from_asset(asset_ref.clone());
            obj.set_translation(-3.0, -1.0, -1.0);
        }

        {
            let asset_ref = self.graphics.new_asset_from_file("assets/isosphere.obj");
            let obj = self.graphics.create_object_from_asset(asset_ref.clone());
            obj.set_translation(3.0, 2.0, -1.0);
        }

        while !self.window.should_close() {
            self.context.poll_events();
            self.flush_events_queue();

            self.graphics.camera_mut().update();
            self.graphics.draw();

            self.window.swap_buffers();
            period.recv().unwrap();
        }
    }

    fn flush_events_queue(&mut self) {
        for (time, event) in glfw::flush_messages(&self.events_receiver) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }

                glfw::WindowEvent::Scroll(_, y) => {
                    let mut camera = self.graphics.camera_mut();
                    let new_pos = (camera.position() - camera.focus_point()) * (1.0 + y as f32) + camera.focus_point();
                    camera.go_to(new_pos);
                }

                glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, glfw::Action::Press, _) => {
                    self.left_mouse_button_down = true;
                }

                glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, glfw::Action::Release, _) => {
                    self.left_mouse_button_down = false;
                    if self.graphics.camera_mut().is_controlled() {
                        self.graphics.camera_mut().release_controls();
                    }
                }

                glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                    println!("Time: {:?}, Button: {:?}, Action: {:?}, Modifiers: [{:?}]", time, glfw::ShowAliases(button), action, modifiers)
                }

                glfw::WindowEvent::CursorPos(x, y) => {
                    if self.left_mouse_button_down {
                        let camera = self.graphics.camera_mut();
                        let (width, height) = self.window.get_size();

                        let x_norm = 2.0 * (x - 0.5 * width as f64)/(width as f64);
                        let y_norm = -2.0 * (y - 0.5 * height as f64)/(height as f64);

                        if camera.is_controlled() {
                            camera.set_control_point(x_norm, y_norm);
                        } else {
                            camera.start_control(x_norm, y_norm);
                        }
                    }
                }

                _ => {
                    // do nothing
                }
            }
        }
    }
}
