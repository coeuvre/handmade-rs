#![windows_subsystem = "windows"]

extern crate winapi;

use std::mem::{size_of, zeroed};

use winapi::shared::minwindef::{LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HMENU, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::memoryapi::*;
use winapi::um::wingdi::*;
use winapi::um::winnt::{LPCWCHAR, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE};
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
static mut BITMAP_MEMORY: LPVOID = 0 as LPVOID;
static mut BITMAP_WIDTH: i32 = 0;
static mut BITMAP_HEIGHT: i32 = 0;
static BYTES_PER_PIXEL: i32 = 4;

unsafe fn render_weird_gradient(x_offset: i32, y_offset: i32) {
    let width = BITMAP_WIDTH;
    let height = BITMAP_HEIGHT;
    let pitch = (width * BYTES_PER_PIXEL) as isize;
    let mut row = BITMAP_MEMORY as *mut u8;
    for y in 0..height {
        let mut pixel = row as *mut u32;
        for x in 0..width {
            let b = x + x_offset;
            let g = y + y_offset;
            *pixel = (((g & 0xFF)<< 8) | (b & 0xFF)) as u32;
            pixel = pixel.offset(1);
        }
        row = row.offset(pitch);
    }
}

#[inline]
fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    let mut bitmap_info = unsafe { zeroed::<BITMAPINFO>() };
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = width;
    bitmap_info.bmiHeader.biHeight = -height;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB;
    bitmap_info
}

unsafe fn win32_resize_dib_section(width: i32, height: i32) {
    if BITMAP_MEMORY != 0 as LPVOID {
        VirtualFree(BITMAP_MEMORY, 0 as usize, MEM_RELEASE);
    }

    BITMAP_WIDTH = width;
    BITMAP_HEIGHT = height;

    let bitmap_memory_size = width * height * BYTES_PER_PIXEL;
    BITMAP_MEMORY = VirtualAlloc(
        0 as LPVOID,
        bitmap_memory_size as usize,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
}

unsafe fn win32_update_window(
    device_context: HDC,
    window_rect: &RECT,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    let window_width = window_rect.right - window_rect.left;
    let window_height = window_rect.bottom - window_rect.top;

    StretchDIBits(
        device_context,
        0,
        0,
        window_width,
        window_height,
        0,
        0,
        BITMAP_WIDTH,
        BITMAP_HEIGHT,
        BITMAP_MEMORY,
        &bitmap_info(BITMAP_WIDTH, BITMAP_HEIGHT),
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
            let mut client_rect = zeroed::<RECT>();
            GetClientRect(window, &mut client_rect);
            win32_update_window(device_context, &client_rect, x, y, width, height);
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

    let mut x_offset = 0;
    let mut y_offset = 0;
    while RUNNING {
        while PeekMessageW(&mut message, 0 as HWND, 0, 0, PM_REMOVE)!= 0 {
            if message.message == WM_QUIT {
                RUNNING = false;
            }

            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        render_weird_gradient(x_offset, y_offset);

        let device_context = GetDC(window);
        let mut client_rect = zeroed::<RECT>();
        GetClientRect(window, &mut client_rect);
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        win32_update_window(device_context, &client_rect, 0, 0, width, height);
        ReleaseDC(window, device_context);

        x_offset += 1;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    unsafe { run() }
}
