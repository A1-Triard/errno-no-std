use core::fmt::{self, Formatter};
use core::ptr::{null, null_mut};
use core::slice::{self};
use widestring::UStr;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::shared::ntdef::LPWSTR;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::*;

fn errno_fmt_fallback(f: &mut Formatter, e: i32) -> fmt::Result {
    write!(f, "error 0x{:04x}", e as DWORD)
}

fn write_utf16_lossy(f: &mut Formatter, s: &[u16]) -> fmt::Result {
    write!(f, "{}", UStr::from_slice(s).display())
}

struct Buf(LPWSTR);

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0 as LPVOID) };
    }
}

pub fn errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
    let mut buf: LPWSTR = null_mut();
    let msg_len = unsafe { FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        null(),
        e as DWORD,
        0,
        &mut buf as *mut _ as LPWSTR,
        0,
        null_mut()
    ) };
    if msg_len == 0 { return errno_fmt_fallback(f, e); }
    let buf = Buf(buf);
    let msg = unsafe { slice::from_raw_parts(buf.0, msg_len as usize) };
    let msg = msg.trim_end_matches(|c| c == '\r' || c == '\n');
    write_utf16_lossy(f, msg)
}

pub fn errno_raw() -> i32 { 
    (unsafe { GetLastError() }) as i32
}

pub fn set_errno_raw(e: i32) {
    unsafe { SetLastError(e as DWORD) }
}
