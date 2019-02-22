#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate const_cstr;
extern crate ctor;
extern crate libc_print;
extern crate paste;

use ctor::*;
use libc_print::*;

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

hook! {
    fn free(p: *mut libc::c_void) -> () {
        libc_println!("> free({:?})", p);
        free__next(p)
    }

    fn malloc(s: libc::size_t) -> *mut libc::c_void {
        libc_println!("> malloc({})", s);
        malloc__next(s)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
