//! **Crate features**
//!
//! * `"std"`
//! Enabled by default. Disable to make the library `#![no_std]`.

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![cfg_attr(not(feature="std"), no_std)]
#[cfg(feature="std")]
extern crate core;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::*;

use core::fmt::{self, Formatter};
#[cfg(feature="std")]
use std::error::Error;
#[cfg(feature="std")]
use std::io::{self};

/// Wraps a platform-specific error code.
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Errno(pub i32);

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        errno_fmt(self.0, f)
    }
}

#[cfg(feature="std")]
impl Error for Errno { }

#[cfg(feature="std")]
impl From<Errno> for io::Error {
    fn from(e: Errno) -> Self {
        io::Error::from_raw_os_error(e.0)
    }
}

/// Returns the platform-specific value of `errno`.
pub fn errno() -> Errno { Errno(errno_raw()) }

/// Sets the platform-specific value of `errno`.
pub fn set_errno(err: Errno) { set_errno_raw(err.0) }

/*
#[test]
fn it_works() {
    let x = errno();
    set_errno(x);
}

#[cfg(feature = "std")]
#[test]
fn it_works_with_to_string() {
    let x = errno();
    let _ = x.to_string();
}

#[cfg(feature = "std")]
#[test]
fn check_description() {
    let expect = if cfg!(windows) {
        "Incorrect function."
    } else if cfg!(target_os = "illumos") {
        "Not owner"
    } else if cfg!(target_os = "wasi") {
        "Argument list too long"
    } else if cfg!(target_os = "haiku") {
        "Operation not allowed"
    } else {
        "Operation not permitted"
    };

    let errno_code = if cfg!(target_os = "haiku") { -2147483633 } else { 1 };
    set_errno(Errno(errno_code));

    assert_eq!(errno().to_string(), expect);
    assert_eq!(
        format!("{:?}", errno()),
        format!("Errno {{ code: {}, description: Some({:?}) }}", errno_code, expect));
}

#[cfg(feature = "std")]
#[test]
fn check_error_into_errno() {
    const ERROR_CODE: i32 = 1;

    let error = io::Error::from_raw_os_error(ERROR_CODE);
    let new_error: io::Error = Errno(ERROR_CODE).into();
    assert_eq!(error.kind(), new_error.kind());
}
*/
