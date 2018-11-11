#![windows_subsystem = "windows"]

extern crate winapi;

use winapi::shared::windef::HWND;
use winapi::um::winuser::MessageBoxW;
use winapi::um::winuser::{MB_ICONINFORMATION, MB_OK};

macro_rules! wstr {
    ($s:expr) => {{
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($s)
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<u16>>()
    }};
}

unsafe fn run() {
    MessageBoxW(
        0 as HWND,
        wstr!("This is Handmade Hero").as_ptr(),
        wstr!("Handmade Hero").as_ptr(),
        MB_OK | MB_ICONINFORMATION,
    );
}

fn main() {
    unsafe { run() }
}
