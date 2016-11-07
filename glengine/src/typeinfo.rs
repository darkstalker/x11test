use super::types;

// this implements some kind of manual reflection
pub trait TypeInfo
{
    fn visit_fields<F>(cb: F)
        where F: Fn(&str /* name */, usize /* offset */, usize /* count */, types::GlTypeEnum);
}

macro_rules! impl_typeinfo
{
    ($t:ty, $($field:ident),+) => (
        impl $crate::typeinfo::TypeInfo for $t
        {
            fn visit_fields<F>(cb: F)
                where F: Fn(&str, usize, usize, ::types::GlTypeEnum)
            {
                use $crate::types::{GlType, ElemCount, GlTypeEnum};
                // we need this to extract the type from a struct field
                fn gltype<T: GlType>(_v: &T) -> GlTypeEnum { T::get_gl_type() }
                fn elem_count<T: ElemCount>(_v: &T) -> usize { T::get_elem_count() }

                let tmp: $t = unsafe{ ::std::mem::uninitialized() };
                let start = &tmp as *const _ as usize;
                $(
                    let offset = &tmp.$field as *const _ as usize - start;
                    let count = elem_count(&tmp.$field);
                    let ty = gltype(&tmp.$field);
                    cb(stringify!($field), offset, count, ty);
                )+
                ::std::mem::forget(tmp);
            }
        }
    )
}
