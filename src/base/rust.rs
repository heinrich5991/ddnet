use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_char;

#[repr(transparent)]
pub struct UserPtr(*mut ());

unsafe impl cxx::ExternType for UserPtr {
    type Id = cxx::type_id!("UserPtr");
    type Kind = cxx::kind::Trivial;
}

impl UserPtr {
    pub unsafe fn cast<T>(&self) -> &T {
        &*(self.0 as *const _)
    }
}

impl<'a, T> From<&'a T> for UserPtr {
    fn from(t: &'a T) -> UserPtr {
        UserPtr(t as *const _ as *mut _)
    }
}

#[repr(transparent)]
pub struct StrRef<'a>(*const c_char, PhantomData<&'a ()>);

unsafe impl<'a> cxx::ExternType for StrRef<'a> {
    type Id = cxx::type_id!("StrRef");
    type Kind = cxx::kind::Trivial;
}

impl<'a> From<&'a CStr> for StrRef<'a> {
    fn from(s: &'a CStr) -> StrRef<'a> {
        StrRef(s.to_bytes_with_nul().as_ptr() as *const _, PhantomData)
    }
}

#[macro_export]
macro_rules! s {
    ($str:expr) => {
        ::ddnet_base::StrRef::from(
            ::std::ffi::CStr::from_bytes_with_nul(::std::concat!($str, "\0").as_bytes()).unwrap(),
        )
    };
}
