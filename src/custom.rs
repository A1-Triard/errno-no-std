use core::fmt::{self, Formatter};

extern "Rust" {
    fn rust_errno() -> i32;
    fn rust_set_errno(errno: i32);
    fn rust_errno_fmt(errno: i32, f: &mut Formatter) -> fmt::Result;
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    unsafe { rust_errno_fmt(e, f) }
}

pub fn errno_raw() -> i32 { 
    unsafe { rust_errno() }
}

pub fn set_errno_raw(e: i32) {
    unsafe { rust_set_errno(e) }
}
