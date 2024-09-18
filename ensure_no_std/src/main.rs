#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }

    #[no_mangle]
    extern "C" fn rust_eh_personality() { }
}

use errno_no_std::{Errno, errno, set_errno};
#[cfg(not(windows))]
use libc::EINVAL;
#[cfg(windows)]
use winapi::shared::winerror::ERROR_ACCESS_DENIED;

#[cfg(not(windows))]
const ERR: i32 = EINVAL;

#[cfg(windows)]
const ERR: i32 = ERROR_ACCESS_DENIED as i32;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    set_errno(Errno(ERR));
    assert_eq!(errno().0, ERR);
    0
}
