#include "handmade.h"

#include <Windows.h>
#include <Xinput.h>
#include <dsound.h>

DebugReadFileResult debug_platform_read_entire_file(char *filename) {
    DebugReadFileResult result = {};

    HANDLE file = CreateFileA(filename, GENERIC_READ, FILE_SHARE_READ, 0, OPEN_ALWAYS, 0, 0);
    if (file != INVALID_HANDLE_VALUE) {
        LARGE_INTEGER file_size;
        if (GetFileSizeEx(file, &file_size)) {
            DWORD file_size32 = safe_truncate_uint64(file_size.QuadPart);
            result.contents = VirtualAlloc(0, file_size32, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
            if (result.contents) {
                DWORD bytes_read;
                if (ReadFile(file, result.contents, file_size32, &bytes_read, 0) && (file_size32 == bytes_read)) {
                    result.content_size = file_size32;
                } else {
                    debug_platform_free_file_memory(result.contents);
                    result.contents = 0;
                }
            }
        }
        CloseHandle(file);
    }

    return result;
}

void debug_platform_free_file_memory(void *memory) {
    if (memory) {
        VirtualFree(memory, 0, MEM_RELEASE);
    }
}

int debug_platform_write_entire_file(char *filename, uint32_t memory_size, void *memory) {
    int result = 0;

    HANDLE file = CreateFileA(filename, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0);
    if (file != INVALID_HANDLE_VALUE) {

        DWORD bytes_written;
        if (WriteFile(file, memory, memory_size, &bytes_written, 0)) {
            result = bytes_written == memory_size;
        } else {
        }

        CloseHandle(file);
    }

    return result;
}


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

static void win32_process_keyboard_message(
    GameButtonState *new_state,
    int is_down
) {
    ASSERT(new_state->ended_down != is_down);
    new_state->ended_down = is_down;
    ++new_state->half_transition_count;
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
            ASSERT(!"Keyboard input came in through a non-dispatch message!");
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

static void win32_process_pending_messages(GameControllerInput *keyboard_controller) {
    MSG message;
    while (PeekMessageA(&message, 0, 0, 0, PM_REMOVE)) {
        switch (message.message) {
            case WM_QUIT: {
                RUNNING = false;
                break;
            }
            case WM_SYSKEYDOWN:
            case WM_SYSKEYUP:
            case WM_KEYDOWN:
            case WM_KEYUP: {
                int vk_code = (int) message.wParam;
                int was_down = (message.lParam & (1 << 30)) != 0;
                int is_down = (message.lParam & (1 << 31)) == 0;
                if (was_down != is_down) {
                    switch ((char) vk_code) {
                        case 'W': {
                            win32_process_keyboard_message(&keyboard_controller->move_up, is_down);
                            break;
                        }
                        case 'A': {
                            win32_process_keyboard_message( &keyboard_controller->move_left, is_down);
                            break;
                        }
                        case 'S': {
                            win32_process_keyboard_message(&keyboard_controller->move_down, is_down);
                            break;
                        }
                        case 'D': {
                            win32_process_keyboard_message(&keyboard_controller->move_right, is_down);
                            break;
                        }
                        case 'Q': {
                            win32_process_keyboard_message(&keyboard_controller->left_shoulder, is_down);
                            break;
                        }
                        case 'E': {
                            win32_process_keyboard_message(&keyboard_controller->right_shoulder, is_down);
                            break;
                        }
                        default: {
                            switch (vk_code) {
                                case VK_UP: {
                                    win32_process_keyboard_message(&keyboard_controller->action_up, is_down);
                                    break;
                                }
                                case VK_LEFT: {
                                    win32_process_keyboard_message( &keyboard_controller->action_left, is_down);
                                    break;
                                }
                                case VK_DOWN: {
                                    win32_process_keyboard_message(&keyboard_controller->action_down, is_down);
                                    break;
                                }
                                case VK_RIGHT: {
                                    win32_process_keyboard_message(&keyboard_controller->action_right, is_down);
                                    break;
                                }
                                case VK_ESCAPE: {
                                    win32_process_keyboard_message(&keyboard_controller->start, is_down);
                                    break;
                                }
                                case VK_SPACE: {
                                    win32_process_keyboard_message(&keyboard_controller->back, is_down);
                                    break;
                                }
                                default: {}
                            }
                        }
                    }
                }

                int alt_key_was_down = (int)(message.lParam & (1 << 29));
                if (is_down && (vk_code == VK_ESCAPE || (alt_key_was_down != 0 && vk_code == VK_F4))) {
                    RUNNING = false;
                }
                break;
            }
            default:  {
                TranslateMessage(&message);
                DispatchMessage(&message);
            }
        }
    }
}

static float win32_process_x_input_stick_value(SHORT stick_value, SHORT dead_zone) {
    float result = 0;
    if (stick_value < -dead_zone) {
        result = ((float) stick_value) / 32768.0f;
    } else if (stick_value > dead_zone) {
        result = ((float) stick_value) / 32767.0f;
    }
    return result;
}

static LONGLONG PERF_COUNT_FREQUENCY;

inline LARGE_INTEGER win32_get_wall_clock() {
    LARGE_INTEGER result;
    QueryPerformanceCounter(&result);
    return result;
}

inline float win32_get_seconds_elapsed(LARGE_INTEGER start, LARGE_INTEGER end) {
    return (float) (end.QuadPart - start.QuadPart) / (float) PERF_COUNT_FREQUENCY;
}

int CALLBACK
WinMain(HINSTANCE Instance, HINSTANCE PrevInstance, LPSTR CmdLine, int CmdShow) {
    LARGE_INTEGER perf_count_frequency_result;
    QueryPerformanceFrequency(&perf_count_frequency_result);
    PERF_COUNT_FREQUENCY = perf_count_frequency_result.QuadPart;

    UINT desired_scheduler_ms = 1;
    int sleep_is_granular = timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR;

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

    // TODO: How do we reliably query on this on Windows?
    int monitor_refresh_hz = 60;
    int game_update_hz = monitor_refresh_hz / 2;
    float target_seconds_per_frame = 1.0f / (float)game_update_hz;

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

    LARGE_INTEGER last_counter = win32_get_wall_clock();

    DWORD64 last_cycle_count = __rdtsc();
    while (RUNNING) {
        GameControllerInput *old_keyboard_controller = GetController(old_input, 0);
        GameControllerInput *new_keyboard_controller = GetController(new_input, 0);
        *new_keyboard_controller = {};
        new_keyboard_controller->is_connected = true;
        for (int index = 0; index < ARRAY_COUNT(new_keyboard_controller->buttons); ++index) {
            new_keyboard_controller->buttons[index].ended_down = old_keyboard_controller->buttons[index].ended_down;
        }

        win32_process_pending_messages(new_keyboard_controller);

        // Note: Input
        DWORD max_controller_count = XUSER_MAX_COUNT;
        if (max_controller_count > ARRAY_COUNT(new_input->controllers) - 1) {
            max_controller_count = ARRAY_COUNT(new_input->controllers) - 1;
        }

        for (DWORD index = 0; index < max_controller_count; ++index) {
            DWORD x_input_controller_index = index + 1;
            GameControllerInput *old_controller = GetController(old_input, x_input_controller_index);
            GameControllerInput *new_controller = GetController(new_input, x_input_controller_index);

            XINPUT_STATE controller_state;
            if (XInputGetState(x_input_controller_index, &controller_state) == ERROR_SUCCESS) {
                new_controller->is_connected = true;
                XINPUT_GAMEPAD *pad = &controller_state.Gamepad;

                new_controller->is_analog = 1;
                new_controller->stick_average_x = win32_process_x_input_stick_value(pad->sThumbLX, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);
                new_controller->stick_average_y = win32_process_x_input_stick_value(pad->sThumbLY, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);

                if (new_controller->stick_average_x != 0.0f || new_controller->stick_average_y != 0.0f) {
                    new_controller->is_analog = true;
                }

                if (pad->wButtons & XINPUT_GAMEPAD_DPAD_UP) {
                    new_controller->stick_average_y = 1.0;
                    new_controller->is_analog = false;
                }
                if (pad->wButtons & XINPUT_GAMEPAD_DPAD_DOWN) {
                    new_controller->stick_average_y = -1.0;
                    new_controller->is_analog = false;
                }
                if (pad->wButtons & XINPUT_GAMEPAD_DPAD_LEFT) {
                    new_controller->stick_average_x = -1.0;
                    new_controller->is_analog = false;
                }
                if (pad->wButtons & XINPUT_GAMEPAD_DPAD_RIGHT) {
                    new_controller->stick_average_x = 1.0;
                    new_controller->is_analog = false;
                }

                float threshold = 0.5f;
                win32_process_xinput_digitial_button(
                    (new_controller->stick_average_x < -threshold) ? 1 : 0,
                    &old_controller->move_left, 1, &new_controller->move_left
                );
                win32_process_xinput_digitial_button(
                    (new_controller->stick_average_x > threshold) ? 1 : 0,
                    &old_controller->move_right, 1, &new_controller->move_right
                );
                win32_process_xinput_digitial_button(
                    (new_controller->stick_average_y < -threshold) ? 1 : 0,
                    &old_controller->move_down, 1, &new_controller->move_down
                );
                win32_process_xinput_digitial_button(
                    (new_controller->stick_average_y > threshold) ? 1 : 0,
                    &old_controller->move_up, 1, &new_controller->move_up
                );

                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_down, XINPUT_GAMEPAD_A, &new_controller->action_down);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_right, XINPUT_GAMEPAD_B, &new_controller->action_right);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_left, XINPUT_GAMEPAD_X, &new_controller->action_left);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_up, XINPUT_GAMEPAD_Y, &new_controller->action_up);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->left_shoulder, XINPUT_GAMEPAD_LEFT_SHOULDER, &new_controller->left_shoulder);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->right_shoulder, XINPUT_GAMEPAD_RIGHT_SHOULDER, &new_controller->right_shoulder);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->back, XINPUT_GAMEPAD_BACK, &new_controller->back);
                win32_process_xinput_digitial_button(pad->wButtons, &old_controller->back, XINPUT_GAMEPAD_START, &new_controller->start);
            } else {
                new_controller->is_connected = false;
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

            bytes_to_lock = (int)((sound_output.running_sample_index * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size);
            target_cursor = (int)((play_cursor
                + sound_output.latency_sample_count * sound_output.bytes_per_sample)
                % sound_output.secondary_buffer_size);

            if (bytes_to_lock > target_cursor) {
                bytes_to_write = (int)((sound_output.secondary_buffer_size - bytes_to_lock) + target_cursor);
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

        LARGE_INTEGER work_counter = win32_get_wall_clock();
        float work_seconds_elapsed = win32_get_seconds_elapsed(last_counter, work_counter);
        float seconds_elapsed_for_frame = work_seconds_elapsed;
        if (seconds_elapsed_for_frame < target_seconds_per_frame) {
            if (sleep_is_granular) {
                DWORD ms = (DWORD)(1000.0f * (target_seconds_per_frame - seconds_elapsed_for_frame));
                if (ms > 0) {
                    Sleep(ms);
                }
            }
            ASSERT(win32_get_seconds_elapsed(last_counter, win32_get_wall_clock()) < target_cursor);
            while (seconds_elapsed_for_frame < target_seconds_per_frame) {
                seconds_elapsed_for_frame = win32_get_seconds_elapsed(last_counter, win32_get_wall_clock());
            }
        } else {
            // TODO: MISSED FRAME RATE
            // TODO: Logging
        }

        HDC device_context = GetDC(window);
        Win32WindowDimension dimension = win32_get_window_dimension(window);
        win32_display_buffer_in_window(
            device_context,
            dimension.width,
            dimension.height,
            &BACK_BUFFER
        );

        GameInput *tmp_input = new_input;
        new_input = old_input;
        old_input = tmp_input;

        LARGE_INTEGER end_counter = win32_get_wall_clock();
        float ms_per_frame = 1000.0f * win32_get_seconds_elapsed(last_counter, end_counter);
        last_counter = end_counter;

        DWORD64 end_cycle_count = __rdtsc();
        int cycles_elapsed = (int) (end_cycle_count - last_cycle_count);
        last_cycle_count = end_cycle_count;

        float fps = 0.0; //(float) PERF_COUNT_FREQUENCY / (float) counter_elapsed;
        char buf[1024];
        sprintf(buf, "%fms/f, %ff/s, %dc/f\n", ms_per_frame, fps, cycles_elapsed);
        OutputDebugString(buf);
    }

    return 0;
}
