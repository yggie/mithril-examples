extern crate gl;
extern crate glfw;
extern crate libc;

use std::mem;
use std::ptr;
use std::str;
use std::iter;
use gl::types::*;
use std::ffi::CString;

pub struct GraphicsEngine<'a> {
    program_id: GLuint,
    vertex_shader_id: GLuint,
    fragment_shader_id: GLuint,
    vertex_array_id: GLuint,
    vertex_buffer_id: GLuint,
}

impl<'a> GraphicsEngine<'a> {
    pub fn new(window: &'a glfw::Window) -> GraphicsEngine<'a> {
        let mut graphics = GraphicsEngine{
            program_id: 0,
            vertex_shader_id: 0,
            fragment_shader_id: 0,
            vertex_array_id: 0,
            vertex_buffer_id: 0,
        };

        graphics.initialize(window);

        return graphics;
    }

    fn initialize(&mut self, window: &glfw::Window) {
        gl::load_with(|s| window.get_proc_address(s));

        self.vertex_shader_id = compile_shader(gl::VERTEX_SHADER, "
        #version 150

        in vec3 vPos;

        void main(void) {
            gl_Position = vec4(vPos, 1.0);
        }
        ");

        self.fragment_shader_id = compile_shader(gl::FRAGMENT_SHADER, "
        #version 150

        uniform vec4 color;
        out vec4 out_color;

        void main(void) {
            out_color = vec4(1.0, 0.0, 0.0, 1.0);
        }
        ");

        self.program_id = link_program(self.vertex_shader_id, self.fragment_shader_id);


        unsafe {

            self.vertex_array_id = 0;
            gl::GenVertexArrays(1, &mut self.vertex_array_id as *mut u32);
            gl::BindVertexArray(self.vertex_array_id);

            let vertex_buffer_data: [GLfloat; 9] = [
                -1.0, -1.0, 0.0,
                 1.0, -1.0, 0.0,
                 0.0,  1.0, 0.0,
                ];

            self.vertex_buffer_id = 0;
            gl::GenBuffers(1, &mut self.vertex_buffer_id as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of::<[GLfloat; 9]>() as i64, mem::transmute(&vertex_buffer_data[0]), gl::STATIC_DRAW);

            gl::UseProgram(self.program_id);

            gl::ClearColor(0.3, 0.9, 0.3, 0.9);
            gl::LineWidth(1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::ClearDepth(1.0);
            gl::PolygonMode(gl::FRONT, gl::LINE);
            gl::PolygonMode(gl::BACK, gl::FILL);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }


    pub fn draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DisableVertexAttribArray(0);
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for GraphicsEngine<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
            gl::DeleteShader(self.fragment_shader_id);
            gl::DeleteShader(self.vertex_shader_id);
            gl::DeleteBuffers(1, &self.vertex_buffer_id);
            gl::DeleteVertexArrays(1, &self.vertex_array_id);
        }
    }
}

fn compile_shader(shader_type: GLenum, shader_source: &str) -> GLuint {
    let shader_source_c_str = CString::from_slice(shader_source.as_bytes());

    unsafe {
        let shader = gl::CreateShader(shader_type);

        gl::ShaderSource(shader, 1, &shader_source_c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer: Vec<u8> = iter::repeat(0u8).take(len as usize - 1).collect();
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buffer.as_slice()).unwrap());
        }

        return shader;
    }
}

fn link_program(vertex_shader_id: GLuint, fragment_shader_id: GLuint) -> GLuint {
    unsafe {
        let program_id = gl::CreateProgram();
        gl::AttachShader(program_id, vertex_shader_id);
        gl::AttachShader(program_id, fragment_shader_id);
        gl::LinkProgram(program_id);

        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer: Vec<u8> = iter::repeat(0u8).take(len as usize - 1).collect();
            gl::GetProgramInfoLog(program_id, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buffer.as_slice()).unwrap());
        }

        return program_id;
    }
}
