extern crate glfw;

use glfw::{Action, Context, Key};

fn main() {
    let context = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    context.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    context.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    context.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (window, events) = context.create_window(640, 480, "mithril - testbed", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_all_polling(true);
    window.make_current();

    while !window.should_close() {
        context.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }

                _ => {
                    // do nothing
                }
            }
        }
    }
}
