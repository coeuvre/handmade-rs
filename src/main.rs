#![windows_subsystem = "windows"]

extern crate winapi;

use std::mem::{zeroed, size_of};

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND, RECT, HBITMAP, HGDIOBJ, HDC};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::*;
use winapi::um::winnt::{LPCWCHAR, HANDLE};
use winapi::um::winuser::*;

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

type Error = &'static str;

static mut RUNNING: bool = false;
static mut BITMAP_MEMORY: *mut c_void = 0 as *mut c_void;
static mut BITMAP_HANDLE: HBITMAP = 0 as HBITMAP;
static mut BITMAP_DEVICE_CONTEXT: HDC = 0 as HDC;

fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    let mut bitmap_info = unsafe { zeroed::<BITMAPINFO>() };
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = width;
    bitmap_info.bmiHeader.biHeight = height;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB;
    bitmap_info
}

unsafe fn win32_resize_dib_section(width: i32, height: i32) {
    if BITMAP_HANDLE != 0 as HBITMAP {
        DeleteObject(BITMAP_HANDLE as HGDIOBJ);
    }

    if BITMAP_DEVICE_CONTEXT == 0 as HDC {
        BITMAP_DEVICE_CONTEXT = CreateCompatibleDC(0 as HDC);
    }

    BITMAP_HANDLE = CreateDIBSection(
        BITMAP_DEVICE_CONTEXT,
        &bitmap_info(width, height),
        DIB_RGB_COLORS,
        &mut BITMAP_MEMORY,
        0 as HANDLE,
        0
    );
}

unsafe fn win32_update_window(device_context: HDC, x: i32, y: i32, width: i32, height: i32) {
    StretchDIBits(
        device_context,
        x,
        y,
        width,
        height,
        x,
        y,
        width,
        height,
        BITMAP_MEMORY,
        &bitmap_info(width, height),
        DIB_RGB_COLORS,
        SRCCOPY,
    );
}

unsafe extern "system" fn win32_main_window_proc(
    window: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_SIZE => {
            let mut client_rect = zeroed::<RECT>();
            GetClientRect(window, &mut client_rect);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;
            win32_resize_dib_section(width, height);
        }
        WM_DESTROY => {
            RUNNING = false;
            println!("WM_DESTORY");
        }
        WM_CLOSE => {
            RUNNING = false;
            println!("WM_CLOSE");
        }
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        }
        WM_PAINT => {
            let mut ps = zeroed::<PAINTSTRUCT>();
            let device_context = BeginPaint(window, &mut ps);
            let x = ps.rcPaint.left;
            let y = ps.rcPaint.top;
            let width = ps.rcPaint.right - ps.rcPaint.left;
            let height = ps.rcPaint.bottom - ps.rcPaint.top;
            win32_update_window(device_context, x, y, width, height);
            EndPaint(window, &mut ps);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn run() -> Result<(), Error> {
    let instance = GetModuleHandleW(0 as LPCWCHAR);

    let mut window_class = zeroed::<WNDCLASSW>();

    let class_name = wcstr!("HandmadeHeroWindowClass");

    window_class.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = Some(win32_main_window_proc);
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

    let mut message = zeroed::<MSG>();
    RUNNING = true;
    while RUNNING {
        let result = GetMessageW(&mut message, 0 as HWND, 0, 0);
        if result <= 0 {
            break;
        }

        TranslateMessage(&message);
        DispatchMessageW(&message);
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    unsafe { run() }
}
