extern crate khronos;

#[link(name="EGL")]
extern {}

pub use khronos::*;
use std::os::raw::c_void;

pub type EGLNativeDisplayType = *mut c_void;
pub type EGLNativeWindowType = *mut c_void;
pub type EGLNativePixmapType = *mut c_void;
pub type EGLint = khronos_int32_t;

pub type NativeDisplayType = EGLNativeDisplayType;
pub type NativeWindowType = EGLNativeWindowType;
pub type NativePixmapType = EGLNativePixmapType;

include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));
