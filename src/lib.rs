#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_cstr;
extern crate ctor;
extern crate libc_print;
extern crate paste;

use ctor::*;
use libc_print::*;
use std::ffi::{CStr};

#[ctor]
fn ctor() {
    libc_println!("> ctor()");
}

#[dtor]
fn dtor() {
    libc_println!("> dtor()");
}

macro_rules! hook {
    ($(fn $real_fn:ident($($v:ident : $t:ty),*) -> $r:ty $body:block)+) => {
        $(
            paste::item! {
                const_cstr! {
                    [<$real_fn __name>] = stringify!($real_fn);
                }

                lazy_static! {
                    static ref [<$real_fn __next>]: extern "C" fn ($($v: $t),*) -> $r = unsafe {
                        let sym = libc::dlsym(libc::RTLD_NEXT, [<$real_fn __name>].as_ptr());
                        ::std::mem::transmute(sym)
                    };
                }
            }

            #[no_mangle]
            pub unsafe extern fn $real_fn ($($v: $t),*) -> $r {
                $body
            }
        )+
    };
}

// hook! {
//     fn free(p: *mut libc::c_void) -> () {
//         libc_println!("> free({:?})", p);
//         free__next(p)
//     }

//     fn malloc(s: libc::size_t) -> *mut libc::c_void {
//         libc_println!("> malloc({})", s);
//         malloc__next(s)
//     }
// }

hook! {
    fn dlopen(filename: *mut libc::c_char, flags: libc::c_int) -> *mut libc::c_void {
        if filename.is_null() {
            libc_println!("> dlopen(NULL, {})", flags);
        } else {
            let name = CStr::from_ptr(filename).to_string_lossy().into_owned();
            libc_println!("> dlopen({}, {})", name, flags);

            if name == "libGL.so.1" {
                libc_println!("> loading OpenGL!");
                // load symbols into our space
                dlopen__next(filename, libc::RTLD_NOW|libc::RTLD_GLOBAL);

                // then return our own space
                return dlopen__next(std::ptr::null_mut(), libc::RTLD_NOW|libc::RTLD_LOCAL);
            }
        }

        dlopen__next(filename, flags)
    }

    fn glXGetProcAddressARB(symbol: *mut libc::c_char) -> *mut libc::c_void {
        if !symbol.is_null() {
            let symbol = CStr::from_ptr(symbol).to_string_lossy().into_owned();
            libc_println!("> glXGetProcAddressARB({})", symbol);
        }

        glXGetProcAddressARB__next(symbol)
    }

    fn glXSwapBuffers(display: *mut libc::c_void, drawable: *mut libc::c_void) -> () {
        libc_println!("> glXSwapBuffers!");
        glXSwapBuffers__next(display, drawable)
    }
}

