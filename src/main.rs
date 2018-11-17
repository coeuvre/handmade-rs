#![windows_subsystem = "windows"]

extern crate winapi;

use std::mem::{size_of, transmute, zeroed};

use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HMENU, HWND, RECT};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::wingdi::*;
use winapi::um::winnt::{LPCWCHAR, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE};
use winapi::um::winuser::*;
use winapi::um::xinput::*;

macro_rules! wcstring {
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

macro_rules! cstring {
    ($s:expr) => {{
        use std::ffi::CString;
        CString::new($s).unwrap()
    }};
}

type Error = &'static str;

struct Win32OffScreenBuffer {
    info: BITMAPINFO,
    memory: LPVOID,
    width: i32,
    height: i32,
    pitch: i32,
}

struct Win32WindowDimension {
    width: i32,
    height: i32,
}

type XInputGetStateFn = extern "system" fn(DWORD, *mut XINPUT_STATE) -> DWORD;
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD {
    return 0;
}
static mut XINPUT_GET_STATE: XInputGetStateFn = xinput_get_state_stub;

type XInputSetStateFn = extern "system" fn(DWORD, *mut XINPUT_VIBRATION) -> DWORD;
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD {
    return 0;
}
static mut XINPUT_SET_STATE: XInputSetStateFn = xinput_set_state_stub;

unsafe fn win32_load_xinput() {
    let library = LoadLibraryA(cstring!("xinput1_3.dll").as_ptr());
    if library != 0 as HINSTANCE {
        XINPUT_GET_STATE = transmute(GetProcAddress(library, cstring!("XInputGetState").as_ptr()));
        XINPUT_SET_STATE = transmute(GetProcAddress(library, cstring!("XInputSetState").as_ptr()));
    }
}

static mut RUNNING: bool = false;
static mut GLOBAL_BACK_BUFFER: *mut Win32OffScreenBuffer = 0 as *mut Win32OffScreenBuffer;

unsafe fn win32_get_window_dimension(window: HWND) -> Win32WindowDimension {
    let mut client_rect = zeroed::<RECT>();
    GetClientRect(window, &mut client_rect);
    let width = client_rect.right - client_rect.left;
    let height = client_rect.bottom - client_rect.top;
    return Win32WindowDimension { width, height };
}

unsafe fn render_weird_gradient(buffer: &mut Win32OffScreenBuffer, x_offset: i32, y_offset: i32) {
    let mut row = buffer.memory as *mut u8;
    for y in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for x in 0..buffer.width {
            let b = x + x_offset;
            let g = y + y_offset;
            *pixel = (((g & 0xFF) << 8) | (b & 0xFF)) as u32;
            pixel = pixel.offset(1);
        }
        row = row.offset(buffer.pitch as isize);
    }
}

unsafe fn win32_resize_dib_section(buffer: &mut Win32OffScreenBuffer, width: i32, height: i32) {
    if buffer.memory != 0 as LPVOID {
        VirtualFree(buffer.memory, 0 as usize, MEM_RELEASE);
    }

    buffer.width = width;
    buffer.height = height;
    let bytes_per_pixel = 4;

    buffer.info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    buffer.info.bmiHeader.biWidth = buffer.width;
    buffer.info.bmiHeader.biHeight = -buffer.height;
    buffer.info.bmiHeader.biPlanes = 1;
    buffer.info.bmiHeader.biBitCount = 32;
    buffer.info.bmiHeader.biCompression = BI_RGB;

    let bitmap_memory_size = buffer.width * buffer.height * bytes_per_pixel;
    buffer.memory = VirtualAlloc(
        0 as LPVOID,
        bitmap_memory_size as usize,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    buffer.pitch = buffer.width * bytes_per_pixel;
}

unsafe fn win32_display_buffer_in_window(
    device_context: HDC,
    window_width: i32,
    window_height: i32,
    buffer: &Win32OffScreenBuffer,
) {
    StretchDIBits(
        device_context,
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
        WM_SYSKEYDOWN |
        WM_SYSKEYUP |
        WM_KEYDOWN |
        WM_KEYUP => {
            let vk_code = wparam;
            let was_down = (lparam & (1 << 30)) != 0;
            let is_down = (lparam & (1 << 31)) != 0;
            if was_down != is_down {
                match vk_code as u8 as char {
                    'W' => {}
                    'A' => {}
                    'S' => {}
                    'D' => {}
                    'Q' => {}
                    'E' => {}
                    _ => match vk_code as i32 {
                        VK_UP => {}
                        VK_LEFT => {}
                        VK_DOWN => {}
                        VK_RIGHT => {}
                        VK_ESCAPE => {}
                        VK_SPACE => {}
                        _ => {}
                    }
                }
            }
        }
        WM_PAINT => {
            let mut ps = zeroed::<PAINTSTRUCT>();
            let device_context = BeginPaint(window, &mut ps);
            let dimension = win32_get_window_dimension(window);
            win32_display_buffer_in_window(
                device_context,
                dimension.width,
                dimension.height,
                &mut *GLOBAL_BACK_BUFFER,
            );
            EndPaint(window, &mut ps);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn run() -> Result<(), Error> {
    win32_load_xinput();

    win32_resize_dib_section(&mut *GLOBAL_BACK_BUFFER, 1280, 720);

    let instance = GetModuleHandleW(0 as LPCWCHAR);

    let mut window_class = zeroed::<WNDCLASSW>();
    let class_name = wcstring!("HandmadeHeroWindowClass");
    window_class.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
    window_class.lpfnWndProc = Some(win32_main_window_proc);
    window_class.hInstance = instance;
    window_class.lpszClassName = class_name.as_ptr();

    let result = RegisterClassW(&window_class);
    if result == 0 {
        return Err("Failed to register class");
    }

    let window = CreateWindowExW(
        0,
        class_name.as_ptr(),
        wcstring!("Handmade Hero").as_ptr(),
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
    let device_context = GetDC(window);

    let mut message = zeroed::<MSG>();
    let mut x_offset: i32 = 0;
    let mut y_offset: i32 = 0;

    RUNNING = true;
    while RUNNING {
        while PeekMessageW(&mut message, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
            if message.message == WM_QUIT {
                RUNNING = false;
            }

            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        for i in 0..XUSER_MAX_COUNT {
            let mut controller_state = zeroed::<XINPUT_STATE>();
            if XINPUT_GET_STATE(i, &mut controller_state) == ERROR_SUCCESS {
                let pad = &controller_state.Gamepad;
                let up = pad.wButtons & XINPUT_GAMEPAD_DPAD_UP;
                let down = pad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN;
                let left = pad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT;
                let right = pad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT;
                let start = pad.wButtons & XINPUT_GAMEPAD_START;
                let back = pad.wButtons & XINPUT_GAMEPAD_BACK;
                let left_shoulder = pad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER;
                let right_shoulder = pad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER;
                let a_button = pad.wButtons & XINPUT_GAMEPAD_A;
                let b_button = pad.wButtons & XINPUT_GAMEPAD_B;
                let x_button = pad.wButtons & XINPUT_GAMEPAD_X;
                let y_button = pad.wButtons & XINPUT_GAMEPAD_Y;

                let stick_x = pad.sThumbLX;
                let stick_y = pad.sThumbLY;

                x_offset += (stick_x >> 12) as i32;
                y_offset += (stick_y >> 12) as i32;
            }
        }

        render_weird_gradient(&mut *GLOBAL_BACK_BUFFER, x_offset, y_offset);

        let dimension = win32_get_window_dimension(window);
        win32_display_buffer_in_window(
            device_context,
            dimension.width,
            dimension.height,
            &*GLOBAL_BACK_BUFFER,
        );
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    unsafe {
        let mut back_buffer = zeroed::<Win32OffScreenBuffer>();
        GLOBAL_BACK_BUFFER = &mut back_buffer;

        run()
    }
}
