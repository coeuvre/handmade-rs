#![windows_subsystem = "windows"]

extern crate core;
extern crate winapi;

mod game;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_rdtsc;
#[cfg(target_arch = "x86")]
use core::arch::x86::_rdtsc;

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
use winapi::um::profileapi::*;
use winapi::um::unknwnbase::LPUNKNOWN;
use winapi::um::wingdi::*;
use winapi::um::winnt::{HRESULT, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE, LARGE_INTEGER};
use winapi::um::winuser::*;
use winapi::um::xinput::*;

use game::*;

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
        library = LoadLibraryA(cstring!("xinput9_1_0.dll").as_ptr());
    }
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

struct Win32SoundOutput {
    samples_per_second: u32,
    bytes_per_sample: u32,
    tone_hz: u32,
    tone_volume: i16,
    wave_period: u32,
    secondary_buffer_size: u32,
    running_sample_index: u32,
    latency_sample_count: u32,
}

unsafe fn win32_clear_buffer(sound_output: &mut Win32SoundOutput) {
    let mut region1 = null_mut();
    let mut region1_size = 0;
    let mut region2 = null_mut();
    let mut region2_size = 0;
    let result = (*GLOBAL_SECONDARY_BUFFER).Lock(
        0,
        sound_output.secondary_buffer_size,
        &mut region1,
        &mut region1_size,
        &mut region2,
        &mut region2_size,
        0,
    );
    if SUCCEEDED(result) {
        let mut dest_sample = region1 as *mut u8;
        for byte_index in 0..region1_size {
            (*dest_sample) = 0;
            dest_sample = dest_sample.add(1);
        }

        dest_sample = region2 as *mut u8;
        for byte_index in 0..region2_size {
            (*dest_sample) = 0;
            dest_sample = dest_sample.add(1);
        }

        (*GLOBAL_SECONDARY_BUFFER).Unlock(region1, region1_size, region2, region2_size);
    }
}

unsafe fn win32_fill_sound_buffer(
    sound_output: &mut Win32SoundOutput,
    byte_to_lock: u32,
    bytes_to_write: u32,
    source_buffer: &GameSoundOutputBuffer,
) {
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
        let mut dest_sample = region1 as *mut i16;
        let mut source_sample = source_buffer.samples;
        let region1_sample_count = region1_size / sound_output.bytes_per_sample;

        for sample_index in 0..region1_sample_count {
            (*dest_sample) = *source_sample;
            dest_sample = dest_sample.add(1);
            source_sample = source_sample.add(1);

            (*dest_sample) = *source_sample;
            dest_sample = dest_sample.add(1);
            source_sample = source_sample.add(1);

            sound_output.running_sample_index += 1;
        }

        dest_sample = region2 as *mut i16;
        let region2_sample_count = region2_size / sound_output.bytes_per_sample;
        for sample_index in 0..region2_sample_count {
            (*dest_sample) = *source_sample;
            dest_sample = dest_sample.add(1);
            source_sample = source_sample.add(1);

            (*dest_sample) = *source_sample;
            dest_sample = dest_sample.add(1);
            source_sample = source_sample.add(1);

            sound_output.running_sample_index += 1;
        }

        (*GLOBAL_SECONDARY_BUFFER).Unlock(region1, region1_size, region2, region2_size);
    }
}

unsafe fn run() -> Result<(), Error> {
    let mut perf_count_frequency = zeroed::<LARGE_INTEGER>();
    QueryPerformanceFrequency(&mut perf_count_frequency);
    let perf_count_frequency = *perf_count_frequency.QuadPart();

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
    let mut sound_output = Win32SoundOutput {
        samples_per_second,
        bytes_per_sample,
        tone_hz,
        tone_volume: 3000,
        wave_period: samples_per_second / tone_hz,
        secondary_buffer_size: samples_per_second * bytes_per_sample,
        running_sample_index: 0,
        latency_sample_count: samples_per_second / 30,
    };
    win32_init_dsound(
        window,
        samples_per_second,
        sound_output.secondary_buffer_size,
    );
    win32_clear_buffer(&mut sound_output);
    (*GLOBAL_SECONDARY_BUFFER).Play(0, 0, DSBPLAY_LOOPING);

    let samples = VirtualAlloc(
        null_mut(),
        sound_output.secondary_buffer_size as usize,
        MEM_COMMIT,
        PAGE_READWRITE,
    ) as *mut i16;

    let mut x_offset: i32 = 0;
    let mut y_offset: i32 = 0;

    RUNNING = true;

    let mut last_counter = zeroed::<LARGE_INTEGER>();
    QueryPerformanceCounter(&mut last_counter);
    let mut last_cycle_count = _rdtsc();
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

                x_offset += (stick_x as f32 / 4096.0) as i32;
                y_offset += (stick_y as f32 / 4096.0) as i32;

                sound_output.tone_hz = 512u32
                    .overflowing_add((256.0 * (stick_y as f32 / 30000.0)) as u32)
                    .0;
                sound_output.wave_period = sound_output.samples_per_second / sound_output.tone_hz;
            }
        }

        let mut is_sound_valid = false;
        let mut play_cursor = 0;
        let mut write_cursor = 0;
        let mut byte_to_lock = 0;
        let mut target_cursor = 0;
        let mut bytes_to_write = 0;
        let result =
            (*GLOBAL_SECONDARY_BUFFER).GetCurrentPosition(&mut play_cursor, &mut write_cursor);
        if SUCCEEDED(result) {
            is_sound_valid = true;

            byte_to_lock = (sound_output.running_sample_index
                * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size;
            target_cursor = (play_cursor
                + sound_output.latency_sample_count * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size;

            bytes_to_write = if byte_to_lock > target_cursor {
                (sound_output.secondary_buffer_size - byte_to_lock) + target_cursor
            } else {
                target_cursor - byte_to_lock
            };
        }

        let mut buffer = GameOffScreenBuffer {
            memory: (*GLOBAL_BACK_BUFFER).memory as *mut u8,
            width: (*GLOBAL_BACK_BUFFER).width,
            height: (*GLOBAL_BACK_BUFFER).height,
            pitch: (*GLOBAL_BACK_BUFFER).pitch,
        };

        let mut sound_buffer = GameSoundOutputBuffer {
            samples,
            sample_count: bytes_to_write / sound_output.bytes_per_sample,
            samples_per_second: sound_output.samples_per_second ,
        };
        game_update_and_render(&mut buffer, &mut sound_buffer);

        // DirectSound output test
        if is_sound_valid {
            win32_fill_sound_buffer(&mut sound_output, byte_to_lock, bytes_to_write, &sound_buffer);
        }

        let dimension = win32_get_window_dimension(window);
        win32_display_buffer_in_window(
            device_context,
            dimension.width,
            dimension.height,
            &*GLOBAL_BACK_BUFFER,
        );

        let end_cycle_count = _rdtsc();

        let mut end_counter = zeroed::<LARGE_INTEGER>();
        QueryPerformanceCounter(&mut end_counter);

        let cycles_elapsed = end_cycle_count - last_cycle_count;
        let counter_elapsed = end_counter.QuadPart() - last_counter.QuadPart();
        let ms_per_frame = counter_elapsed * 1000 / perf_count_frequency;
        let fps = perf_count_frequency / counter_elapsed;
        println!("{}ms/f, {}f/s, {}c/f", ms_per_frame, fps, cycles_elapsed);

        last_counter = end_counter;
        last_cycle_count = end_cycle_count;
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
