extern crate winapi;
extern crate dsound;
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate xinput;

use std::mem;

use winapi::dsound::*;
use winapi::guiddef::*;
use winapi::minwindef::*;
use winapi::mmreg::*;
use winapi::unknwnbase::*;
use winapi::windef::*;
use winapi::winerror::*;
use winapi::wingdi::*;
use winapi::winnt::*;
use winapi::winuser::*;
use winapi::xinput::*;

use dsound::*;
use gdi32::*;
use kernel32::*;
use user32::*;
use xinput::*;

macro_rules! cstr {
    ($str:expr) => ({
        use std::ffi::CString;
        CString::new($str).unwrap().as_ptr()
    });
}

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

struct Win32OffscreenBuffer {
    // NOTE(coeuvre): Pixels are always 32-bits wide, Memory Order BB GG RR XX
    info: BITMAPINFO,
    memory: LPVOID,
    width: i32,
    height: i32,
    pitch: isize,
}

struct Win32WindowDiension {
    width: i32,
    height: i32,
}

// TODO(coeuvre): This is a global for now.
static mut GLOBAL_RUNNING: bool = true;
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
};

// NOTE(coeuvre): XInputGetState
type XInputGetState = extern "system" fn(dwUserIndex: DWORD, pState: *mut XINPUT_STATE) -> DWORD;
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD {
    ERROR_DEVICE_NOT_CONNECTED
}

// NOTE(coeuvre): XInputSetState
type XInputSetState = extern "system" fn(dwUserIndex: DWORD, pVibration: *mut XINPUT_VIBRATION)
                                         -> DWORD;
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD {
    ERROR_DEVICE_NOT_CONNECTED
}

type DirectSoundCreate = extern "system" fn(pcGuidDevice: LPCGUID,
                                            ppDS: *mut LPDIRECTSOUND,
                                            pUnkOuter: LPUNKNOWN)
                                            -> HRESULT;

static mut XINPUT_GET_STATE: *mut XInputGetState = 0 as *mut XInputGetState;
static mut XINPUT_SET_STATE: *mut XInputSetState = 0 as *mut XInputSetState;

unsafe fn win32_load_xinput() {
    // FIXME(coeuvre): For some reasons, even we successfully load XInput
    // library and set XINPUT_GET_STATE point to function returned by
    // GetProcAddress, it will still crash. So use the static linked
    // function for now.

    // TODO(coeuvre): Test this on Windows 8
    let mut xinput_library = LoadLibraryW(wstr!("xinput1_4.dll"));
    if xinput_library == 0 as HMODULE {
        // TODO(coeuvre): Diagnostic
        xinput_library = LoadLibraryW(wstr!("xinput1_3.dll"));
    }

    if xinput_library != 0 as HMODULE {
        XINPUT_GET_STATE = mem::transmute(GetProcAddress(xinput_library, cstr!("XInputGetState")));
        if XINPUT_GET_STATE == 0 as *mut XInputGetState {
            XINPUT_GET_STATE = mem::transmute(&xinput_get_state_stub);
        }

        XINPUT_SET_STATE = mem::transmute(GetProcAddress(xinput_library, cstr!("XInputSetState")));
        if XINPUT_SET_STATE == 0 as *mut XInputSetState {
            XINPUT_SET_STATE = mem::transmute(&xinput_set_state_stub);
        }

        // TODO(coeuvre): Diagnostic
    } else {
        // TODO(coeuvre): Diagnostic
    }

    XINPUT_GET_STATE = mem::transmute(&XInputGetState);
    XINPUT_SET_STATE = mem::transmute(&XInputSetState);
}

unsafe fn win32_init_dsound(window: HWND, samples_per_second: u32, buffer_size: u32) {
    // NOTE(coeuvre): Load the library.
    let dsound_library = LoadLibraryW(wstr!("dsound.dll"));
    if dsound_library != 0 as HMODULE {
        // NOTE(coeuvre): Get a DirectSound object! - cooperative
        let mut direct_sound_create: *mut DirectSoundCreate =
            mem::transmute(GetProcAddress(dsound_library, cstr!("DirectSoundCreate")));

        // FIXME(coeuvre): For some reasons, even we successfully load DSound
        // library and set direct_sound_create point to function returned by
        // GetProcAddress, it will still crash. So use the static linked
        // function for now.
        //
        // Same as loading XInput library.
        direct_sound_create = mem::transmute(&DirectSoundCreate);

        // TODO(coeuvre): Double-check that this works on XP - DirectSound8 or 7??
        let mut direct_sound = mem::uninitialized();
        if direct_sound_create != 0 as *mut DirectSoundCreate &&
           SUCCEEDED((*direct_sound_create)(0 as LPCGUID, &mut direct_sound, 0 as LPUNKNOWN)) {
            let mut wave_format: WAVEFORMATEX = mem::zeroed();
            wave_format.wFormatTag = WAVE_FORMAT_PCM;
            wave_format.nChannels = 2;
            wave_format.nSamplesPerSec = samples_per_second;
            wave_format.wBitsPerSample = 16;
            wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
            wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec *
                                          wave_format.nBlockAlign as u32;
            wave_format.cbSize = 0;

            if SUCCEEDED((*direct_sound).SetCooperativeLevel(window, DSSCL_PRIORITY)) {
                let mut buffer_description: DSBUFFERDESC = mem::zeroed();
                buffer_description.dwSize = mem::size_of::<DSBUFFERDESC>() as u32;
                buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;

                // NOTE(coeuvre): "Create" a primary buffer.
                // TODO(coeuvre): DSBCAPS_GLOBALFOCUS?
                let mut primary_buffer = mem::uninitialized();
                if SUCCEEDED((*direct_sound).CreateSoundBuffer(&buffer_description,
                                                               &mut primary_buffer,
                                                               0 as LPUNKNOWN)) {
                    if SUCCEEDED((*primary_buffer).SetFormat(&wave_format)) {
                        // NOTE(coeuvre): We have finnally set the format!
                        println!("Primary buffer format was set.");
                    } else {
                        // TODO(coeuvre): Diagnostic
                    }
                } else {
                    // TODO(coeuvre): Diagnostic
                }
            } else {
                // TODO(coeuvre): Diagnostic
            }

            // TODO(coeuvre): DSBCAPS_GETCURRENTPOSITION2
            let mut buffer_description: DSBUFFERDESC = mem::zeroed();
            buffer_description.dwSize = mem::size_of::<DSBUFFERDESC>() as u32;
            buffer_description.dwFlags = 0;
            buffer_description.dwBufferBytes = buffer_size;
            buffer_description.lpwfxFormat = &mut wave_format;
            let mut secondary_buffer = mem::uninitialized();
            if SUCCEEDED((*direct_sound).CreateSoundBuffer(&buffer_description,
                                                           &mut secondary_buffer,
                                                           0 as LPUNKNOWN)) {
                println!("Secondary buffer created successfully.");
            }
        } else {
            // TODO(coeuvre): Diagnostic
        }
    } else {
        // TODO(coeuvre): Diagnostic
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

unsafe fn win32_resize_dib_section(buffer: &mut Win32OffscreenBuffer, width: i32, height: i32) {
    // TODO(coeuvre): Bulletproof this.
    // Maybe don't free first, free after, then free first if that fails.

    if buffer.memory != 0 as LPVOID {
        VirtualFree(buffer.memory, 0, MEM_RELEASE);
    }

    buffer.width = width;
    buffer.height = height;

    let bytes_per_pixel = 4;

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
    let bitmap_memory_size = buffer.width * buffer.height * bytes_per_pixel;
    buffer.memory = VirtualAlloc(0 as LPVOID,
                                 bitmap_memory_size as u32,
                                 MEM_RESERVE | MEM_COMMIT,
                                 PAGE_READWRITE);

    buffer.pitch = (buffer.width * bytes_per_pixel) as isize;

    // TODO(coeuvre): Probably clear this to black.
}

unsafe fn win32_display_buffer_in_window(buffer: &Win32OffscreenBuffer,
                                         device_context: HDC,
                                         window_width: i32,
                                         window_height: i32) {
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
        WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
            let vk_code = wparam as i32;
            let was_down = (lparam & (1 << 30)) != 0;
            let is_down = (lparam & (1 << 31)) == 0;

            if was_down != is_down {
                match vk_code as u8 as char {
                    'W' => {}
                    'A' => {}
                    'S' => {}
                    'D' => {}
                    'Q' => {}
                    'E' => {}
                    _ => {
                        match vk_code {
                            VK_UP => {}
                            VK_LEFT => {}
                            VK_DOWN => {}
                            VK_RIGHT => {}
                            VK_ESCAPE => {
                                print!("ESCAPSE: ");
                                if is_down {
                                    println!("is_down");
                                }
                                if was_down {
                                    println!("was_down");
                                }
                            }
                            VK_SPACE => {}
                            _ => {}
                        }
                    }
                }
            }

            let alt_key_was_down = (lparam & (1 << 29)) != 0;
            if vk_code == VK_F4 && alt_key_was_down {
                GLOBAL_RUNNING = false;
            }
        }
        WM_PAINT => {
            let mut paint = mem::uninitialized();
            let device_context = BeginPaint(window, &mut paint);
            let dimension = win32_get_window_dimension(window);
            win32_display_buffer_in_window(&GLOBAL_BACK_BUFFER,
                                           device_context,
                                           dimension.width,
                                           dimension.height);
            EndPaint(window, &paint);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }

    0
}

unsafe fn win_main(instance: HINSTANCE) {
    win32_load_xinput();

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

            win32_init_dsound(window, 48000, (48000 * mem::size_of::<u16>() * 2) as u32);

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

                // TODO(coeuvre): Should we poll this more frequently?
                for controller_index in 0..XUSER_MAX_COUNT {
                    let mut controller_state = mem::uninitialized();
                    if (*XINPUT_GET_STATE)(controller_index, &mut controller_state) ==
                       ERROR_SUCCESS {
                        // NOTE(coeuvre): The controller is plugged in.
                        // TODO(coeuvre): See if controller_state.dwPacketNumber increments too rapidly.
                        let ref pad = controller_state.Gamepad;

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
                    } else {
                        // NOTE(coeuvre): The controller is not available.
                    }
                }

                render_weird_gradient(&GLOBAL_BACK_BUFFER, x_offset, y_offset);

                let dimension = win32_get_window_dimension(window);
                win32_display_buffer_in_window(&GLOBAL_BACK_BUFFER,
                                               device_context,
                                               dimension.width,
                                               dimension.height);
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
        // NOTE(coeuvre): Initialize all the global variables which needs call functions.
        XINPUT_GET_STATE = mem::transmute(&xinput_get_state_stub);
        XINPUT_SET_STATE = mem::transmute(&xinput_set_state_stub);

        // Hide the console window
        let console = GetConsoleWindow();
        if console != 0 as HWND {
            ShowWindow(console, winapi::SW_HIDE);
        }

        let instance = GetModuleHandleW(0 as LPCWSTR);

        win_main(instance);
    }
}
