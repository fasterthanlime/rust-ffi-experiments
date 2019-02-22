
extern crate libc_print;

use libc_print::*;
use std::ffi::{CStr};

#[macro_export]
macro_rules! hook {
    ($(fn $real_fn:ident($($v:ident : $t:ty),*) -> $r:ty $body:block)+) => {
        $(
            paste::item! {
                const_cstr! {
                    [<$real_fn __name>] = stringify!($real_fn);
                }

                lazy_static! {
                    pub static ref [<$real_fn __next>]: extern "C" fn ($($v: $t),*) -> $r = unsafe {
                        let sym = libc::dlsym(libc::RTLD_NEXT, [<$real_fn __name>].as_ptr());
                        ::std::mem::transmute(sym)
                    };
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $real_fn ($($v: $t),*) -> $r {
                $body
            }
        )+
    };
}

#[macro_export]
macro_rules! hook_gl {
    ($(fn $real_fn:ident($($v:ident : $t:ty),*) -> $r:ty $body:block)+) => {
        $(
            paste::item! {
                const_cstr! {
                    [<$real_fn __name>] = stringify!($real_fn);
                }

                lazy_static! {
                    pub static ref [<$real_fn __next>]: extern "C" fn ($($v: $t),*) -> $r = unsafe {
                        libc_println!("getting proc address for {}", stringify!([<$real_fn __name>]));
                        let sym = hooks::glXGetProcAddressARB__next([<$real_fn __name>].as_ptr());
                        if sym.is_null() {
                            libc_println!("uh oh, GetProcAddress returned null :(");
                        }
                        ::std::mem::transmute(sym)
                    };
                }
            }

            #[allow(non_snake_case)]
            pub unsafe extern "C" fn $real_fn ($($v: $t),*) -> $r {
                $body
            }
        )+
    };
}

hook! {
    fn glXGetProcAddressARB(symbol: *const libc::c_char) -> *mut libc::c_void {
        if !symbol.is_null() {
            let symbol = CStr::from_ptr(symbol).to_string_lossy().into_owned();
            libc_println!("> glXGetProcAddressARB({})", symbol);

            // if symbol == "glXSwapBuffers" {
            //     return glXSwapBuffers as *mut libc::c_void
            // }
        }

        glXGetProcAddressARB__next(symbol)
    }
}
