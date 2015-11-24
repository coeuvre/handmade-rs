extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate user32;

use std::mem;

use winapi::minwindef::*;
use winapi::windef::*;
use winapi::winuser::*;
use winapi::winnt::{LPCWSTR, HANDLE};
use winapi::wingdi::*;

use gdi32::*;

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

// TODO(coeuvre): This is a global for now.
static mut RUNNING: bool = true;

static mut BITMAP_INFO: BITMAPINFO = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
        biSize: 0,
        biWidth: 0,
        biHeight: 0,
        biPlanes: 0,
        biBitCount: 0,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    },
    bmiColors: [],
};
static mut BITMAP_MEMORY: LPVOID = 0 as LPVOID;
static mut BITMAP_HANDLE: HBITMAP = 0 as HBITMAP;
static mut BITMAP_DEVICE_CONTEXT: HDC = 0 as HDC;

unsafe fn win32_resize_dib_section(width: i32, height: i32) {
    // TODO(coeuvre): Bulletproof this.
    // Maybe don't free first, free after, then free first if that fails.

    if BITMAP_HANDLE != 0 as HBITMAP {
        DeleteObject(BITMAP_HANDLE as HGDIOBJ);
    }

    if BITMAP_DEVICE_CONTEXT == 0 as HDC {
        // TODO(coeuvre): Should we recreate these under certain special circumstances
        BITMAP_DEVICE_CONTEXT = CreateCompatibleDC(0 as HDC);
    }

    // TODO(coeuvre): Free our DIBSection
    BITMAP_INFO.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    BITMAP_INFO.bmiHeader.biWidth = width;
    BITMAP_INFO.bmiHeader.biHeight = height;
    BITMAP_INFO.bmiHeader.biPlanes = 1;
    BITMAP_INFO.bmiHeader.biBitCount = 32;
    BITMAP_INFO.bmiHeader.biCompression = BI_RGB;

    // TODO(coeuvre): Based on ssylvan's suggestion, maybe we can just
    // allocate this ourselves?

    BITMAP_HANDLE = CreateDIBSection(BITMAP_DEVICE_CONTEXT, &BITMAP_INFO,
                                     DIB_RGB_COLORS, &mut BITMAP_MEMORY,
                                     0 as HANDLE, 0);
}

unsafe fn win32_update_window(device_context: HDC,
                              x: i32, y: i32,
                              width: i32, height: i32) {
    StretchDIBits(device_context,
                  x, y, width, height,
                  x, y, width, height,
                  BITMAP_MEMORY, &BITMAP_INFO, DIB_RGB_COLORS, SRCCOPY);
}

unsafe extern "system" fn win32_main_window_callback(window: HWND, message: UINT,
                                                     wparam: WPARAM, lparam: LPARAM)
                                                     -> LRESULT {
    match message {
        WM_SIZE => {
            let mut client_rect = mem::uninitialized();
            GetClientRect(window, &mut client_rect);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;
            win32_resize_dib_section(width, height);
            println!("WM_SIZE");
        },
        WM_CLOSE => {
            // TODO(coeuvre): Handle this with a message to the user?
            RUNNING = false;
        },
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        },
        WM_DESTROY => {
            // TODO(coeuvre): Handle this as an error - recreate window?
            RUNNING = false;
        },
        WM_PAINT => {
            let mut paint = mem::uninitialized();
            let device_context = BeginPaint(window, &mut paint);
            let x = paint.rcPaint.left;
            let y = paint.rcPaint.top;
            let width = paint.rcPaint.right - paint.rcPaint.left;
            let height = paint.rcPaint.bottom - paint.rcPaint.top;
            win32_update_window(device_context, x, y, width, height);
            EndPaint(window, &paint);
        },
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn win_main(instance: HINSTANCE) {
    let mut window_class: WNDCLASSW = mem::zeroed();
    // TODO(coeuvre): Check if HREDRAW/VREDRAW/OWNDC still matter
    window_class.lpfnWndProc = Some(win32_main_window_callback);
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
            let mut message = mem::uninitialized();

            while RUNNING {
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
