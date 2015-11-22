extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate user32;

use std::mem;

use winapi::minwindef::*;
use winapi::windef::*;
use winapi::winuser::*;
use winapi::winnt::LPCWSTR;
use winapi::wingdi::{BLACKNESS, WHITENESS};

use gdi32::PatBlt;

use kernel32::{GetConsoleWindow, GetModuleHandleW};

use user32::*;

macro_rules! wstr {
    ($str:expr) => ({
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let wstr : Vec<u16> = OsStr::new($str).encode_wide()
            .chain(Some(0).into_iter()).collect();
        wstr.as_ptr()
    });
}

unsafe extern "system" fn main_window_callback(window: HWND, message: UINT,
                                               wparam: WPARAM, lparam: LPARAM)
                                               -> LRESULT {
    match message {
        WM_SIZE => {
            println!("WM_SIZE");
        },
        WM_DESTROY => {
            println!("WM_DESTROY");
        },
        WM_CLOSE => {
            println!("WM_CLOSE");
        },
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        },
        WM_PAINT => {
            let mut paint: PAINTSTRUCT = mem::uninitialized();
            let device_context = BeginPaint(window, &mut paint);
            let x = paint.rcPaint.left;
            let y = paint.rcPaint.top;
            let width = paint.rcPaint.right - paint.rcPaint.left;
            let height = paint.rcPaint.bottom - paint.rcPaint.top;
            static mut operation: DWORD = WHITENESS;
            PatBlt(device_context, x, y, width, height, operation);
            if operation == WHITENESS {
                operation = BLACKNESS;
            } else {
                operation = WHITENESS;
            }
            EndPaint(window, &paint);
        },
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn win_main(instance: HINSTANCE) {
    let mut window_class: WNDCLASSW = mem::zeroed();
    // TODO(coeuvre): Check if HREDRAW/VREDRAW/OWNDC still matter
    window_class.lpfnWndProc = Some(main_window_callback);
    window_class.hInstance = instance;
    window_class.lpszClassName = wstr!("HandmadeHeroWindowClass");

    if RegisterClassW(&window_class) != 0 {
        let window_handle = CreateWindowExW(0, window_class.lpszClassName,
                                            wstr!("Handmade Hero"),
                                            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                                            CW_USEDEFAULT, CW_USEDEFAULT,
                                            CW_USEDEFAULT, CW_USEDEFAULT,
                                            0 as HWND, 0 as HMENU,
                                            instance, 0 as LPVOID);
        if window_handle != 0 as HWND {
            let mut message: MSG = mem::uninitialized();

            loop {
                let message_result = GetMessageW(&mut message, 0 as HWND, 0, 0);
                if message_result > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                } else {
                    break;
                }
            }
        } else {
            // TODO(coeuvre): Logging
        }
    } else {
        // TODO(coeuvre): Logging
    }
}

fn main() {
    unsafe {
        // Hide the console window
        let console = GetConsoleWindow();
        if console != 0 as HWND {
            ShowWindow(console, winapi::SW_HIDE);
        }

        let instance = GetModuleHandleW(0 as LPCWSTR);

        win_main(instance);
    }
}
