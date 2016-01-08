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
        let wstr: Vec<u16> = OsStr::new($str)
                                 .encode_wide()
                                 .chain(Some(0).into_iter())
                                 .collect();
        wstr.as_ptr()
    });
}

// TODO(coeuvre): This is a global for now.
static mut GLOBAL_RUNNING: bool = true;

struct Win32OffscreenBuffer {
    info: BITMAPINFO,
    memory: LPVOID,
    width: i32,
    height: i32,
    pitch: isize,
    bytes_per_pixel: i32,
}

struct Win32WindowDiension {
    width: i32,
    height: i32,
}

static mut GLOBAL_BACK_BUFFER: Win32OffscreenBuffer = Win32OffscreenBuffer {
    info: BITMAPINFO {
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
    },
    memory: 0 as LPVOID,
    width: 0,
    height: 0,
    pitch: 0,
    bytes_per_pixel: 0,
};

unsafe fn render_weird_gradient(buffer: &Win32OffscreenBuffer,
                                blue_offset: i32,
                                green_offset: i32) {
    // TODO(coeuvre): Let's see what the optimizer does.

    let mut row = buffer.memory as *mut u8;
    for y in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for x in 0..buffer.width {
            let blue = (x + blue_offset) as u8 as u32;
            let green = (y + green_offset) as u8 as u32;
            *pixel = (green << 8) | blue;
            pixel = pixel.offset(1);
        }
        row = row.offset(buffer.pitch);
    }
}

unsafe fn win32_get_window_dimension(window: HWND) -> Win32WindowDiension {
    let mut client_rect = mem::uninitialized();
    GetClientRect(window, &mut client_rect);
    Win32WindowDiension {
        width: client_rect.right - client_rect.left,
        height: client_rect.bottom - client_rect.top,
    }
}

unsafe fn win32_resize_dib_section(buffer: &mut Win32OffscreenBuffer, width: i32, height: i32) {
    // TODO(coeuvre): Bulletproof this.
    // Maybe don't free first, free after, then free first if that fails.

    if buffer.memory != 0 as LPVOID {
        VirtualFree(buffer.memory, 0, MEM_RELEASE);
    }

    buffer.width = width;
    buffer.height = height;
    buffer.bytes_per_pixel = 4;

    // NOTE(coeuvre): When the biheight field is negative, this is the clue to
    // Windows to treat this bitmap as top-down, not bottom-up, meaning that
    // the first three bytes of the image are the color for the top left pixel
    // in the bitmap, not the bottom left!
    buffer.info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    buffer.info.bmiHeader.biWidth = buffer.width;
    buffer.info.bmiHeader.biHeight = -buffer.height;
    buffer.info.bmiHeader.biPlanes = 1;
    buffer.info.bmiHeader.biBitCount = 32;
    buffer.info.bmiHeader.biCompression = BI_RGB;

    // NOTE(coeuvre): Thank you to Chris Hecker of Spy Party fame
    // for clarifying the deal with StretchDIBits and BitBlt!
    // No more DC for us.
    let bitmap_memory_size = buffer.width * buffer.height * buffer.bytes_per_pixel;
    buffer.memory = VirtualAlloc(0 as LPVOID,
                                 bitmap_memory_size as u32,
                                 MEM_COMMIT,
                                 PAGE_READWRITE);

    buffer.pitch = (buffer.width * buffer.bytes_per_pixel) as isize;

    // TODO(coeuvre): Probably clear this to black.
}

unsafe fn win32_display_buffer_in_window(device_context: HDC,
                                         window_width: i32,
                                         window_height: i32,
                                         buffer: &Win32OffscreenBuffer) {
    // TODO(coeuvre): Aspect ratio correction
    // TODO(coeuvre): Player wth stretch modes
    StretchDIBits(device_context,
                  // x, y, width, height,
                  // x, y, width, height,
                  0,
                  0,
                  window_width,
                  window_height,
                  0,
                  0,
                  buffer.width,
                  buffer.height,
                  buffer.memory,
                  &buffer.info,
                  DIB_RGB_COLORS,
                  SRCCOPY);
}

unsafe extern "system" fn win32_main_window_callback(window: HWND,
                                                     message: UINT,
                                                     wparam: WPARAM,
                                                     lparam: LPARAM)
                                                     -> LRESULT {
    match message {
        WM_SIZE => {}
        WM_CLOSE => {
            // TODO(coeuvre): Handle this with a message to the user?
            GLOBAL_RUNNING = false;
        }
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
        }
        WM_DESTROY => {
            // TODO(coeuvre): Handle this as an error - recreate window?
            GLOBAL_RUNNING = false;
        }
        WM_PAINT => {
            let mut paint = mem::uninitialized();
            let device_context = BeginPaint(window, &mut paint);
            let dimension = win32_get_window_dimension(window);
            win32_display_buffer_in_window(device_context,
                                           dimension.width,
                                           dimension.height,
                                           &GLOBAL_BACK_BUFFER);
            EndPaint(window, &paint);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn win_main(instance: HINSTANCE) {
    let mut window_class: WNDCLASSW = mem::zeroed();

    win32_resize_dib_section(&mut GLOBAL_BACK_BUFFER, 1280, 720);

    // TODO(coeuvre): Check if HREDRAW/VREDRAW/OWNDC still matter
    window_class.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
    window_class.lpfnWndProc = Some(win32_main_window_callback);
    window_class.hInstance = instance;
    window_class.lpszClassName = wstr!("HandmadeHeroWindowClass");

    if RegisterClassW(&window_class) != 0 {
        let window = CreateWindowExW(0,
                                     window_class.lpszClassName,
                                     wstr!("Handmade Hero"),
                                     WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                                     CW_USEDEFAULT,
                                     CW_USEDEFAULT,
                                     CW_USEDEFAULT,
                                     CW_USEDEFAULT,
                                     0 as HWND,
                                     0 as HMENU,
                                     instance,
                                     0 as LPVOID);
        if window != 0 as HWND {
            // NOTE(coeuvre): Since we specified CS_OWNDC, we can just
            // get one device context and use it forever because we
            // are not sharing it with anyone.
            let device_context = GetDC(window);

            let mut x_offset = 0;
            let mut y_offset = 0;

            GLOBAL_RUNNING = true;
            while GLOBAL_RUNNING {
                let mut message = mem::uninitialized();

                while PeekMessageW(&mut message, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
                    if message.message == WM_QUIT {
                        GLOBAL_RUNNING = false;
                    }

                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }

                render_weird_gradient(&GLOBAL_BACK_BUFFER, x_offset, y_offset);

                let dimension = win32_get_window_dimension(window);
                win32_display_buffer_in_window(device_context,
                                               dimension.width,
                                               dimension.height,
                                               &GLOBAL_BACK_BUFFER);

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
