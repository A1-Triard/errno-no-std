//! **Crate features**
//!
//! * `"std"`
//! Enabled by default. Disable to make the library `#![no_std]`.

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::unnecessary_cast)]

#![cfg_attr(not(feature="std"), no_std)]
#[cfg(feature="std")]
extern crate core;

#[cfg(test)]
extern crate std;

#[cfg(all(not(windows), not(custom_errno)))]
mod posix;
#[cfg(all(not(windows), not(custom_errno)))]
use posix::*;

#[cfg(all(windows, not(custom_errno)))]
mod winapi;
#[cfg(all(windows, not(custom_errno)))]
use crate::winapi::*;

#[cfg(custom_errno)]
mod custom;
#[cfg(custom_errno)]
use custom::*;

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

#[cfg(test)]
mod test {
    use crate::*;
    use copy_from_str::CopyFromStrExt;
    use core::fmt::{self, Write};
    use core::str::{self};
    use quickcheck_macros::quickcheck;

    #[cfg(custom_errno)]
    mod custom_errno {
        use std::cell::Cell;
        use std::fmt::{self, Formatter};
        use std::thread_local;

        thread_local! {
            static ERRNO: Cell<i32> = Cell::new(0);
        }

        #[no_mangle]
        extern "Rust" fn rust_errno() -> i32 { ERRNO.with(|x| x.get()) }

        #[no_mangle]
        extern "Rust" fn rust_set_errno(e: i32) {
            ERRNO.with(|x| x.set(e))
        }

        #[no_mangle]
        extern "Rust" fn rust_errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
            write!(f, "Error {}", e)
        }
    }

    struct Buf<'a> {
        s: &'a mut str,
        len: usize,
    }

    impl<'a> Write for Buf<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let advanced_len = self.len.checked_add(s.len()).unwrap();
            self.s[self.len .. advanced_len].copy_from_str(s);
            self.len = advanced_len;
            Ok(())
        }
    }

    #[quickcheck]
    fn errno_after_set_errno(e: i32) -> bool {
        set_errno(Errno(e));
        errno() == Errno(e)
    }

    #[quickcheck]
    fn error_display(e: i32) -> bool {
        let mut buf = [0; 1024];
        let buf = str::from_utf8_mut(&mut buf[..]).unwrap();
        let mut buf = Buf { s: buf, len: 0 };
        write!(&mut buf, "{}", Errno(e)).unwrap();
        let res = &buf.s[.. buf.len];
        if res.len() <= 5 { return false; }
        let end = res.chars().last().unwrap();
        end.is_ascii_alphanumeric() && !end.is_whitespace() || end == '.'
    }
}

#[cfg(all(test, not(windows), not(target_os="macos"), not(custom_errno)))]
mod test_localization {
    use crate::*;
    use copy_from_str::CopyFromStrExt;
    use core::fmt::{self, Write};
    use core::str::{self};
    use libc::{LC_ALL, EACCES, setlocale};

    struct Buf<'a> {
        s: &'a mut str,
        len: usize,
    }

    impl<'a> Write for Buf<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let advanced_len = self.len.checked_add(s.len()).unwrap();
            self.s[self.len .. advanced_len].copy_from_str(s);
            self.len = advanced_len;
            Ok(())
        }
    }

    struct DefaultLocale;

    impl Drop for DefaultLocale {
        fn drop(&mut self) {
            unsafe { setlocale(LC_ALL, b"\0".as_ptr() as *const _); }
        }
    }

    #[test]
    fn localized_messages() {
        let _default_locale = DefaultLocale;
        let locales: &[&'static [u8]] = &[
            b"en_US.UTF-8\0",
            b"ja_JP.EUC-JP\0",
            b"uk_UA.KOI8-U\0",
            b"uk_UA.UTF-8\0"
        ];
        for &locale in locales {
            unsafe { setlocale(LC_ALL, locale.as_ptr() as *const _) };
            let msg = match locale.split(|&b| b == b'.').next().unwrap() {
                b"en_US" => "Permission denied",
                b"ja_JP" => "許可がありません",
                b"uk_UA" => "Відмовлено у доступі",
                _ => panic!("message?"),
            };
            let mut buf = [0; 1024];
            let buf = str::from_utf8_mut(&mut buf[..]).unwrap();
            let mut buf = Buf { s: buf, len: 0 };
            write!(&mut buf, "{}", Errno(EACCES)).unwrap();
            let res = &buf.s[.. buf.len];
            assert_eq!(res, msg);
        }
    }
}
