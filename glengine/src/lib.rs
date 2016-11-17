extern crate array_ext;
extern crate custom_gl as gl;
extern crate custom_egl as egl;

mod types;
#[macro_use]
mod typeinfo;
mod shader;
mod eglw;

use std::mem;
use std::ffi::CStr;
use std::ptr;
use std::cell::Cell;
use gl::types::*;
use typeinfo::TypeInfo;
use array_ext::Array;
use shader::{Shader, Program};

pub use egl::NativeDisplayType;
pub use egl::NativeWindowType;
pub use eglw::Surface;
pub use types::{Point, Color, TexCoord, Rect};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Vertex
{
    pos: [i16; 2],
    col: [f32; 4],
    texc: [f32; 2],
}

impl_typeinfo!(Vertex, pos, col, texc);

pub struct DrawEngine
{
    egl_disp: eglw::Display,
    prog: Program,
    vbo_vert: GLuint,
    vbo_idx: GLuint,
    default_tex: GLuint,
    vert_off: Cell<usize>,
    idx_off: Cell<usize>,
    vert_len: Cell<usize>,
    idx_len: Cell<usize>,
    cur_tex: Cell<GLuint>,
    pub max_verts: usize,
    pub max_idxs: usize,
}

impl DrawEngine
{
    pub fn new(xdisp: NativeDisplayType) -> Result<Self, &'static str>
    {
        let egl_disp = eglw::Display::new(xdisp)?;

        unsafe
        {
            let vendor = CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _);
            let renderer = CStr::from_ptr(gl::GetString(gl::RENDERER) as *const _);
            let version = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _);
            let exts = CStr::from_ptr(gl::GetString(gl::EXTENSIONS) as *const _);
            println!("GL vendor: {:?}\nGL renderer: {:?}\nGL version: {:?}\nGL extensions: {:?}",
                vendor, renderer, version, exts);
        }

        let mut eng = unsafe {
            let prog = Program::new(&[
                Shader::new(gl::VERTEX_SHADER, &[include_str!("test.vert.glsl")]).unwrap_or_else(|e| panic!("vert: {}", e)),
                Shader::new(gl::FRAGMENT_SHADER, &[include_str!("test.frag.glsl")]).unwrap_or_else(|e| panic!("frag: {}", e)),
            ]).unwrap_or_else(|e| panic!("link: {}", e));
            prog.set_active();

            // vertex buffer
            let mut vbo_vert = 0;
            gl::GenBuffers(1, &mut vbo_vert);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_vert);
            let size = mem::size_of::<Vertex>();
            Vertex::visit_fields(|name, offset, count, ty| {
                let num = prog.get_attrib(name).unwrap();
                gl::VertexAttribPointer(num, count as GLint, ty as GLenum, gl::FALSE, size as GLsizei, offset as *const _);
                gl::EnableVertexAttribArray(num);
            });

            // index buffer
            let mut vbo_idx = 0;
            gl::GenBuffers(1, &mut vbo_idx);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_idx);

            DrawEngine{
                egl_disp: egl_disp,
                prog: prog,
                vbo_vert: vbo_vert,
                vbo_idx: vbo_idx,
                default_tex: 0,
                vert_off: Cell::new(0),
                idx_off: Cell::new(0),
                vert_len: Cell::new(0),
                idx_len: Cell::new(0),
                cur_tex: Cell::new(0),
                // allocate 1mb
                max_verts: 32768,
                max_idxs: 65536,
            }
        };

        // 1x1 white texture
        let tex = eng.create_texture(1, 1);
        eng.update_texture(tex, 0, 0, 1, 1, &[255, 255, 255, 255]);
        eng.default_tex = tex;

        eng.alloc_vert();
        eng.alloc_idx();

        Ok(eng)
    }

    pub fn create_window_surface(&self, win: NativeWindowType) -> Result<Surface, &'static str>
    {
        self.egl_disp.create_window_surface(win)
    }

    pub fn create_texture(&self, width: u32, height: u32) -> GLuint
    {
        unsafe
        {
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            self.cur_tex.set(tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, width as GLsizei, height as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
            tex
        }
    }

    pub fn update_texture(&self, tex_id: GLuint, x: i32, y: i32, width: u32, height: u32, rgba_data: &[u8])
    {
        assert!(width as usize * height as usize * 4 == rgba_data.len());
        if self.cur_tex.get() != tex_id
        {
            unsafe { gl::BindTexture(gl::TEXTURE_2D, tex_id) };
            self.cur_tex.set(tex_id);
        }
        unsafe { gl::TexSubImage2D(gl::TEXTURE_2D, 0, x, y, width as GLsizei, height as GLsizei, gl::RGBA, gl::UNSIGNED_BYTE, rgba_data.as_ptr() as *const _) };
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
                gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
                gl::UniformMatrix4fv(loc_tf, 1, gl::FALSE, tf.as_ptr());
            }
        }

        DrawContext::new(self, surface)
    }

    fn alloc_vert(&self)
    {
        self.vert_off.set(0);
        let size = self.max_verts * mem::size_of::<Vertex>();
        unsafe { gl::BufferData(gl::ARRAY_BUFFER, size as GLsizeiptr, ptr::null(), gl::DYNAMIC_DRAW) };
    }

    fn alloc_idx(&self)
    {
        self.idx_off.set(0);
        let size = self.max_idxs * mem::size_of::<u16>();
        unsafe { gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size as GLsizeiptr, ptr::null(), gl::DYNAMIC_DRAW) };
    }

    fn clear(&self, r: f32, g: f32, b: f32, a: f32)
    {
        self.vert_len.set(0);
        self.idx_len.set(0);
        unsafe
        {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn push_elems<T: Array<u16>>(&self, tex: GLuint, verts: &[Vertex], idxs: T)
    {
        assert!(verts.len() <= self.max_verts && idxs.len() <= self.max_idxs);
        let tex_id = if tex == 0 { self.default_tex } else { tex };

        let vert_start = self.vert_off.get() + self.vert_len.get();
        let idx_start = self.idx_off.get() + self.idx_len.get();

        let oom_vert = vert_start + verts.len() > self.max_verts;
        let oom_idx = idx_start + idxs.len() > self.max_idxs;
        let new_tex = self.cur_tex.get() != tex_id;

        if oom_vert || oom_idx || new_tex
        {
            self.commit();

            if oom_vert { self.alloc_vert(); }
            if oom_idx { self.alloc_idx(); }

            if new_tex
            {
                self.cur_tex.set(tex_id);
                unsafe { gl::BindTexture(gl::TEXTURE_2D, tex_id) };
            }
        }

        let vert_size = vert_start * mem::size_of::<Vertex>();
        unsafe { gl::BufferSubData(gl::ARRAY_BUFFER, vert_size as GLsizeiptr, mem::size_of_val(verts) as GLintptr, verts.as_ptr() as *const _) };
        self.vert_len.set(self.vert_len.get() + verts.len());

        let idxs = idxs.map_(|idx| idx + vert_start as u16);
        let idx_size = idx_start * mem::size_of::<u16>();
        unsafe { gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, idx_size as GLsizeiptr, mem::size_of_val(&idxs) as GLintptr, idxs.as_ptr() as *const _) };
        self.idx_len.set(self.idx_len.get() + idxs.len());
    }

    fn commit(&self)
    {
        if self.vert_len.get() == 0 { return }

        let offset = self.idx_off.get() * mem::size_of::<i16>();
        unsafe { gl::DrawElements(gl::TRIANGLES, self.idx_len.get() as GLsizei, gl::UNSIGNED_SHORT, offset as *const _) };

        self.vert_off.set(self.vert_off.get() + self.vert_len.get());
        self.idx_off.set(self.idx_off.get() + self.idx_len.get());

        self.vert_len.set(0);
        self.idx_len.set(0);
    }
}

impl Drop for DrawEngine
{
    fn drop(&mut self)
    {
        unsafe
        {
            gl::DeleteBuffers(2, [self.vbo_vert, self.vbo_idx].as_ptr());
            gl::DeleteTextures(1, &self.default_tex);
        }
    }
}

pub struct DrawContext<'a>
{
    eng: &'a DrawEngine,
    surface: &'a Surface<'a>,
}

impl<'a> DrawContext<'a>
{
    fn new(eng: &'a DrawEngine, surface: &'a Surface) -> Self
    {
        DrawContext{
            eng: eng,
            surface: surface,
        }
    }

    pub fn clear(&self, color: Color)
    {
        self.eng.clear(color[0], color[1], color[2], color[3]);
    }

    pub fn draw_triangle<T>(&self, p0: Point, p1: Point, p2: Point, params: T)
        where T: Into<DrawTriangleParams>
    {
        let DrawTriangleParams{ color, tex_id, texc } = params.into();

        self.eng.push_elems(tex_id, &[
            Vertex{ pos: p0, col: color, texc: texc[0] },
            Vertex{ pos: p1, col: color, texc: texc[1] },
            Vertex{ pos: p2, col: color, texc: texc[2] },
        ], [0, 1, 2]);
    }

    pub fn draw_rect<T>(&self, pos: Point, width: u16, height: u16, params: T)
        where T: Into<DrawRectParams>
    {
        let (x, y) = (pos[0], pos[1]);
        let xw = x + width as i16;
        let yh = y + height as i16;
        let DrawRectParams{ color, tex_id, texc } = params.into();

        self.eng.push_elems(tex_id, &[
            Vertex{ pos: pos,      col: color, texc: texc.top_left() },
            Vertex{ pos: [xw, y],  col: color, texc: texc.top_right() },
            Vertex{ pos: [xw, yh], col: color, texc: texc.bottom_right() },
            Vertex{ pos: [ x, yh], col: color, texc: texc.bottom_left() },
        ], [0, 1, 2,
            2, 3, 0]);
    }
}

impl<'a> Drop for DrawContext<'a>
{
    fn drop(&mut self)
    {
        self.eng.commit();
        self.surface.swap_buffers();
    }
}

pub struct Texture<'a>
{
    id: GLuint,
    eng: &'a DrawEngine,
}

impl<'a> Texture<'a>
{
    pub fn new(eng: &'a DrawEngine, width: u32, height: u32) -> Self
    {
        Texture{
            id: eng.create_texture(width, height),
            eng: eng,
        }
    }

    pub fn update(&self, x: i32, y: i32, width: u32, height: u32, rgba_data: &[u8])
    {
        self.eng.update_texture(self.id, x, y, width, height, rgba_data)
    }
}

impl<'a> Drop for Texture<'a>
{
    fn drop(&mut self)
    {
        unsafe { gl::DeleteTextures(1, &self.id) };
    }
}

#[derive(Clone, Copy)]
pub struct DrawTriangleParams
{
    color: Color,
    tex_id: GLuint,
    texc: [TexCoord; 3],
}

impl From<()> for DrawTriangleParams
{
    fn from(_u: ()) -> Self
    {
        From::from([1.0, 1.0, 1.0, 1.0])
    }
}

impl From<Color> for DrawTriangleParams
{
    fn from(col: Color) -> Self
    {
        DrawTriangleParams{ color: col, tex_id: 0, texc: Default::default() }
    }
}

impl<'a> From<(&'a Texture<'a>, [TexCoord; 3])> for DrawTriangleParams
{
    fn from((tex, texc): (&'a Texture<'a>, [TexCoord; 3])) -> Self
    {
        From::from(([1.0, 1.0, 1.0, 1.0], tex, texc))
    }
}

impl<'a> From<(Color, &'a Texture<'a>, [TexCoord; 3])> for DrawTriangleParams
{
    fn from((col, tex, texc): (Color, &'a Texture<'a>, [TexCoord; 3])) -> Self
    {
        DrawTriangleParams{ color: col, tex_id: tex.id, texc: texc }
    }
}

#[derive(Clone, Copy)]
pub struct DrawRectParams
{
    color: Color,
    tex_id: GLuint,
    texc: Rect<f32>,
}

impl From<()> for DrawRectParams
{
    fn from(_u: ()) -> Self
    {
        From::from([1.0, 1.0, 1.0, 1.0])
    }
}

impl From<Color> for DrawRectParams
{
    fn from(col: Color) -> Self
    {
        DrawRectParams{ color: col, tex_id: 0, texc: Default::default() }
    }
}

impl<'a> From<&'a Texture<'a>> for DrawRectParams
{
    fn from(tex: &Texture) -> Self
    {
        From::from((tex, [0.0, 0.0], [1.0, 1.0]))
    }
}

impl<'a> From<(Color, &'a Texture<'a>)> for DrawRectParams
{
    fn from((col, tex): (Color, &Texture)) -> Self
    {
        From::from((col, tex, [0.0, 0.0], [1.0, 1.0]))
    }
}

impl<'a> From<(&'a Texture<'a>, TexCoord, TexCoord)> for DrawRectParams
{
    fn from((tex, tl, br): (&'a Texture, TexCoord, TexCoord)) -> Self
    {
        From::from((tex, Rect::new(tl, br)))
    }
}

impl<'a> From<(&'a Texture<'a>, Rect<f32>)> for DrawRectParams
{
    fn from((tex, rect): (&'a Texture, Rect<f32>)) -> Self
    {
        From::from(([1.0, 1.0, 1.0, 1.0], tex, rect))
    }
}

impl<'a> From<(Color, &'a Texture<'a>, TexCoord, TexCoord)> for DrawRectParams
{
    fn from((col, tex, tl, br): (Color, &'a Texture, TexCoord, TexCoord)) -> Self
    {
        From::from((col, tex, Rect::new(tl, br)))
    }
}

impl<'a> From<(Color, &'a Texture<'a>, Rect<f32>)> for DrawRectParams
{
    fn from((col, tex, rect): (Color, &'a Texture, Rect<f32>)) -> Self
    {
        DrawRectParams{ color: col, tex_id: tex.id, texc: rect }
    }
}
