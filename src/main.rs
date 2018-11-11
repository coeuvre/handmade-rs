#![windows_subsystem = "windows"]

extern crate winapi;

type Error = &'static str;

use winapi::shared::minwindef::{LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winnt::LPCWCHAR;
use winapi::um::winuser::*;
use winapi::um::wingdi::*;

macro_rules! wcstr {
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

unsafe extern "system" fn window_proc(
    window: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_SIZE => {
            println!("WM_SIZE");
        }
        WM_DESTROY => {
            println!("WM_DESTORY");
        }
        WM_CLOSE => {
            println!("WM_CLOSE");
        }
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
            let hdc = BeginPaint(window, &mut ps);
            let x = ps.rcPaint.left;
            let y = ps.rcPaint.top;
            let width = ps.rcPaint.right - ps.rcPaint.left;
            let height = ps.rcPaint.bottom - ps.rcPaint.top;
            static mut OPERATION: u32 = WHITENESS;
            if OPERATION == WHITENESS {
                OPERATION = BLACKNESS;
            } else {
                OPERATION = WHITENESS;
            }
            PatBlt(hdc, x, y, width, height, OPERATION);
            EndPaint(window, &mut ps);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn run() -> Result<(), Error> {
    let instance = GetModuleHandleW(0 as LPCWCHAR);

    let mut window_class = std::mem::zeroed::<WNDCLASSW>();

    let class_name = wcstr!("HandmadeHeroWindowClass");

    window_class.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = Some(window_proc);
    window_class.hInstance = instance;
    window_class.lpszClassName = class_name.as_ptr();

    let result = RegisterClassW(&window_class);
    if result == 0 {
        return Err("Failed to register class");
    }

    let window_name = wcstr!("Handmade Hero");
    let window = CreateWindowExW(
        0,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        0 as HWND,
        0 as HMENU,
        instance,
        0 as LPVOID,
    );
    if window == 0 as HWND {
        return Err("Failed to create window");
    }

    let mut message = std::mem::zeroed::<MSG>();
    'game: loop {
        let result = GetMessageW(&mut message, 0 as HWND, 0, 0);
        if result <= 0 {
            break 'game;
        }

        TranslateMessage(&message);
        DispatchMessageW(&message);
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    unsafe { run() }
}
