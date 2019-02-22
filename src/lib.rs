mod hooks;

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
    libc_println!("libcapsule starting up, hi!");
}

hook! {
    fn dlopen(filename: *mut libc::c_char, flags: libc::c_int) -> *mut libc::c_void {
        if filename.is_null() {
            libc_println!("> dlopen(NULL, {})", flags);
        } else {
            let name = CStr::from_ptr(filename).to_string_lossy().into_owned();
            libc_println!("> dlopen({}, {})", name, flags);

            if name == "libGL.so.1" {
                // load symbols into our space
                dlopen__next(filename, libc::RTLD_NOW|libc::RTLD_GLOBAL);

                // then return our own space
                return dlopen__next(std::ptr::null_mut(), libc::RTLD_NOW|libc::RTLD_LOCAL);
            }
        }

        dlopen__next(filename, flags)
    }
}

hook_gl! {
    fn glXSwapBuffers(display: *mut libc::c_void, drawable: *mut libc::c_void) -> () {
        libc_println!("> glXSwapBuffers!");
        glXSwapBuffers__next(display, drawable)
    }
}
