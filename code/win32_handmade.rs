extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate user32;

use std::mem;

use winapi::minwindef::*;
use winapi::windef::*;
use winapi::winuser::*;
use winapi::winnt::*;
use winapi::wingdi::*;

use gdi32::*;

use kernel32::{GetConsoleWindow, GetModuleHandleW, VirtualAlloc, VirtualFree};

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
static mut BITMAP_WIDTH: i32 = 0;
static mut BITMAP_HEIGHT: i32 = 0;
const BYTES_PER_PIXEL: i32 = 4;

unsafe fn render_weird_gradient(blue_offset: i32, green_offset: i32) {
    let width = BITMAP_WIDTH;
    let pitch = (width * BYTES_PER_PIXEL) as isize;
    let mut row: *mut u8 = BITMAP_MEMORY as *mut u8;
    for y in 0..BITMAP_HEIGHT {
        let mut pixel: *mut u32 = row as *mut u32;
        for x in 0..BITMAP_WIDTH {
            let blue = (x + blue_offset) as u8 as u32;
            let green = (y + green_offset) as u8 as u32;
            *pixel = (green << 8) | blue;
            pixel = pixel.offset(1);
        }
        row = row.offset(pitch);
    }
}

unsafe fn win32_resize_dib_section(width: i32, height: i32) {
    // TODO(coeuvre): Bulletproof this.
    // Maybe don't free first, free after, then free first if that fails.

    if BITMAP_MEMORY != 0 as LPVOID {
        VirtualFree(BITMAP_MEMORY, 0, MEM_RELEASE);
    }

    BITMAP_WIDTH = width;
    BITMAP_HEIGHT = height;

    BITMAP_INFO.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    BITMAP_INFO.bmiHeader.biWidth = BITMAP_WIDTH;
    BITMAP_INFO.bmiHeader.biHeight = -BITMAP_HEIGHT;
    BITMAP_INFO.bmiHeader.biPlanes = 1;
    BITMAP_INFO.bmiHeader.biBitCount = 32;
    BITMAP_INFO.bmiHeader.biCompression = BI_RGB;

    // NOTE(coeuvre): Thank you to Chris Hecker of Spy Party fame
    // for clarifying the deal with StretchDIBits and BitBlt!
    // No more DC for us.
    let bitmap_memory_size = width * height * BYTES_PER_PIXEL;
    BITMAP_MEMORY = VirtualAlloc(0 as LPVOID, bitmap_memory_size as u32,
                                 MEM_COMMIT, PAGE_READWRITE);

    // TODO(coeuvre): Probably clear this to black.
}

unsafe fn win32_update_window(device_context: HDC, client_rect: &RECT,
                              x: i32, y: i32,
                              width: i32, height: i32) {
    let window_width = client_rect.right - client_rect.left;
    let window_height = client_rect.bottom - client_rect.top;
    StretchDIBits(device_context,
                  /*
                  x, y, width, height,
                  x, y, width, height,
                  */
                  0, 0, BITMAP_WIDTH, BITMAP_HEIGHT,
                  0, 0, window_width, window_height,
                  BITMAP_MEMORY, &BITMAP_INFO, DIB_RGB_COLORS, SRCCOPY);
}

unsafe extern "system"
fn win32_main_window_callback(window: HWND, message: UINT, wparam: WPARAM,
                              lparam: LPARAM) -> LRESULT {
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

            let mut client_rect = mem::uninitialized();
            GetClientRect(window, &mut client_rect);
            win32_update_window(device_context, &client_rect, x, y,
                                width, height);
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
        let window = CreateWindowExW(0, window_class.lpszClassName,
                                            wstr!("Handmade Hero"),
                                            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                                            CW_USEDEFAULT, CW_USEDEFAULT,
                                            CW_USEDEFAULT, CW_USEDEFAULT,
                                            0 as HWND, 0 as HMENU,
                                            instance, 0 as LPVOID);
        if window != 0 as HWND {
            let mut x_offset = 0;
            let mut y_offset = 0;

            RUNNING = true;
            while RUNNING {
                let mut message = mem::uninitialized();

                while PeekMessageW(&mut message, 0 as HWND, 0, 0,
                                   PM_REMOVE) != 0 {
                    if message.message == WM_QUIT {
                        RUNNING = false;
                    }

                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }

                render_weird_gradient(x_offset, y_offset);

                let device_context = GetDC(window);
                let mut client_rect = mem::uninitialized();
                GetClientRect(window, &mut client_rect);
                let width = client_rect.right - client_rect.left;
                let height = client_rect.bottom - client_rect.top;
                win32_update_window(device_context, &client_rect, 0, 0,
                                    width, height);
                ReleaseDC(window, device_context);

                x_offset += 1;
                y_offset += 2;
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
