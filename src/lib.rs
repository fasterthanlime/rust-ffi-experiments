#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_cstr;
extern crate ctor;
extern crate libc_print;

use ctor::*;
use libc_print::*;
use std::mem;

#[ctor]
fn ctor() {
    libc_println!("[ctor called]");
}

#[dtor]
fn dtor() {
    libc_println!("[dtor called]");
}

const_cstr! {
    malloc_name = "malloc";
}

lazy_static! {
    static ref _malloc: extern "C" fn(s: libc::size_t) -> *mut libc::c_void =
        unsafe { mem::transmute(libc::dlsym(libc::RTLD_NEXT, malloc_name.as_ptr())) };
}

#[no_mangle]
pub unsafe extern "C" fn malloc(s: libc::size_t) -> *mut libc::c_void {
    libc_println!("[allocating {} bytes]", s);
    _malloc(s)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// stolen^Wadapted from https://github.com/geofft/redhook
macro_rules! hook {
    (fn $real_fn:ident($($v:ident : $t:ty),*) -> $r:ty => $hook_fn:ident $body:block) => {
        const_cstr! {
            foobar = stringify!(real_fn);
        }

        #[allow(non_camel_case_types)]
        pub struct $real_fn {__private_field: ()}
        #[allow(non_uppercase_globals)]
        static $real_fn: $real_fn = $real_fn {__private_field: ()};

        lazy_static! {
            static ref REAL: unsafe extern fn ($($v: $t),*) -> $r = unsafe {
                ::std::mem::transmute(libc::dlsym(libc::RTLD_NEXT, foobar.as_ptr()))
            };
        }

        impl $real_fn {
            fn get(&self) -> unsafe extern fn ($($v: $t),*) -> $r {
                use ::std::sync::{Once, ONCE_INIT};

                static mut REAL: *mut ::std::ffi::c_void = ::std::ptr::null_mut();
                static mut ONCE: Once = ONCE_INIT;

                unsafe {
                    ONCE.call_once(|| {
                        REAL = libc::dlsym(libc::RTLD_NEXT, foobar.as_ptr())
                    });
                    ::std::mem::transmute(REAL)
                }
            }

            #[no_mangle]
            pub unsafe extern fn $real_fn ($($v: $t),*) -> $r {
                {
                    ::std::panic::catch_unwind(|| $hook_fn($($v),*)).ok()
                }.unwrap_or_else(|| $real_fn.get()($($v),*))
                // so, if hook_fn crashes we still call real_fn, huh!
            }
        }

        pub unsafe fn $hook_fn($($v: $t),*) -> $r {
            $body
        }
    };
}

macro_rules! real {
    ($real_fn:ident) => {
        $real_fn.get()
    };
}

hook! {
    fn free(p: *mut libc::c_void) -> () => my_free {
        libc_println!("calling free!");
        real!(free)(p)
    }
}
