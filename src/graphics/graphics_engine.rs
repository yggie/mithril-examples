extern crate gl;
extern crate glfw;
extern crate mithril;

use std::mem;
use std::ptr;
use std::str;
use std::iter;
use std::rc::Rc;
use gl::types::*;
use self::mithril::math::Vector;
use std::ffi::CString;
use graphics;

macro_rules! verify(
    ($e: expr) => {
        {
            let result = $e;
            assert_eq!(gl::GetError(), 0);
            result
        }
    }
);

pub struct GraphicsEngine<'a> {
    camera: graphics::Camera,
    program_id: GLuint,
    vertex_shader_id: GLuint,
    fragment_shader_id: GLuint,
    color_id: GLint,
    model_matrix_id: GLint,
    view_matrix_id: GLint,
    projection_matrix_id: GLint,
    objects: Vec<graphics::Object<'a>>,
    assets: Vec<Rc<Asset<'a>>>,
    assets_vertex_array_id: GLuint,
}

pub struct Buffer {
    id: GLuint,
    length: usize,
}

pub struct Asset<'a> {
    vertex_buffer: Buffer,
    normal_buffer: Buffer,
    element_buffer: Buffer,
}

impl<'a> GraphicsEngine<'a> {
    pub fn new(window: &glfw::Window) -> GraphicsEngine<'a> {
        let mut graphics = GraphicsEngine{
            camera: graphics::Camera::new(Vector::new(4.0, 4.0, 4.0), Vector::new(0.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)),
            program_id: 0,
            vertex_shader_id: 0,
            fragment_shader_id: 0,
            color_id: -1,
            model_matrix_id: -1,
            view_matrix_id: -1,
            projection_matrix_id: -1,
            assets: Vec::new(),
            assets_vertex_array_id: 0,
            objects: Vec::new(),
        };

        gl::load_with(|s| window.get_proc_address(s));
        graphics.initialize();

        return graphics;
    }

    fn initialize(&mut self) {
        self.vertex_shader_id = compile_shader(gl::VERTEX_SHADER, "
        #version 150

        uniform mat4 model_matrix;
        uniform mat4 view_matrix;
        uniform mat4 projection_matrix;

        in vec3 vertex_pos;
        in vec3 vertex_norm;

        out vec3 normal;

        void main(void) {
            gl_Position = projection_matrix * view_matrix * model_matrix * vec4(vertex_pos, 1.0);
            normal = vec3(view_matrix * model_matrix * vec4(vertex_norm, 0.0));
        }
        ");

        self.fragment_shader_id = compile_shader(gl::FRAGMENT_SHADER, "
        #version 150

        uniform vec4 color;

        in vec3 normal;

        out vec4 out_color;

        void main(void) {
            const vec3 vertex_to_light = normalize(vec3(1.0, 1.0, 0.0));

            float diffuse = clamp(pow(dot(normal, vertex_to_light), 3), 0.0, 0.7) + 0.3;

            out_color = vec4(color.xyz * diffuse, 1.0);
        }
        ");

        self.program_id = link_program(self.vertex_shader_id, self.fragment_shader_id);


        unsafe {
            gl::GenVertexArrays(1, &mut self.assets_vertex_array_id as *mut u32);

            verify!(gl::UseProgram(self.program_id));

            gl::ClearColor(0.1, 0.4, 0.2, 0.9);
            gl::LineWidth(1.0);
            verify!(gl::Enable(gl::DEPTH_TEST));
            gl::DepthFunc(gl::LESS);
            gl::ClearDepth(1.0);

            let model_variable_name = CString::from_slice("model_matrix".as_bytes());
            self.model_matrix_id = gl::GetUniformLocation(self.program_id, model_variable_name.as_ptr());

            let color_variable_name = CString::from_slice("color".as_bytes());
            self.color_id = gl::GetUniformLocation(self.program_id, color_variable_name.as_ptr());
            gl::Uniform4fv(self.color_id, 1, mem::transmute(&[1.0f32, 0.0f32, 0.0f32, 1.0f32][0]));

            let view_matrix_variable_name = CString::from_slice("view_matrix".as_bytes());
            self.view_matrix_id = gl::GetUniformLocation(self.program_id, view_matrix_variable_name.as_ptr());

            let projection_matrix_variable_name = CString::from_slice("projection_matrix".as_bytes());
            self.projection_matrix_id = gl::GetUniformLocation(self.program_id, projection_matrix_variable_name.as_ptr());
        }
    }


    pub fn new_asset_from_file(&mut self, filepath: &str) -> Rc<Asset> {
        let (vertices, normals, indices) = graphics::utils::import_from_obj(filepath);
        let mut vertex_buffer_id: GLuint = 0;
        let mut element_buffer_id: GLuint = 0;
        let mut normal_buffer_id: GLuint = 0;

        unsafe {
            // send vertex data
            gl::GenBuffers(1, &mut vertex_buffer_id as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertices.len() * mem::size_of::<GLfloat>()) as i64,
                           mem::transmute(&vertices.as_slice()[0]),
                           gl::STATIC_DRAW);

            // send vertex index data
            gl::GenBuffers(1, &mut element_buffer_id as *mut u32);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_id);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (indices.len() * mem::size_of::<GLuint>()) as i64,
                           mem::transmute(&indices.as_slice()[0]),
                           gl::STATIC_DRAW);

            // send vertex normal data
            gl::GenBuffers(1, &mut normal_buffer_id as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, normal_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (normals.len() * mem::size_of::<GLfloat>()) as i64,
                           mem::transmute(&normals.as_slice()[0]),
                           gl::STATIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        let asset_ref = Rc::new(Asset{
            vertex_buffer: Buffer{ id: vertex_buffer_id, length: vertices.len() },
            normal_buffer: Buffer{ id: normal_buffer_id, length: normals.len() },
            element_buffer: Buffer{ id: element_buffer_id, length: indices.len() },
        });
        self.assets.push(asset_ref.clone());

        return asset_ref;
    }


    pub fn create_object_from_asset(&mut self, asset: Rc<Asset<'a>>) -> &mut graphics::Object<'a> {
        self.objects.push(graphics::Object::new(asset.clone()));

        // compiler HAX
        let index = self.objects.len() - 1;
        return &mut self.objects[index];
    }


    pub fn camera_mut(&mut self) -> &mut graphics::Camera {
        &mut self.camera
    }


    pub fn draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let view_matrix = self.camera.view_matrix();
            gl::UniformMatrix4fv(self.view_matrix_id, 1, gl::TRUE, mem::transmute(&view_matrix[0]));

            let projection_matrix = self.camera.projection_matrix();
            gl::UniformMatrix4fv(self.projection_matrix_id, 1, gl::TRUE, mem::transmute(&projection_matrix[0]));

            // draw simple objects
            gl::BindVertexArray(self.assets_vertex_array_id);
            for object in self.objects.iter() {
                self.render_object(object);
            }
            gl::BindVertexArray(0);
        }
    }


    fn render_object(&self, object: &graphics::Object) {
        let asset = object.asset();

        unsafe {
            gl::UniformMatrix4fv(self.model_matrix_id, 1, gl::TRUE, mem::transmute(&object.model_matrix()[0]));

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            // set up the vertex data pointer
            gl::BindBuffer(gl::ARRAY_BUFFER, asset.vertex_buffer.id);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            // set up the normal data pointer
            gl::BindBuffer(gl::ARRAY_BUFFER, asset.normal_buffer.id);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            // unbind the buffers, no longer need to modify the pointers
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // bind the common index array
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, asset.element_buffer.id);
            verify!(gl::Enable(gl::DEPTH_TEST));
            gl::DrawElements(gl::TRIANGLES, asset.element_buffer.length as i32, gl::UNSIGNED_INT, ptr::null());

            gl::DisableVertexAttribArray(1);
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
            gl::DeleteVertexArrays(1, &self.assets_vertex_array_id);

            for asset in self.assets.iter() {
                gl::DeleteBuffers(1, &asset.vertex_buffer.id);
                gl::DeleteBuffers(1, &asset.element_buffer.id);
                gl::DeleteBuffers(1, &asset.normal_buffer.id);
            }
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
