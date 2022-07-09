use ddnet_base::ColorRGBA;

pub use self::ffi::*;

#[repr(transparent)]
pub struct FCommandCallback(pub extern "C" fn(&IConsole_IResult, UserPtr));

unsafe impl cxx::ExternType for FCommandCallback {
    type Id = cxx::type_id!("IConsole_FCommandCallback");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("base/rust.h");
        include!("engine/console.h");
        include!("engine/rust.h");

        type ColorRGBA = ddnet_base::ColorRGBA;
        type StrRef<'a> = ddnet_base::StrRef<'a>;
        type UserPtr = ddnet_base::UserPtr;
        type IConsole_FCommandCallback = super::FCommandCallback;

        type IConsole_IResult;
        type IConsole;

        pub fn Register(
            self: Pin<&mut IConsole>,
            pName: StrRef<'_>,
            pParams: StrRef<'_>,
            Flags: i32,
            pfnFunc: IConsole_FCommandCallback,
            pUser: UserPtr,
            pHelp: StrRef<'_>,
        );
        pub fn Print(
            self: &IConsole,
            Level: i32,
            pFrom: StrRef<'_>,
            pStr: StrRef<'_>,
            PrintColor: ColorRGBA,
        );
    }
}

#[allow(non_upper_case_globals)]
pub const gs_ConsoleDefaultColor: ColorRGBA = ColorRGBA {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

#[allow(non_upper_case_globals)]
pub const IConsole_OUTPUT_LEVEL_STANDARD: i32 = 0;
#[allow(non_upper_case_globals)]
pub const IConsole_OUTPUT_LEVEL_ADDINFO: i32 = 1;
#[allow(non_upper_case_globals)]
pub const IConsole_OUTPUT_LEVEL_DEBUG: i32 = 2;
