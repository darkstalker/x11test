use egl;
use egl::types::*;

use std::mem;
use std::ptr;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

pub type NativeDisplay = NativeDisplayType;
pub type NativeWindow = NativeWindowType;

pub struct Display
{
    egl_disp: EGLDisplay,
    egl_config: EGLConfig,
    gl_context: EGLContext,
}

impl Display
{
    pub fn new(disp: NativeDisplayType) -> Result<Self, &'static str>
    {
        let egl_disp = unsafe { egl::GetDisplay(disp) };
        if egl_disp == egl::NO_DISPLAY
        {
            return Err("error opening EGL display")
        }

        if unsafe { egl::Initialize(egl_disp, ptr::null_mut(), ptr::null_mut()) } == 0
        {
            return Err("error initializing EGL")
        }

        unsafe
        {
            let vendor = CStr::from_ptr(egl::QueryString(egl_disp, egl::VENDOR as EGLint));
            let version = CStr::from_ptr(egl::QueryString(egl_disp, egl::VERSION as EGLint));
            let apis = CStr::from_ptr(egl::QueryString(egl_disp, egl::CLIENT_APIS as EGLint));
            let exts = CStr::from_ptr(egl::QueryString(egl_disp, egl::EXTENSIONS as EGLint));
            println!("EGL vendor: {:?}\nEGL version: {:?}\nEGL apis: {:?}\nEGL extensions: {:?}",
                vendor, version, apis, exts);
        }

        let cfg_attribs = [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::ALPHA_SIZE, 8,
            //egl::DEPTH_SIZE, 24,
            egl::CONFORMANT, egl::OPENGL_ES2_BIT,
            egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
            egl::NONE
        ];
        let configs: [EGLConfig; 1] = unsafe{ mem::zeroed() };
        let mut num_cfg = 0;
        if unsafe { egl::ChooseConfig(egl_disp, cfg_attribs.as_ptr() as _, configs.as_ptr() as *mut _, configs.len() as EGLint, &mut num_cfg) } == 0
        {
            return Err("error choosing EGL config")
        }
        if num_cfg == 0
        {
            return Err("no compatible EGL configs found")
        }

        let ctx_attribs = [
            egl::CONTEXT_CLIENT_VERSION, 2,
            egl::NONE
        ];
        let context = unsafe { egl::CreateContext(egl_disp, configs[0], egl::NO_CONTEXT, ctx_attribs.as_ptr() as _) };
        if context == egl::NO_CONTEXT
        {
            return Err("error creating GL context")
        }

        // we need to bind a context before making any GL call
        if unsafe { egl::MakeCurrent(egl_disp, egl::NO_SURFACE, egl::NO_SURFACE, context) } == 0
        {
            return Err("error binding GL context")
        }

        Ok(Display{
            egl_disp: egl_disp,
            egl_config: configs[0],
            gl_context: context,
        })
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void
    {
        let name_ = CString::new(name).unwrap();
        unsafe { egl::GetProcAddress(name_.as_ptr()) as *const _ }
    }

    pub fn create_window_surface(&self, win: NativeWindowType) -> Result<Surface, &'static str>
    {
        let surface = unsafe { egl::CreateWindowSurface(self.egl_disp, self.egl_config, win, ptr::null()) };
        if surface == egl::NO_SURFACE
        {
            return Err("can't create EGL surface")
        }
        Ok(Surface{ id: surface, disp: self })
    }
}

impl Drop for Display
{
    fn drop(&mut self)
    {
        unsafe
        {
            egl::MakeCurrent(self.egl_disp, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT);
            egl::DestroyContext(self.egl_disp, self.gl_context);
            egl::Terminate(self.egl_disp);
        }
    }
}

pub struct Surface<'a>
{
    id: EGLSurface,
    disp: &'a Display,
}

impl<'a> Surface<'a>
{
    pub fn swap_buffers(&self)
    {
        if unsafe { egl::SwapBuffers(self.disp.egl_disp, self.id) } == 0
        {
            panic!("error on eglSwapBuffers")
        }
    }

    pub fn make_current(&self)
    {
        if unsafe { egl::MakeCurrent(self.disp.egl_disp, self.id, self.id, self.disp.gl_context) } == 0
        {
            panic!("error in eglMakeCurrent")
        }
    }
}

impl<'a> Drop for Surface<'a>
{
    fn drop(&mut self)
    {
        unsafe{ egl::DestroySurface(self.disp.egl_disp, self.id) };
    }
}
