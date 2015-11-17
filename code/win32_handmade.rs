extern crate winapi;
extern crate user32;

use std::ptr;
use std::ffi::CString;

use winapi::winuser::*;
use user32::*;

fn main() {
    unsafe {
        MessageBoxA(ptr::null_mut(),
                    CString::new("This is Handmade Hero.").unwrap().as_ptr(),
                    CString::new("Handmade Hero").unwrap().as_ptr(),
                    MB_OK | MB_ICONINFORMATION);
    }
}
