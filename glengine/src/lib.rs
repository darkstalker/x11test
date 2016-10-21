extern crate array_ext;
extern crate custom_gl as gl;
extern crate custom_egl as egl;

mod types;
#[macro_use]
mod typeinfo;
mod shader;
mod eglw;

use std::mem;
use std::rc::Rc;
use std::ffi::CStr;
use std::ptr;
use gl::types::*;
use typeinfo::TypeInfo;
use array_ext::Array;
use shader::{Shader, Program};

pub use egl::NativeDisplayType;
pub use egl::NativeWindowType;
pub use eglw::Surface;

#[derive(Clone, Copy)]
#[repr(C)]
struct Vertex
{
    pos: [i16; 2],
    col: [f32; 4],
    //tex: [f32; 2],
}

impl_typeinfo!(Vertex, pos, col);

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
enum PrimType
{
    Points = gl::POINTS,
    Lines = gl::LINES,
    Triangles = gl::TRIANGLES,
}

pub struct DrawEngine
{
    egl_disp: eglw::Display,
    gl: Rc<gl::Gles2>,
    prog: Program,
    vbo_vert: GLuint,
    vbo_idx: GLuint,
    pub max_verts: usize,
    pub max_idxs: usize,
}

impl DrawEngine
{
    pub fn new(xdisp: NativeDisplayType) -> Result<Self, &'static str>
    {
        let egl_disp = try!(eglw::Display::new(xdisp));

        let gl = Rc::new(gl::Gles2::load_with(|name| egl_disp.get_proc_address(name)));

        unsafe
        {
            let vendor = CStr::from_ptr(gl.GetString(gl::VENDOR) as *const _);
            let renderer = CStr::from_ptr(gl.GetString(gl::RENDERER) as *const _);
            let version = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _);
            let exts = CStr::from_ptr(gl.GetString(gl::EXTENSIONS) as *const _);
            println!("GL vendor: {:?}\nGL renderer: {:?}\nGL version: {:?}\nGL extensions: {:?}",
                vendor, renderer, version, exts);
        }

        unsafe
        {
            let prog = Program::new(gl.clone(), &[
                Shader::new(gl.clone(), gl::VERTEX_SHADER, &[include_str!("test.vert.glsl")]).unwrap_or_else(|e| panic!("vert: {}", e)),
                Shader::new(gl.clone(), gl::FRAGMENT_SHADER, &[include_str!("test.frag.glsl")]).unwrap_or_else(|e| panic!("frag: {}", e)),
            ]).unwrap_or_else(|e| panic!("link: {}", e));
            prog.set_active();

            // vertex buffer
            let mut vbo_vert = 0;
            gl.GenBuffers(1, &mut vbo_vert);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo_vert);
            let size = mem::size_of::<Vertex>();
            Vertex::visit_fields(|name, offset, count, ty| {
                let num = prog.get_attrib(name).unwrap();
                gl.VertexAttribPointer(num, count as GLint, ty as GLenum, gl::FALSE, size as GLsizei, offset as *const _);
                gl.EnableVertexAttribArray(num);
            });

            // index buffer
            let mut vbo_idx = 0;
            gl.GenBuffers(1, &mut vbo_idx);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_idx);

            Ok(DrawEngine{
                egl_disp: egl_disp,
                gl: gl,
                prog: prog,
                vbo_vert: vbo_vert,
                vbo_idx: vbo_idx,
                // allocate ~1mb per buffer
                max_verts: 52429,
                max_idxs: 524288,
            })
        }
    }

    pub fn create_window_surface(&self, win: NativeWindowType) -> Result<Surface, &'static str>
    {
        self.egl_disp.create_window_surface(win)
    }

    pub fn begin_draw<'a>(&'a self, surface: &'a Surface, (width, height): (u32, u32)) -> DrawContext
    {
        if !surface.is_current()
        {
            let (w, h) = (width as f32, height as f32);
            let tf = [
                2.0/w,    0.0, 0.0, 0.0,
                  0.0, -2.0/h, 0.0, 0.0,
                  0.0,    0.0, 1.0, 0.0,
                 -1.0,    1.0, 0.0, 1.0,
            ];
            surface.make_current();
            let loc_tf = self.prog.get_uniform("tf").unwrap() as GLint;
            unsafe
            {
                self.gl.Viewport(0, 0, width as GLsizei, height as GLsizei);
                self.gl.UniformMatrix4fv(loc_tf, 1, gl::FALSE, tf.as_ptr());
            }
        }

        DrawContext::new(self, surface, self.gl.clone())
    }
}

impl Drop for DrawEngine
{
    fn drop(&mut self)
    {
        unsafe { self.gl.DeleteBuffers(2, [self.vbo_vert, self.vbo_idx].as_ptr()) };
    }
}

pub struct DrawContext<'a>
{
    eng: &'a DrawEngine,
    surface: &'a Surface<'a>,
    gl: Rc<gl::Gles2>,
    ty: PrimType,
    vert_len: usize,
    idx_len: usize,
}

impl<'a> DrawContext<'a>
{
    fn new(eng: &'a DrawEngine, surface: &'a Surface, gl: Rc<gl::Gles2>) -> Self
    {
        let dc = DrawContext{
            eng: eng,
            surface: surface,
            gl: gl,
            ty: PrimType::Triangles,
            vert_len: 0,
            idx_len: 0,
        };

        dc.alloc_vert();
        dc.alloc_idx();
        dc
    }

    pub fn clear(&mut self, color: [f32; 4])
    {
        self.vert_len = 0;
        self.idx_len = 0;
        unsafe
        {
            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn draw_point(&mut self, x: i16, y: i16, color: [f32; 4])
    {
        self.push_elems(PrimType::Points, &[Vertex{ pos: [x, y], col: color }], [0]);
    }

    pub fn draw_line(&mut self, x0: i16, y0: i16, x1: i16, y1: i16, color: [f32; 4])
    {
        self.push_elems(PrimType::Lines, &[
            Vertex{ pos: [x0, y0], col: color },
            Vertex{ pos: [x1, y1], col: color },
        ], [0, 1]);
    }

    pub fn draw_triangle(&mut self, x0: i16, y0: i16, x1: i16, y1: i16, x2: i16, y2: i16, color: [f32; 4])
    {
        self.push_elems(PrimType::Triangles, &[
            Vertex{ pos: [x0, y0], col: color },
            Vertex{ pos: [x1, y1], col: color },
            Vertex{ pos: [x2, y2], col: color },
        ], [0, 1, 2]);
    }

    pub fn draw_rect(&mut self, x: i16, y: i16, width: u16, height: u16, color: [f32; 4])
    {
        let xw = x + width as i16;
        let yh = y + height as i16;

        self.push_elems(PrimType::Triangles, &[
            Vertex{ pos: [ x, y],  col: color },
            Vertex{ pos: [xw, y],  col: color },
            Vertex{ pos: [xw, yh], col: color },
            Vertex{ pos: [ x, yh], col: color }
        ], [
            0, 1, 2,
            2, 3, 0,
        ]);
    }

    fn push_elems<T: Array<u16>>(&mut self, ty: PrimType, verts: &[Vertex], idxs: T)
    {
        assert!(verts.len() <= self.eng.max_verts && idxs.len() <= self.eng.max_idxs);

        if self.ty != ty ||
            self.vert_len + verts.len() > self.eng.max_verts ||
            self.idx_len + idxs.len() > self.eng.max_idxs
        {
            self.commit(true);
            self.ty = ty;
        }

        let vert_size = self.vert_len * mem::size_of::<Vertex>();
        let idx_size = self.idx_len * mem::size_of::<u16>();
        let idx_base = self.vert_len as u16;
        let idx_new = idxs.map(|idx| idx + idx_base);
        unsafe
        {
            self.gl.BufferSubData(gl::ARRAY_BUFFER, vert_size as GLsizeiptr, mem::size_of_val(verts) as GLintptr, verts.as_ptr() as *const _);
            self.gl.BufferSubData(gl::ELEMENT_ARRAY_BUFFER, idx_size as GLsizeiptr, mem::size_of_val(&idx_new) as GLintptr, idx_new.as_ptr() as *const _);
        }
        self.vert_len += verts.len();
        self.idx_len += idx_new.len();
    }

    fn alloc_vert(&self)
    {
        let size = self.eng.max_verts * mem::size_of::<Vertex>();
        unsafe { self.gl.BufferData(gl::ARRAY_BUFFER, size as GLsizeiptr, ptr::null(), gl::STREAM_DRAW) };
    }

    fn alloc_idx(&self)
    {
        let size = self.eng.max_idxs * mem::size_of::<u16>();
        unsafe { self.gl.BufferData(gl::ELEMENT_ARRAY_BUFFER, size as GLsizeiptr, ptr::null(), gl::STREAM_DRAW) };
    }

    fn commit(&mut self, realloc: bool)
    {
        if self.vert_len == 0 { return }

        unsafe { self.gl.DrawElements(self.ty as GLenum, self.idx_len as GLsizei, gl::UNSIGNED_SHORT, 0 as *const _) };

        if realloc
        {
            self.vert_len = 0;
            self.idx_len = 0;
            self.alloc_vert();
            self.alloc_idx();
        }
    }
}

impl<'a> Drop for DrawContext<'a>
{
    fn drop(&mut self)
    {
        self.commit(false);
        self.surface.swap_buffers();
    }
}
