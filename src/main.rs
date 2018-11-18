#![windows_subsystem = "windows"]

extern crate winapi;

use std::mem::{size_of, transmute, zeroed};
use std::ptr::{null, null_mut};

use winapi::shared::guiddef::LPCGUID;
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::mmreg::*;
use winapi::shared::windef::{HDC, HMENU, HWND, RECT};
use winapi::shared::winerror::{ERROR_DEVICE_NOT_CONNECTED, ERROR_SUCCESS, SUCCEEDED};
use winapi::um::dsound::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::unknwnbase::LPUNKNOWN;
use winapi::um::wingdi::*;
use winapi::um::winnt::{HRESULT, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE};
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
    return ERROR_DEVICE_NOT_CONNECTED;
}
static mut XINPUT_GET_STATE: XInputGetStateFn = xinput_get_state_stub;

type XInputSetStateFn = extern "system" fn(DWORD, *mut XINPUT_VIBRATION) -> DWORD;
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD {
    return ERROR_DEVICE_NOT_CONNECTED;
}
static mut XINPUT_SET_STATE: XInputSetStateFn = xinput_set_state_stub;

unsafe fn win32_load_xinput() {
    let mut library = LoadLibraryA(cstring!("xinput1_4.dll").as_ptr());
    if library == 0 as HINSTANCE {
        library = LoadLibraryA(cstring!("xinput1_3.dll").as_ptr());
    }
    if library != 0 as HINSTANCE {
        XINPUT_GET_STATE = transmute(GetProcAddress(library, cstring!("XInputGetState").as_ptr()));
        XINPUT_SET_STATE = transmute(GetProcAddress(library, cstring!("XInputSetState").as_ptr()));
    }
}

static mut RUNNING: bool = false;
static mut GLOBAL_BACK_BUFFER: *mut Win32OffScreenBuffer = 0 as *mut Win32OffScreenBuffer;
static mut GLOBAL_SECONDARY_BUFFER: *mut IDirectSoundBuffer = 0 as *mut IDirectSoundBuffer;

type DirectSoundCreateFn = fn(LPCGUID, *mut LPDIRECTSOUND, LPUNKNOWN) -> HRESULT;

unsafe fn win32_init_dsound(window: HWND, samples_per_seconds: u32, buffer_size: u32) {
    let library = LoadLibraryA(cstring!("dsound.dll").as_ptr());
    if library == 0 as HINSTANCE {
        return;
    }

    let ptr = GetProcAddress(library, cstring!("DirectSoundCreate").as_ptr());
    if ptr == null_mut() {
        return;
    }

    let mut direct_sound: *mut IDirectSound = null_mut();
    let direct_sound_create: DirectSoundCreateFn = transmute(ptr);
    let result = direct_sound_create(null(), &mut direct_sound, null_mut());
    if SUCCEEDED(result) {
        let mut wave_format = zeroed::<WAVEFORMATEX>();
        wave_format.wFormatTag = WAVE_FORMAT_PCM;
        wave_format.nChannels = 2;
        wave_format.nSamplesPerSec = samples_per_seconds;
        wave_format.wBitsPerSample = 16;
        wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
        wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec * (wave_format.nBlockAlign as u32);

        // Create primary buffer
        let result = (*direct_sound).SetCooperativeLevel(window, DSSCL_PRIORITY);
        if SUCCEEDED(result) {
            let mut buffer_description = zeroed::<DSBUFFERDESC>();
            buffer_description.dwSize = size_of::<DSBUFFERDESC>() as u32;
            buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;

            let mut primary_buffer: *mut IDirectSoundBuffer = null_mut();
            let result = (*direct_sound).CreateSoundBuffer(
                &buffer_description,
                &mut primary_buffer,
                null_mut(),
            );
            if SUCCEEDED(result) {
                let result = (*primary_buffer).SetFormat(&mut wave_format);
                if SUCCEEDED(result) {
                    println!("Primary buffer format was set");
                }
            }
        }

        // Create secondary buffer
        {
            let mut buffer_description = zeroed::<DSBUFFERDESC>();
            buffer_description.dwSize = size_of::<DSBUFFERDESC>() as u32;
            buffer_description.dwBufferBytes = buffer_size;
            buffer_description.lpwfxFormat = &mut wave_format;

            let result = (*direct_sound).CreateSoundBuffer(
                &buffer_description,
                &mut GLOBAL_SECONDARY_BUFFER,
                null_mut(),
            );
            if SUCCEEDED(result) {
                println!("Secondary buffer created");
            }
        }
    }
}

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
    if buffer.memory != null_mut() {
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
        null_mut(),
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
        WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
            let vk_code = wparam as i32;
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
                    _ => match vk_code {
                        VK_UP => {}
                        VK_LEFT => {}
                        VK_DOWN => {}
                        VK_RIGHT => {}
                        VK_ESCAPE => {}
                        VK_SPACE => {}
                        _ => {}
                    },
                }
            }

            let alt_key_was_down = lparam & (1 << 29);
            if is_down && (vk_code == VK_ESCAPE || alt_key_was_down != 0 && vk_code == VK_F4) {
                RUNNING = false;
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

    let instance = GetModuleHandleW(null_mut());

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
        null_mut(),
    );
    if window == 0 as HWND {
        return Err("Failed to create window");
    }
    let device_context = GetDC(window);

    let samples_per_second = 48000;
    let bytes_per_sample = (size_of::<u16>() * 2) as u32;
    let tone_hz = 256;
    let tone_volume: i16 = 3000;
    let square_wave_period = samples_per_second / tone_hz;
    let half_square_wave_period = square_wave_period / 2;
    let secondary_buffer_size = samples_per_second * bytes_per_sample;
    let mut running_sample_index: u32 = 0;
    win32_init_dsound(window, samples_per_second, secondary_buffer_size);
    let mut sound_is_playing = false;

    let mut x_offset: i32 = 0;
    let mut y_offset: i32 = 0;

    RUNNING = true;
    while RUNNING {
        let mut message = zeroed::<MSG>();
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

        // DirectSound output test
        {
            let mut play_cursor = 0;
            let mut write_cursor = 0;
            let result =
                (*GLOBAL_SECONDARY_BUFFER).GetCurrentPosition(&mut play_cursor, &mut write_cursor);
            if SUCCEEDED(result) {
                let byte_to_lock = (running_sample_index * bytes_per_sample) % secondary_buffer_size;
                let bytes_to_write = if byte_to_lock == play_cursor {
                    secondary_buffer_size
                } else if byte_to_lock > play_cursor {
                    (secondary_buffer_size - byte_to_lock) + play_cursor
                } else {
                    play_cursor - byte_to_lock
                };

                let mut region1 = null_mut();
                let mut region1_size = 0;
                let mut region2 = null_mut();
                let mut region2_size = 0;
                let result = (*GLOBAL_SECONDARY_BUFFER).Lock(
                    byte_to_lock,
                    bytes_to_write,
                    &mut region1,
                    &mut region1_size,
                    &mut region2,
                    &mut region2_size,
                    0,
                );
                if SUCCEEDED(result) {
                    // [i16  i16  ] i16  i16   ...
                    // [LEFT RIGHT] LEFT RIGHT ...
                    let mut sample_out = region1 as *mut i16;
                    let region1_sample_count = region1_size / bytes_per_sample;

                    for sample_index in 0..region1_sample_count {
                        let sample_value = if (running_sample_index / half_square_wave_period) % 2 == 0 {
                            tone_volume
                        } else {
                            -tone_volume
                        };
                        (*sample_out) = sample_value;
                        sample_out = sample_out.offset(1);
                        (*sample_out) = sample_value;
                        sample_out = sample_out.offset(1);
                        running_sample_index += 1;
                    }

                    sample_out = region2 as *mut i16;
                    let region2_sample_count = region2_size / bytes_per_sample;
                    for sample_index in 0..region2_sample_count {
                        let sample_value = if (running_sample_index / half_square_wave_period) % 2 == 0 {
                            tone_volume
                        } else {
                            -tone_volume
                        };
                        (*sample_out) = sample_value;
                        sample_out = sample_out.offset(1);
                        (*sample_out) = sample_value;
                        sample_out = sample_out.offset(1);
                        running_sample_index += 1;
                    }

                    (*GLOBAL_SECONDARY_BUFFER).Unlock(region1, region1_size, region2, region2_size);

                    if !sound_is_playing {
                        sound_is_playing = true;
                        (*GLOBAL_SECONDARY_BUFFER).Play(0, 0, DSBPLAY_LOOPING);
                    }
                }
            }
        }

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
