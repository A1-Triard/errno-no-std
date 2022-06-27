use core::fmt::{self, Formatter};
use core::mem::transmute;
use core::ptr::null;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct CustomErrno {
    pub get: fn() -> i32,
    pub set: fn(errno: i32),
    pub fmt: fn(errno: i32, f: &mut Formatter) -> fmt::Result,
}

static CUSTOM_ERRNO: AtomicUsize = AtomicUsize::new({
    let no_custom_errno: *const CustomErrno = null();
    unsafe { transmute(no_custom_errno) }
});

/// Allows an application compiling and running
/// in a heavy no-standard environment
/// specify a method to work with errno.
///
/// Only zero-sized `CustomErrno` implementations are allowed
pub fn set_custom_errno(custom_errno: &'static CustomErrno) {
    let custom_errno: *const CustomErrno = custom_errno as _;
    let custom_errno = unsafe { transmute(custom_errno) };
    CUSTOM_ERRNO.store(custom_errno, Ordering::Relaxed);
}

fn get_custom_errno() -> &'static CustomErrno {
    let custom_errno = CUSTOM_ERRNO.load(Ordering::Relaxed);
    let custom_errno: *const CustomErrno = unsafe { transmute(custom_errno) };
    assert!(!custom_errno.is_null(), "custom errno is not set");
    unsafe { &*custom_errno }
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    (get_custom_errno().fmt)(e, f)
}

pub fn errno_raw() -> i32 { 
    (get_custom_errno().get)()
}

pub fn set_errno_raw(e: i32) {
    (get_custom_errno().set)(e)
}
