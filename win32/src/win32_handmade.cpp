#include <Windows.h>
#include <Xinput.h>
#include <dsound.h>

#include <stdio.h>
#include <stdint.h>

#include "handmade.h"

#define KILOBYTES(value) ((value) * 1024LL)
#define MEGABYTES(value) (KILOBYTES(value) * 1024LL)
#define GIGABYTES(value) (MEGABYTES(value) * 1024LL)
#define TERABYTES(value) (GIGABYTES(value) * 1024LL)

bool RUNNING;

// NOTE: XinputGetState
#define X_INPUT_GET_STATE(name) DWORD WINAPI name(DWORD UserIndex, XINPUT_STATE *State)

typedef X_INPUT_GET_STATE(XInputGetStateFn);

X_INPUT_GET_STATE(X_INPUT_GET_STATE_STUB) {
    return ERROR_DEVICE_NOT_CONNECTED;
}

static XInputGetStateFn *X_INPUT_GET_STATE_ = X_INPUT_GET_STATE_STUB;
#define XInputGetState X_INPUT_GET_STATE_

// NOTE: XinputSetState
#define X_INPUT_SET_STATE(name) DWORD WINAPI name(DWORD UserIndex, XINPUT_VIBRATION *Vibration)

typedef X_INPUT_SET_STATE(XInputSetStateFn);

X_INPUT_SET_STATE(X_INPUT_SET_STATE_STUB) {
    return ERROR_DEVICE_NOT_CONNECTED;
}

static XInputSetStateFn *X_INPUT_SET_STATE_ = X_INPUT_SET_STATE_STUB;
#define XInputSetState X_INPUT_SET_STATE_

static void
win32_load_xinput() {
    // TODO: Test this on Windows 8
    HMODULE library = LoadLibrary("xinput1_4.dll");
    if (!library) {
        // TODO: Diagnostic
        library = LoadLibrary("xinput9_1_0.dll");
    }
    if (!library) {
        // TODO: Diagnostic
        library = LoadLibrary("xinput1_3.dll");
    }

    if (library) {
        XInputGetState = (XInputGetStateFn *) GetProcAddress(library, "XInputGetState");
        if (!XInputGetState) {
            XInputGetState = X_INPUT_GET_STATE_STUB;
        }

        XInputSetState = (XInputSetStateFn *) GetProcAddress(library, "XInputSetState");
        if (!XInputSetState) {
            XInputSetState = X_INPUT_SET_STATE_STUB;
        }

        // TODO: Diagnostic
    } else {
        // TODO: Diagnostic
    }
}

static void win32_process_xinput_digitial_button(
    USHORT x_input_button_state, 
    GameButtonState *old_state, 
    USHORT button_bit, 
    GameButtonState *new_state
) {
    new_state->ended_down = (x_input_button_state & button_bit) == button_bit;
    if (old_state->ended_down == new_state->ended_down) {
        new_state->half_transition_count = 1;
    } else { 
        new_state->half_transition_count = 0;
    }    
}

struct Win32WindowDimension {
    int width;
    int height;
};

struct Win32OffscreenBuffer {
    BITMAPINFO info;
    void *memory;
    int width;
    int height;
    int pitch;
};

static Win32OffscreenBuffer BACK_BUFFER;
static LPDIRECTSOUNDBUFFER SECONDARY_BUFFER;


static Win32WindowDimension win32_get_window_dimension(HWND window) {
    Win32WindowDimension result;
    RECT client_rect;
    GetClientRect(window, &client_rect);
    result.width = client_rect.right - client_rect.left;
    result.height = client_rect.bottom - client_rect.top;
    return result;
}

static void win32_resize_dib_section(Win32OffscreenBuffer *buffer, int width, int height) {
    if (buffer->memory) {
        VirtualFree(buffer->memory, 0, MEM_RELEASE);
    }

    buffer->width = width;
    buffer->height = height;

    int bytes_per_pixel = 4;

    buffer->info.bmiHeader.biSize = sizeof(buffer->info.bmiHeader);
    buffer->info.bmiHeader.biWidth = buffer->width;
    buffer->info.bmiHeader.biHeight = -buffer->height;
    buffer->info.bmiHeader.biPlanes = 1;
    buffer->info.bmiHeader.biBitCount = 32;
    buffer->info.bmiHeader.biCompression = BI_RGB;

    int bitmap_memory_size = buffer->width * buffer->height * bytes_per_pixel;
    buffer->memory = VirtualAlloc(
            0,
            bitmap_memory_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE
    );
    buffer->pitch = buffer->width * bytes_per_pixel;
}

static void
win32_display_buffer_in_window(HDC device_context, int window_width, int window_height, Win32OffscreenBuffer *buffer) {
    StretchDIBits(
            device_context,
            0,
            0,
            window_width,
            window_height,
            0,
            0,
            buffer->width,
            buffer->height,
            buffer->memory,
            &buffer->info,
            DIB_RGB_COLORS,
            SRCCOPY
    );
}

LRESULT CALLBACK win32_main_window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_DESTROY:
        case WM_CLOSE: {
            RUNNING = false;
            break;
        }
        case WM_SYSKEYDOWN:
        case WM_SYSKEYUP:
        case WM_KEYDOWN:
        case WM_KEYUP: {
            int vk_code = (int) wparam;
            int was_down = (lparam & (1 << 30)) != 0;
            int is_down = (lparam & (1 << 31)) != 0;
            if (was_down != is_down) {
                switch ((char) vk_code) {
                    case 'W': {
                        break;
                    }
                    case 'A': {
                        break;
                    }
                    case 'S': {
                        break;
                    }
                    case 'D': {
                        break;
                    }
                    case 'Q': {
                        break;
                    }
                    case 'E': {
                        break;
                    }
                    default: {
                        switch (vk_code) {
                        case VK_UP: {
                            break;
                        }
                        case VK_LEFT: {
                            break;
                        }
                        case VK_DOWN: {
                            break;
                        }
                        case VK_RIGHT: {
                            break;
                        }
                        case VK_ESCAPE: {
                            break;
                        }
                        case VK_SPACE: {
                            break;
                        }
                        default: {}
                        }
                    }
                }
            }

            int alt_key_was_down = lparam & (1 << 29);
            if (is_down && (vk_code == VK_ESCAPE || alt_key_was_down != 0 && vk_code == VK_F4)) {
                RUNNING = false;
            }
            break;
        }
        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC device_context = BeginPaint(window, &ps);
            Win32WindowDimension dimension = win32_get_window_dimension(window);
            win32_display_buffer_in_window(device_context, dimension.width, dimension.height, &BACK_BUFFER);
            EndPaint(window, &ps);
            break;
        }
        default: {
            return DefWindowProc(window, message, wparam, lparam);
        }
    }

    return 0;
}

#define DIRECT_SOUND_CREATE(name) HRESULT WINAPI name(LPCGUID pcGuidDevice, LPDIRECTSOUND *ppDS, LPUNKNOWN pUnkOuter)
typedef DIRECT_SOUND_CREATE(DirectSoundCreateFn);


struct Win32SoundOutput {
    int samples_per_second;
    int bytes_per_sample;
    DWORD secondary_buffer_size;
    uint32_t running_sample_index;
    uint32_t latency_sample_count;
};

static void win32_init_dsound(HWND window, int samples_per_second, int buffer_size) {
    HMODULE library = LoadLibrary("dsound.dll");
    if (!library) {
        return;
    }

    DirectSoundCreateFn *DirectSoundCreate = (DirectSoundCreateFn *) GetProcAddress(library, "DirectSoundCreate");
    if (!DirectSoundCreate) {
        return;
    }

    LPDIRECTSOUND direct_sound;
    if (!SUCCEEDED(DirectSoundCreate(0, &direct_sound, 0))) {
        return;
    }

    WAVEFORMATEX wave_format = {};
    wave_format.wFormatTag = WAVE_FORMAT_PCM;
    wave_format.nChannels = 2;
    wave_format.nSamplesPerSec = samples_per_second;
    wave_format.wBitsPerSample = 16;
    wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
    wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec * wave_format.nBlockAlign;
    wave_format.cbSize = 0;

    if (SUCCEEDED(direct_sound->SetCooperativeLevel(window, DSSCL_PRIORITY))) {
        DSBUFFERDESC buffer_description = {};
        buffer_description.dwSize = sizeof(buffer_description);
        buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;

        // NOTE: "Create" a primary buffer
        LPDIRECTSOUNDBUFFER primary_buffer;
        if (SUCCEEDED(direct_sound->CreateSoundBuffer(&buffer_description, &primary_buffer, 0))) {
            HRESULT Error = primary_buffer->SetFormat(&wave_format);
            if (SUCCEEDED(Error)) {
                // NOTE: We have finnaly set the format!
                OutputDebugString("Primary buffer format was set.\n");
            } else {
                // TODO: Diagnostic
            }
        } else {
            // TODO: Diagnostic
        }
    }

    // NOTE: "Create" a secondary buffer
    // TODO: DSBCAPS_GETCURRENTPOSITION2
    DSBUFFERDESC buffer_description = {};
    buffer_description.dwSize = sizeof(buffer_description);
    buffer_description.dwFlags = DSBCAPS_GETCURRENTPOSITION2;
#if HANDMADE_INTERNAL
    buffer_description.dwFlags |= DSBCAPS_GLOBALFOCUS;
#endif
    buffer_description.dwBufferBytes = buffer_size;
    buffer_description.lpwfxFormat = &wave_format;
    HRESULT Error = direct_sound->CreateSoundBuffer(&buffer_description, &SECONDARY_BUFFER, 0);
    if (SUCCEEDED(Error)) {
        OutputDebugString("Secondary buffer created successfully.\n");
    } else {
    }
}

static void win32_clear_buffer(Win32SoundOutput *sound_output) {
    VOID *region1;
    DWORD region1_size;
    VOID *region2;
    DWORD region2_size;
    if (SUCCEEDED(SECONDARY_BUFFER->Lock(
            0,
            sound_output->secondary_buffer_size,
            &region1, &region1_size,
            &region2, &region2_size,
            0))) {
        uint8_t *dst = (uint8_t *) region1;
        for (DWORD index = 0; index < region1_size; ++index) {
            *dst++ = 0;
        }

        dst = (uint8_t *) region2;
        for (DWORD index = 0; index < region2_size; ++index) {
            *dst++ = 0;
        }

        SECONDARY_BUFFER->Unlock(region1, region1_size, region2, region2_size);
    }
}

static void win32_fill_sound_buffer(
    Win32SoundOutput *sound_output,
    int bytes_to_lock, 
    int bytes_to_write,
    GameSoundBuffer *source
) {
    VOID *Region1;
    DWORD Region1Size;
    VOID *Region2;
    DWORD Region2Size;

    if (SUCCEEDED(SECONDARY_BUFFER->Lock(bytes_to_lock, bytes_to_write,
                                              &Region1, &Region1Size,
                                              &Region2, &Region2Size,
                                              0))) {
        // TODO: assert that Region1Size/Region2Size is valid
        DWORD Region1SampleCount = Region1Size / sound_output->bytes_per_sample;
        int16_t *DestSample = (int16_t *) Region1;
        int16_t *SourceSample = (int16_t *) source->samples;
        for (DWORD SampleIndex = 0; SampleIndex < Region1SampleCount; ++SampleIndex) {
            *DestSample++ = *SourceSample++;
            *DestSample++ = *SourceSample++;
            ++sound_output->running_sample_index;
        }

        DWORD Region2SampleCount = Region2Size / sound_output->bytes_per_sample;
        DestSample = (int16_t *) Region2;
        for (DWORD SampleIndex = 0; SampleIndex < Region2SampleCount; ++SampleIndex) {
            *DestSample++ = *SourceSample++;
            *DestSample++ = *SourceSample++;
            ++sound_output->running_sample_index;
        }

        SECONDARY_BUFFER->Unlock(Region1, Region1Size, Region2, Region2Size);
    }
}

int CALLBACK
WinMain(HINSTANCE Instance, HINSTANCE PrevInstance, LPSTR CmdLine, int CmdShow) {
    LARGE_INTEGER perf_count_frequency_result;
    QueryPerformanceCounter(&perf_count_frequency_result);
    LONGLONG perf_count_frequency = perf_count_frequency_result.QuadPart;

    win32_load_xinput();

    win32_resize_dib_section(&BACK_BUFFER, 1280, 720);

    HINSTANCE instance = GetModuleHandle(0);

    LPCSTR class_name = "HandmadeHero";
    WNDCLASS window_class = {};
    window_class.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
    window_class.lpfnWndProc = win32_main_window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = class_name;

    if (!RegisterClass(&window_class)) {
        return -1;
    }

    HWND window = CreateWindowEx(
            0, class_name, "Handmade Hero",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT,
            CW_USEDEFAULT, CW_USEDEFAULT,
            0, 0, instance, 0
    );
    if (!window) {
        return -1;
    }

    int samples_per_second = 48000;
    int bytes_per_sample = sizeof(int16_t) * 2;
    Win32SoundOutput sound_output = {};
    sound_output.samples_per_second = samples_per_second;
    sound_output.bytes_per_sample = bytes_per_sample;
    sound_output.secondary_buffer_size = samples_per_second * bytes_per_sample;
    sound_output.running_sample_index = 0;
    sound_output.latency_sample_count = samples_per_second / 15;

    win32_init_dsound(window, samples_per_second, sound_output.secondary_buffer_size);
    win32_clear_buffer(&sound_output);
    SECONDARY_BUFFER->Play(0, 0, DSBPLAY_LOOPING);

    void *samples = VirtualAlloc(
        0, sound_output.secondary_buffer_size, 
        MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE
    );

    // TODO: Game Memory
#if HANDMADE_INTERNAL
    LPVOID base_address = (LPVOID) TERABYTES(2);
#else
    LPVOID base_address = 0;
#endif
    GameMemory game_memory = {};
    game_memory.permanent_storage_size = MEGABYTES(64);
    game_memory.transient_storage_size = MEGABYTES(256);
    size_t total_size = game_memory.permanent_storage_size + game_memory.transient_storage_size;
    game_memory.permanent_storage = VirtualAlloc(
            base_address,
            total_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE
    );
    game_memory.transient_storage = ((char *) game_memory.permanent_storage) + game_memory.permanent_storage_size;

    RUNNING = true;

    GameInput input[2] = {};
    GameInput *new_input = &input[0];
    GameInput *old_input = &input[1];

    LARGE_INTEGER last_counter;
    QueryPerformanceCounter(&last_counter);
    DWORD64 last_cycle_count = __rdtsc();
    while (RUNNING) {
        MSG message;
        while (PeekMessageA(&message, 0, 0, 0, PM_REMOVE)) {
            if (message.message == WM_QUIT) {
                RUNNING = false;
            }

            TranslateMessage(&message);
            DispatchMessage(&message);
        }

        // Note: Input
        DWORD max_controller_count = XUSER_MAX_COUNT;
        if (max_controller_count > ARRAY_COUNT(new_input->controllers)) {
            max_controller_count = ARRAY_COUNT(new_input->controllers);
        }

        for (DWORD index = 0; index < max_controller_count; ++index) {
            GameControllerInput *old_controller = &old_input->controllers[index];
            GameControllerInput *new_controller = &new_input->controllers[index];

            XINPUT_STATE controller_state;
            if (XInputGetState(index, &controller_state) == ERROR_SUCCESS) {
                XINPUT_GAMEPAD *pad = &controller_state.Gamepad;                
                
                new_controller->is_analog = 1;
                new_controller->start_x = old_controller->end_x;
                new_controller->start_y = old_controller->end_y;

                float x;
                if (pad->sThumbLX < 0) {
                    x = ((float) pad->sThumbLX) / 32768.0f;
                } else {
                    x = ((float) pad->sThumbLX) / 32767.0f;
                }
                new_controller->min_x = x;
                new_controller->max_x = x;
                new_controller->end_x = x;

                float y;
                if (pad->sThumbLY < 0) {
                    y = ((float) pad->sThumbLY) / 32768.0f;
                } else {
                    y = ((float) pad->sThumbLY) / 32767.0f;
                }
                new_controller->min_y = y;
                new_controller->max_y = y;
                new_controller->end_y = y;

                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->down, XINPUT_GAMEPAD_A, &new_controller->down);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->right, XINPUT_GAMEPAD_B, &new_controller->right);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->left, XINPUT_GAMEPAD_X, &new_controller->left);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->up, XINPUT_GAMEPAD_Y, &new_controller->up);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->left_shoulder, XINPUT_GAMEPAD_LEFT_SHOULDER, &new_controller->left_shoulder);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->right_shoulder, XINPUT_GAMEPAD_RIGHT_SHOULDER, &new_controller->right_shoulder);
            }
        }

        // NOTE: Sound
        bool is_sound_valid = false;
        DWORD play_cursor;
        DWORD write_cursor;
        int bytes_to_lock = 0;
        int target_cursor = 0;
        int bytes_to_write = 0;
        if (SUCCEEDED(SECONDARY_BUFFER->GetCurrentPosition(&play_cursor, &write_cursor))) {
            is_sound_valid = true;

            bytes_to_lock = (sound_output.running_sample_index * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size;
            target_cursor = (play_cursor
                + sound_output.latency_sample_count * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size;

            if (bytes_to_lock > target_cursor) {
                bytes_to_write = (sound_output.secondary_buffer_size - bytes_to_lock) + target_cursor;
            } else {
                bytes_to_write = target_cursor - bytes_to_lock;
            }
        }

        GameOffscreenBuffer offscreen_buffer = {};
        offscreen_buffer.memory = BACK_BUFFER.memory;
        offscreen_buffer.width = BACK_BUFFER.width;
        offscreen_buffer.height = BACK_BUFFER.height;
        offscreen_buffer.pitch = BACK_BUFFER.pitch;

        GameSoundBuffer sound_buffer = {};
        sound_buffer.samples = samples;
        sound_buffer.sample_count = bytes_to_write / sound_output.bytes_per_sample;
        sound_buffer.samples_per_second = sound_output.samples_per_second;

        game_update_and_render(&game_memory, new_input, &offscreen_buffer, &sound_buffer);

        if (is_sound_valid) {
            win32_fill_sound_buffer(&sound_output, bytes_to_lock, bytes_to_write, &sound_buffer);
        }

        HDC device_context = GetDC(window);
        Win32WindowDimension dimension = win32_get_window_dimension(window);
        win32_display_buffer_in_window(
            device_context,
            dimension.width,
            dimension.height,
            &BACK_BUFFER
        );

        DWORD64 end_cycle_count = __rdtsc();
        LARGE_INTEGER end_counter;
        QueryPerformanceCounter(&end_counter);

        int cycles_elapsed = (int) (end_cycle_count - last_cycle_count);
        int counter_elapsed = (int) (end_counter.QuadPart - last_counter.QuadPart);
        int ms_per_frame = (int) (counter_elapsed * 1000 / perf_count_frequency);
        int fps = (int) (perf_count_frequency / counter_elapsed);
        char buf[1024];
        sprintf(buf, "%dms/f, %df/s, %dc/f", ms_per_frame, fps, cycles_elapsed);
        OutputDebugString(buf);

        last_counter = end_counter;
        last_cycle_count = end_cycle_count;

        GameInput *tmp_input = new_input;
        new_input = old_input;
        old_input = tmp_input;
    }

    return 0;
}
