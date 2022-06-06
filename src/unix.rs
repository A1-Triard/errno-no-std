use core::fmt::{self, Formatter};
use core::mem::{MaybeUninit, transmute};
use core::slice::{self};
use core::str::{self};
use errno_sys::errno_location;
use libc::{CODESET, E2BIG, c_char, c_int, nl_langinfo, strlen};
use libc::{iconv, iconv_open, iconv_close, iconv_t};

extern "C" {
    // Avoid GNU strerror_r on Linux and Newlib systems
    #[cfg_attr(any(target_os = "linux", target_env = "newlib"), link_name = "__xpg_strerror_r")]
    fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: libc::size_t) -> c_int;
}

fn write_byte(f: &mut Formatter, c: u8) -> fmt::Result {
    write!(f, "\\x{:02X}", c)
}

fn write_fallback(f: &mut Formatter, s: &[u8]) -> fmt::Result {
    for &c in s {
        write_byte(f, c)?;
    }
    Ok(())
}

fn write_utf8_lossy(f: &mut Formatter, mut s: &[u8]) -> fmt::Result {
    loop {
        match str::from_utf8(s) {
            Ok(valid) => {
                write!(f, "{}", valid)?;
                break Ok(());
            }
            Err(error) => {
                let (valid, after_valid) = s.split_at(error.valid_up_to());
                let valid = unsafe { str::from_utf8_unchecked(valid) };
                write!(f, "{}", valid)?;
                let invalid_len = error.error_len().unwrap_or(after_valid.len());
                let (invalid, tail) = after_valid.split_at(invalid_len);
                write_fallback(f, invalid)?;
                s = tail;
            }
        }
    }
}

struct Iconv(iconv_t);

impl Drop for Iconv {
    fn drop(&mut self) {
        let iconv_close_res = unsafe { iconv_close(self.0) };
        debug_assert_eq!(iconv_close_res, 0);
    }
}

fn localized_msg_fmt(msg: &[u8], f: &mut Formatter) -> fmt::Result {
    let nl = unsafe {
        let nl = nl_langinfo(CODESET) as *const c_char;
        slice::from_raw_parts(nl as *const u8, strlen(nl) + 1)
    };
    if nl == b"UTF-8\0" {
        return write_utf8_lossy(f, msg);
    }
    let c = unsafe { iconv_open(b"UTF-8\0".as_ptr() as _, nl.as_ptr() as _) };
    if c as usize == (-1isize) as usize {
        return write_fallback(f, msg);
    }
    let c = Iconv(c);
    let mut msg_ptr = msg.as_ptr() as *const c_char as *mut c_char;
    let mut msg_len = msg.len();
    let mut uni_buf: [MaybeUninit<u8>; 128] = unsafe { MaybeUninit::uninit().assume_init() };
    loop {
        let mut uni_buf_ptr = uni_buf.as_mut_ptr() as *mut c_char;
        let mut uni_buf_len = uni_buf.len();
        let iconv_res: isize = unsafe { transmute(iconv(
            c.0,
            (&mut msg_ptr) as *mut _,
            (&mut msg_len) as *mut _,
            (&mut uni_buf_ptr) as *mut _,
            (&mut uni_buf_len) as *mut _
        )) };
        let iconv_ok = if iconv_res == -1 {
            if errno_raw() == E2BIG { None } else { Some(false) }
        } else {
            Some(true)
        };
        let uni_len = uni_buf.len() - uni_buf_len;
        let uni = &uni_buf[.. uni_len];
        let uni = unsafe { str::from_utf8_unchecked(transmute(uni)) };
        write!(f, "{}", uni)?;
        match iconv_ok {
            Some(true) => {
                debug_assert_eq!(msg_len, 0);
                return Ok(());
            },
            Some(false) => {
                debug_assert!(msg_len > 0);
                write_byte(f, msg[msg.len() - msg_len])?;
                msg_ptr = unsafe { msg_ptr.add(1) };
                msg_len -= 1;
            },
            None => { }
        }
    }
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    // 128 bytes should be long enough for all error messages
    const BUF_SIZE: usize = 128;

    let mut buf: [c_char; BUF_SIZE] = [0; BUF_SIZE];
    if unsafe { strerror_r(e, buf.as_mut_ptr(), BUF_SIZE) } < 0 {
        return Err(fmt::Error);
    }

    let msg = unsafe { slice::from_raw_parts(buf.as_ptr().cast::<u8>(), strlen(buf.as_ptr())) };

    localized_msg_fmt(msg, f)
}

pub fn errno_raw() -> i32 { 
    (unsafe { *errno_location() }) as i32
}

pub fn set_errno_raw(e: i32) {
    unsafe {
        *errno_location() = e;
    }
}

#[cfg(all(test, not(target_os="macos")))]
mod test {
    use copy_from_str::CopyFromStrExt;
    use core::fmt::{self, Display, Formatter, Write};
    use core::str::{self};
    use libc::{LC_ALL, setlocale};
    use crate::unix::localized_msg_fmt;

    struct DefaultLocale;

    impl Drop for DefaultLocale {
        fn drop(&mut self) {
            unsafe { setlocale(LC_ALL, b"\0".as_ptr() as *const _); }
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

    struct LocalizedStr(&'static [u8]);

    impl Display for LocalizedStr {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            localized_msg_fmt(self.0, f)
        }
    }

    #[test]
    fn localized_msg_fmt_invalid_non_utf8_encoding() {
        let _default_locale = DefaultLocale;
        unsafe { setlocale(LC_ALL, "ja_JP.EUC-JP\0".as_ptr() as *const _) };
        let mut buf = [0; 1024];
        let buf = str::from_utf8_mut(&mut buf[..]).unwrap();
        let mut buf = Buf { s: buf, len: 0 };
        write!(&mut buf, "{}", LocalizedStr(b"\x8E\xA1\x8E\x69")).unwrap();
        let res = &buf.s[.. buf.len];
        assert_eq!(res, "｡\\x8Ei");
    }
}
