mod types;
#[macro_use]
mod typeinfo;
mod shader;

use std::mem;
use gl;
use gl::types::*;
use self::typeinfo::TypeInfo;
use self::shader::{Shader, Program};

#[repr(C)]
struct Vertex
{
    pos: [i16; 2],
    col: [f32; 4],
    //tex: [f32; 2],
}

impl_typeinfo!(Vertex, pos, col);

pub struct DrawEngine
{
    prog: Program,
    vbo: GLuint,
}

impl DrawEngine
{
    pub fn new() -> Self
    {
        //FIXME: should use struct gl generator
        // need to prevent leaking state and current context as much as possible
        unsafe
        {
            let prog = Program::new(&[
                Shader::new(gl::VERTEX_SHADER, &[include_str!("test.vert.glsl")]).unwrap_or_else(|e| panic!("vert: {}", e)),
                Shader::new(gl::FRAGMENT_SHADER, &[include_str!("test.frag.glsl")]).unwrap_or_else(|e| panic!("frag: {}", e)),
            ]).unwrap_or_else(|e| panic!("link: {}", e));

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let size = mem::size_of::<Vertex>();
            Vertex::visit_fields(|name, offset, count, ty| {
                let num = prog.get_attrib(name).unwrap();
                //println!("attr '{}' ({}), offset={} count={} type={:?}", name, num, offset, count, ty);
                gl::VertexAttribPointer(num, count as GLint, ty as GLenum, gl::FALSE, size as GLsizei, offset as *const _);
                gl::EnableVertexAttribArray(num);
            });

            DrawEngine{ prog: prog, vbo: vbo }
        }
    }

    pub fn begin_draw(&self, (width, height): (u32, u32)) -> DrawContext
    {
        self.prog.set_active();
        let loc_tf = self.prog.get_uniform("tf").unwrap() as GLint;
        let (w, h) = (width as f32, height as f32);
        let tf = [
            2.0/w, 0.0, 0.0,
            0.0, -2.0/h, 0.0,
            -1.0, 1.0, 1.0,
        ];
        unsafe
        {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
            gl::UniformMatrix3fv(loc_tf, 1, gl::FALSE, tf.as_ptr());

        }
        DrawContext
    }
}

impl Drop for DrawEngine
{
    fn drop(&mut self)
    {
        unsafe{ gl::DeleteBuffers(1, &mut self.vbo) };
    }
}

// TODO. add ref to window/state so we can call swapbuffers on drop
pub struct DrawContext;

impl DrawContext
{
    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32)
    {
        //TODO: discard pending draw operations
        unsafe
        {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    //TODO: add to a queue then dispatch on drop or primitive switch
    pub fn draw_rect(&self, x: i16, y: i16, width: u16, height: u16, color: [f32; 4])
    {
        let xw = x + width as i16;
        let yh = y + height as i16;

        let verts = [
            Vertex{ pos: [x,  y], col: color },
            Vertex{ pos: [xw, y], col: color },
            Vertex{ pos: [xw, yh], col: color },
            Vertex{ pos: [x,  yh], col: color },
        ];
        let idx: [u16; 6] = [
            0, 1, 2,
            2, 3, 0,
        ];

        unsafe
        {
            gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&verts) as GLsizeiptr, verts.as_ptr() as *const _, gl::STREAM_DRAW);
            gl::DrawElements(gl::TRIANGLES, idx.len() as GLsizei, gl::UNSIGNED_SHORT, idx.as_ptr() as *const _);
        }
    }
}
