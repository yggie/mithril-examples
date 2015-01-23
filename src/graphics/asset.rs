extern crate gl;

use graphics;
use std::mem;
use gl::types::{ GLfloat, GLuint };

pub struct Asset<'a> {
    vertex_buffer_id: GLuint,
    element_buffer_id: GLuint,
    normal_buffer_id: GLuint,
    num_vertices: usize,
    num_indices: usize,
    num_normals: usize,
}

impl<'a> Asset<'a> {
    pub fn new_from_file(filepath: &str) -> Asset<'a> {
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

        return Asset{
            vertex_buffer_id: vertex_buffer_id,
            element_buffer_id: element_buffer_id,
            normal_buffer_id: normal_buffer_id,
            num_vertices: vertices.len(),
            num_indices: indices.len(),
            num_normals: normals.len(),
        };
    }
}

#[unsafe_destructor]
impl<'a> Drop for Asset<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_buffer_id);
            gl::DeleteBuffers(1, &self.element_buffer_id);
            gl::DeleteBuffers(1, &self.normal_buffer_id);
        }
    }
}
