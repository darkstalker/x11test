use gl;

//types from GLES2, intentionally omitted Fixed
#[derive(Debug)]
#[repr(u32)]
pub enum GlTypeEnum
{
    Byte            = gl::BYTE,
    UnsignedByte    = gl::UNSIGNED_BYTE,
    Short           = gl::SHORT,
    UnsignedShort   = gl::UNSIGNED_SHORT,
    Float           = gl::FLOAT,
}

// mapping from rust type => opengl type enum
pub trait GlType
{
    fn get_gl_type() -> GlTypeEnum;
}

impl GlType for i8  { fn get_gl_type() -> GlTypeEnum { GlTypeEnum::Byte } }
impl GlType for u8  { fn get_gl_type() -> GlTypeEnum { GlTypeEnum::UnsignedByte } }
impl GlType for i16 { fn get_gl_type() -> GlTypeEnum { GlTypeEnum::Short } }
impl GlType for u16 { fn get_gl_type() -> GlTypeEnum { GlTypeEnum::UnsignedShort } }
impl GlType for f32 { fn get_gl_type() -> GlTypeEnum { GlTypeEnum::Float } }

impl<T> GlType for [T; 1] where T: GlType { fn get_gl_type() -> GlTypeEnum { T::get_gl_type() } }
impl<T> GlType for [T; 2] where T: GlType { fn get_gl_type() -> GlTypeEnum { T::get_gl_type() } }
impl<T> GlType for [T; 3] where T: GlType { fn get_gl_type() -> GlTypeEnum { T::get_gl_type() } }
impl<T> GlType for [T; 4] where T: GlType { fn get_gl_type() -> GlTypeEnum { T::get_gl_type() } }

pub trait ElemCount
{
    fn get_elem_count() -> usize;
}

impl ElemCount for i8  { fn get_elem_count() -> usize { 1 } }
impl ElemCount for u8  { fn get_elem_count() -> usize { 1 } }
impl ElemCount for i16 { fn get_elem_count() -> usize { 1 } }
impl ElemCount for u16 { fn get_elem_count() -> usize { 1 } }
impl ElemCount for f32 { fn get_elem_count() -> usize { 1 } }

impl<T> ElemCount for [T; 1] { fn get_elem_count() -> usize { 1 } }
impl<T> ElemCount for [T; 2] { fn get_elem_count() -> usize { 2 } }
impl<T> ElemCount for [T; 3] { fn get_elem_count() -> usize { 3 } }
impl<T> ElemCount for [T; 4] { fn get_elem_count() -> usize { 4 } }
