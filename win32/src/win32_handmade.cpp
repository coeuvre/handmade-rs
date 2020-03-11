#include "../../game/handmade_platform.h"
#include "win32_handmade.h"
#include <stdio.h>

static void toggle_fullscreen(HWND window) {
    DWORD style = GetWindowLong(window, GWL_STYLE);
    if (style & WS_OVERLAPPEDWINDOW) {
        MONITORINFO monitor_info = {sizeof(monitor_info)};
        if (GetWindowPlacement(window, &WINDOW_POSITION) &&
            GetMonitorInfo(MonitorFromWindow(window, MONITOR_DEFAULTTOPRIMARY), &monitor_info)) {
            SetWindowLong(window, GWL_STYLE, style & ~WS_OVERLAPPEDWINDOW);
            SetWindowPos(window, HWND_TOP, monitor_info.rcMonitor.left, monitor_info.rcMonitor.top,
                         monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                         monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
                         SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
        }
    } else {
        SetWindowLong(window, GWL_STYLE, style | WS_OVERLAPPEDWINDOW);
        SetWindowPlacement(window, &WINDOW_POSITION);
        SetWindowPos(window, 0, 0, 0, 0, 0,
                     SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
    }
}

static void cat_strings(size_t source_a_count, char *source_a,
                        size_t source_b_count, char *source_b,
                        size_t dest_count, char *dest) {
    for (int i = 0; i < source_a_count; ++i) {
        *dest++ = *source_a++;
    }

    for (int i = 0; i < source_b_count; ++i) {
        *dest++ = *source_b++;
    }

    *dest = 0;
}

static int string_length(char *string) {
    int result = 0;
    while (*string++) {
        result++;
    }
    return result;
}

DEBUG_PLATFORM_FREE_FILE_MEMORY(debug_platform_free_file_memory) {
    if (memory) {
        VirtualFree(memory, 0, MEM_RELEASE);
    }
}

DEBUG_PLATFORM_READ_ENTIRE_FILE(debug_platform_read_entire_file) {
    DebugReadFileResult result = {};

    HANDLE file = CreateFileA(file_name, GENERIC_READ, FILE_SHARE_READ, 0, OPEN_ALWAYS, 0, 0);
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

DEBUG_PLATFORM_WRITE_ENTIRE_FILE(debug_platform_write_entire_file) {
    int result = 0;

    HANDLE file = CreateFileA(file_name, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0);
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

static void win32_build_exe_path_file_name(Win32State *state, char* file_name, int dest_count, char *dest) {
    cat_strings(state->one_past_last_exe_file_name_slash - state->exe_file_name, state->exe_file_name,
                string_length(file_name), file_name, dest_count, dest);
}

static void win32_get_input_file_location(Win32State *state, bool input_stream, int index, int dest_count, char *dest) {
    char tmp[64];
    wsprintf(tmp, "loop_edit_%d_%s.hmi", index, input_stream ? "input" : "state");
    win32_build_exe_path_file_name(state, tmp, dest_count, dest);
}

static Win32ReplayBuffer *win32_get_replay_buffer(Win32State *state, int index) {
    ASSERT(index > 0 && index < ARRAY_COUNT(state->replay_buffers));
    return &state->replay_buffers[index];
}

static void win32_begin_input_recording(Win32State *state, int input_recording_index) {
    Win32ReplayBuffer *replay_buffer = win32_get_replay_buffer(state, input_recording_index);
    if (replay_buffer->memory) {
        state->input_recording_index = input_recording_index;

        char file_name[WIN32_STATE_FILE_NAME_COUNT];
        win32_get_input_file_location(state, true, input_recording_index, sizeof(file_name), file_name);
        state->input_recording_handle = CreateFileA(file_name, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0);

#if 0
        LARGE_INTEGER file_position;
        file_position.QuadPart = state->total_size;
        SetFilePointerEx(state->input_recording_handle, file_position, 0, FILE_BEGIN);
#endif

        CopyMemory(replay_buffer->memory, state->game_memory_block, state->total_size);
    }
}

static void win32_end_input_recording(Win32State *state) {
    state->input_recording_index = 0;

    if (state->input_recording_handle != INVALID_HANDLE_VALUE) {
        CloseHandle(state->input_recording_handle);
        state->input_recording_handle = INVALID_HANDLE_VALUE;
    }
}

static void win32_begin_input_playback(Win32State *state, int input_playback_index) {
    Win32ReplayBuffer *replay_buffer = win32_get_replay_buffer(state, input_playback_index);
    if (replay_buffer->memory) {
        state->input_playback_index = input_playback_index;
        char file_name[WIN32_STATE_FILE_NAME_COUNT];
        win32_get_input_file_location(state, true, input_playback_index, sizeof(file_name), file_name);
        state->input_playback_handle = CreateFileA(file_name, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0);

#if 0
        LARGE_INTEGER file_position;
        file_position.QuadPart = state->total_size;
        SetFilePointerEx(state->input_playback_handle, file_position, 0, FILE_BEGIN);
#endif

        CopyMemory(state->game_memory_block, replay_buffer->memory, state->total_size);
    }
}

static void win32_end_input_playback(Win32State *state) {
    state->input_playback_index = 0;

    if (state->input_playback_handle != INVALID_HANDLE_VALUE) {
        CloseHandle(state->input_playback_handle);
        state->input_playback_handle = INVALID_HANDLE_VALUE;
    }
}

static void win32_record_input(Win32State *state, GameInput *new_input) {
    DWORD bytes_written;
    WriteFile(state->input_recording_handle, new_input, sizeof(*new_input), &bytes_written, 0);
}

static void win32_playback_input(Win32State *state, GameInput *new_input) {
    DWORD bytes_read;
    if (ReadFile(state->input_playback_handle, new_input, sizeof(*new_input), &bytes_read, 0)) {
        if (bytes_read == 0) {
            int input_playing_index = state->input_playback_index;
            win32_end_input_playback(state);
            win32_begin_input_playback(state, input_playing_index);
            ReadFile(state->input_playback_handle, new_input, sizeof(*new_input), &bytes_read, 0);
        }
    }
}

inline FILETIME win32_get_last_write_time(char *file_name) {
    FILETIME result = {};

    WIN32_FILE_ATTRIBUTE_DATA data;
    if (GetFileAttributesEx(file_name, GetFileExInfoStandard, &data)) {
        result = data.ftLastWriteTime;
    }

    return result;
}

static Win32GameCode win32_load_game_code(char *source_dll_name, char *temp_dll_name) {
    Win32GameCode result = {};
    result.library_last_write_time = win32_get_last_write_time(source_dll_name);
    CopyFile(source_dll_name, temp_dll_name, FALSE);
    result.library = LoadLibrary(temp_dll_name);
    if (result.library) {
        result.game_update_and_render = (GameUpdateAndRender *) GetProcAddress(result.library, "game_update_and_render");
        result.game_get_sound_samples = (GameGetSoundSamples *) GetProcAddress(result.library, "game_get_sound_samples");

        result.is_valid = result.game_update_and_render && result.game_get_sound_samples;
    }
    if (!result.is_valid) {
        result.game_update_and_render = 0;
        result.game_get_sound_samples = 0;
    }

    return result;
}

static void win32_unload_game_code(Win32GameCode *game_code) {
    if (game_code->library) {
        FreeLibrary(game_code->library);
        game_code->library = 0;
    }

    game_code->is_valid = 0;
    game_code->game_update_and_render = 0;
    game_code->game_get_sound_samples = 0;
}

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
    if (new_state->ended_down != is_down) {
        new_state->ended_down = is_down;
        ++new_state->half_transition_count;
    }
}

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

    buffer->bytes_per_pixel = 4;

    buffer->info.bmiHeader.biSize = sizeof(buffer->info.bmiHeader);
    buffer->info.bmiHeader.biWidth = buffer->width;
    buffer->info.bmiHeader.biHeight = -buffer->height;
    buffer->info.bmiHeader.biPlanes = 1;
    buffer->info.bmiHeader.biBitCount = 32;
    buffer->info.bmiHeader.biCompression = BI_RGB;

    int bitmap_memory_size = buffer->width * buffer->height * buffer->bytes_per_pixel;
    buffer->memory = VirtualAlloc(
            0,
            bitmap_memory_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE
    );
    buffer->pitch = buffer->width * buffer->bytes_per_pixel;
}

static void
win32_display_buffer_in_window(HDC device_context, int window_width, int window_height, Win32OffscreenBuffer *buffer) {
    if (window_width >= buffer->width * 2 && window_height >= buffer->height * 2) {
        StretchDIBits(
                device_context,
                0,
                0,
                buffer->width * 2,
                buffer->height * 2,
                0,
                0,
                buffer->width,
                buffer->height,
                buffer->memory,
                &buffer->info,
                DIB_RGB_COLORS,
                SRCCOPY
        );
    } else {
        int offset_x = 10;
        int offset_y = 10;

        PatBlt(device_context, 0, 0, window_width, offset_y, BLACKNESS);
        PatBlt(device_context, 0, offset_y, offset_x, window_height - offset_y, BLACKNESS);
        PatBlt(device_context, offset_x + buffer->width, offset_y, window_width - offset_x - buffer->width,
               window_height - offset_y, BLACKNESS);
        PatBlt(device_context, offset_x, offset_y + buffer->height, window_width - offset_x,
               window_height - offset_y - buffer->height, BLACKNESS);

        StretchDIBits(
                device_context,
                offset_x,
                offset_y,
                buffer->width,
                buffer->height,
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
}

LRESULT CALLBACK win32_main_window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_ACTIVATEAPP: {
#if 0
            if (wparam == TRUE) {
                SetLayeredWindowAttributes(window, RGB(0, 0, 0), 255, LWA_ALPHA);
            } else {
                SetLayeredWindowAttributes(window, RGB(0, 0, 0), 64, LWA_ALPHA);
            }
#endif
            break;
        }
        case WM_DESTROY:
        case WM_CLOSE: {
            RUNNING = false;
            break;
        }
        case WM_SETCURSOR: {
            if (DEBUG_SHOW_CURSOR) {
                return DefWindowProc(window, message, wparam, lparam);
            } else {
                SetCursor(0);
            }
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

static void win32_process_pending_messages(GameControllerInput *keyboard_controller, Win32State *state) {
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
                    if (vk_code == 'W') {
                        win32_process_keyboard_message(&keyboard_controller->move_up, is_down);
                    } else if (vk_code == 'A') {
                        win32_process_keyboard_message( &keyboard_controller->move_left, is_down);
                    } else if (vk_code == 'S') {
                        win32_process_keyboard_message(&keyboard_controller->move_down, is_down);
                    } else if (vk_code == 'D') {
                        win32_process_keyboard_message(&keyboard_controller->move_right, is_down);
                    } else if (vk_code == 'Q') {
                        win32_process_keyboard_message(&keyboard_controller->left_shoulder, is_down);
                    } else if (vk_code == 'E') {
                        win32_process_keyboard_message(&keyboard_controller->right_shoulder, is_down);
                    } else if (vk_code == VK_UP) {
                        win32_process_keyboard_message(&keyboard_controller->action_up, is_down);
                    } else if (vk_code == VK_LEFT) {
                        win32_process_keyboard_message( &keyboard_controller->action_left, is_down);
                    } else if (vk_code == VK_DOWN) {
                        win32_process_keyboard_message(&keyboard_controller->action_down, is_down);
                    } else if (vk_code == VK_RIGHT) {
                        win32_process_keyboard_message(&keyboard_controller->action_right, is_down);
                    } else if (vk_code == VK_ESCAPE) {
                        win32_process_keyboard_message(&keyboard_controller->back, is_down);
                    } else if (vk_code == VK_SPACE) {
                        win32_process_keyboard_message(&keyboard_controller->start, is_down);
                    }
#if HANDMADE_INTERNAL
                    else if (vk_code == 'P') {
                        if (is_down) {
                            PAUSE = !PAUSE;
                        }
                    } else if (vk_code == 'L') {
                        if (is_down) {
                            if (state->input_playback_index == 0) {
                                if (state->input_recording_index == 0) {
                                    win32_end_input_playback(state);
                                    win32_begin_input_recording(state, 1);
                                } else {
                                    win32_end_input_recording(state);
                                    win32_begin_input_playback(state, 1);
                                }
                            } else {
                                win32_end_input_playback(state);
                            }
                        }
                    }
#endif
                }

                if (is_down) {
                    int alt_key_was_down = (int)(message.lParam & (1 << 29));
                    if (vk_code == VK_ESCAPE || (alt_key_was_down && vk_code == VK_F4)) {
                        RUNNING = false;
                    }
                    if (vk_code == VK_RETURN && alt_key_was_down) {
                        if (message.hwnd) {
                            toggle_fullscreen(message.hwnd);
                        }
                    }
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

#if 0
static void win32_debug_draw_vertical(Win32OffscreenBuffer *buffer, int x, int top, int bottom, uint32_t color) {
    if (top < 0) { top = 0;}
    if (bottom >= buffer->height) { bottom = buffer->height; }
    if (x >= 0 && x < buffer->width) {
        uint8_t *pixel = (uint8_t *) buffer->memory + x * buffer->bytes_per_pixel + top * buffer->pitch;
        for (int y = top; y < bottom; ++y) {
            *(uint32_t *) pixel = color;
            pixel += buffer->pitch;
        }
    }
}

inline void win32_draw_sound_buffer_marker(Win32OffscreenBuffer *buffer, Win32SoundOutput *sound_output, DWORD cursor, float c, int pad_x, int top, int bottom, uint32_t color) {
    int x = pad_x + (int) (c * (float) cursor);
    win32_debug_draw_vertical(buffer, x, top, bottom, color);
}

static void win32_debug_sync_display(
    Win32OffscreenBuffer *buffer,
    int marker_count,
    Win32DebugTimeMarker *markers,
    int current_marker_index,
    Win32SoundOutput *sound_output,
    float target_seconds_per_frame
) {
    int pad_x = 16;
    int pad_y = 16;
    int line_height = 64;

    float c = (float) (buffer->width - 2 * pad_x) / (float) sound_output->secondary_buffer_size;
    for (int i = 0; i < marker_count; ++i) {
        Win32DebugTimeMarker *this_marker = &markers[i];
        ASSERT(this_marker->output_play_cursor < sound_output->secondary_buffer_size);
        ASSERT(this_marker->output_write_cursor < sound_output->secondary_buffer_size);
        ASSERT(this_marker->output_location < sound_output->secondary_buffer_size);
        ASSERT(this_marker->output_byte_count < sound_output->secondary_buffer_size);
        ASSERT(this_marker->flip_play_cursor < sound_output->secondary_buffer_size);
        ASSERT(this_marker->flip_write_cursor < sound_output->secondary_buffer_size);

        DWORD play_color = 0xFFFFFFFF;
        DWORD write_color = 0xFFFF0000;
        DWORD expected_flip_color = 0xFFFFFF00;
        DWORD play_window_color = 0xFFFF00FF;
        int top = pad_y;
        int bottom = pad_y + line_height;
        if (i == current_marker_index) {
            top += line_height + pad_y;
            bottom += line_height + pad_y;

            int first_top = top;

            win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->output_play_cursor, c, pad_x, top, bottom, play_color);
            win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->output_write_cursor, c, pad_x, top, bottom, write_color);

            top += line_height + pad_y;
            bottom += line_height + pad_y;
            win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->output_location, c, pad_x, top, bottom, play_color);
            win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->output_location + this_marker->output_byte_count, c, pad_x, top, bottom, write_color);

            top += line_height + pad_y;
            bottom += line_height + pad_y;

            win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->expected_flip_play_cursor, c, pad_x, first_top, bottom, expected_flip_color);
        }
        win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->flip_play_cursor, c, pad_x, top, bottom, play_color);
        win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->flip_play_cursor + 480 * sound_output->bytes_per_sample, c, pad_x, top, bottom, play_window_color);
        win32_draw_sound_buffer_marker(buffer, sound_output, this_marker->flip_write_cursor, c, pad_x, top, bottom, write_color);
    }
}
#endif

static void win32_get_exe_file_name(Win32State *state) {
    GetModuleFileName(0, state->exe_file_name, sizeof(state->exe_file_name));
    state->one_past_last_exe_file_name_slash = state->exe_file_name;
    for (char *p = state->exe_file_name; *p; ++p) {
        if (*p == '\\') {
            state->one_past_last_exe_file_name_slash = p + 1;
        }
    }
}

int CALLBACK WinMain(HINSTANCE Instance, HINSTANCE PrevInstance, LPSTR CmdLine, int CmdShow) {
    Win32State win32_state = {};

    win32_get_exe_file_name(&win32_state);

    char source_game_code_dll_full_path[WIN32_STATE_FILE_NAME_COUNT];
    win32_build_exe_path_file_name(&win32_state, "handmade.dll", sizeof(source_game_code_dll_full_path), source_game_code_dll_full_path);

    char temp_game_code_dll_full_path[WIN32_STATE_FILE_NAME_COUNT];
    win32_build_exe_path_file_name(&win32_state, "handmade_temp.dll", sizeof(temp_game_code_dll_full_path), temp_game_code_dll_full_path);

    LARGE_INTEGER perf_count_frequency_result;
    QueryPerformanceFrequency(&perf_count_frequency_result);
    PERF_COUNT_FREQUENCY = perf_count_frequency_result.QuadPart;

    UINT desired_scheduler_ms = 1;
    int sleep_is_granular = timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR;

    win32_load_xinput();

    win32_resize_dib_section(&BACK_BUFFER, 960, 540);

    HINSTANCE instance = GetModuleHandle(0);

#ifdef HANDMADE_INTERNAL
    DEBUG_SHOW_CURSOR = true;
#endif
    LPCSTR class_name = "HandmadeHero";
    WNDCLASS window_class = {};
    window_class.style = CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = win32_main_window_proc;
    window_class.hInstance = instance;
    window_class.hCursor = LoadCursor(0, IDC_ARROW);
    window_class.lpszClassName = class_name;

    if (!RegisterClass(&window_class)) {
        return -1;
    }

    HWND window = CreateWindowEx(
        0, //WS_EX_TOPMOST | WS_EX_LAYERED,
        class_name, "Handmade Hero",
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT,
        CW_USEDEFAULT, CW_USEDEFAULT,
        0, 0, instance, 0
    );
    if (!window) {
        return -1;
    }

    int monitor_refresh_hz = 60;
    HDC device_context = GetDC(window);
    int win32_refresh_rate = GetDeviceCaps(device_context, VREFRESH);
    ReleaseDC(window, device_context);
    if (win32_refresh_rate > 1) {
        monitor_refresh_hz = win32_refresh_rate;
    }
    float game_update_hz = (float) monitor_refresh_hz / 2.0f;
    float target_seconds_per_frame = 1.0f / (float) game_update_hz;

    int samples_per_second = 48000;
    int bytes_per_sample = sizeof(int16_t) * 2;

    Win32SoundOutput sound_output = {};
    sound_output.samples_per_second = samples_per_second;
    sound_output.bytes_per_sample = bytes_per_sample;
    sound_output.secondary_buffer_size = samples_per_second * bytes_per_sample;
    sound_output.running_sample_index = 0;
    sound_output.safety_bytes = (uint32_t) ((float) sound_output.samples_per_second * (float) sound_output.bytes_per_sample / game_update_hz / 3.0f);

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
    game_memory.transient_storage_size = GIGABYTES(1);
    win32_state.total_size = game_memory.permanent_storage_size + game_memory.transient_storage_size;
    win32_state.game_memory_block = VirtualAlloc(
            base_address,
            win32_state.total_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE
    );
    game_memory.permanent_storage = win32_state.game_memory_block;
    game_memory.transient_storage = ((char *) game_memory.permanent_storage) + game_memory.permanent_storage_size;
    game_memory.debug_platform_free_file_memory = debug_platform_free_file_memory;
    game_memory.debug_platform_read_entire_file = debug_platform_read_entire_file;
    game_memory.debug_platform_write_entire_file = debug_platform_write_entire_file;

    for (int index = 1; index < ARRAY_COUNT(win32_state.replay_buffers); ++index) {
        Win32ReplayBuffer *replay_buffer = &win32_state.replay_buffers[index];
        win32_get_input_file_location(&win32_state, false, index, sizeof(replay_buffer->file_name), replay_buffer->file_name);
        replay_buffer->file_handle = CreateFileA(replay_buffer->file_name, GENERIC_WRITE | GENERIC_READ, 0, 0,
                                                 CREATE_ALWAYS, 0, 0);
        DWORD max_size_high = win32_state.total_size >> 32;
        DWORD max_size_low = win32_state.total_size & 0xFFFFFFFF;
        replay_buffer->memory_map = CreateFileMapping(replay_buffer->file_handle, 0, PAGE_READWRITE,
                                                      max_size_high, max_size_low,
                                                      0);
        replay_buffer->memory = MapViewOfFile(replay_buffer->memory_map, FILE_MAP_ALL_ACCESS, 0, 0, win32_state.total_size);
    }

    RUNNING = true;

    bool is_sound_valid = false;

    GameInput input[2] = {};
    GameInput *new_input = &input[0];
    GameInput *old_input = &input[1];

    LARGE_INTEGER last_counter = win32_get_wall_clock();
    LARGE_INTEGER flip_wall_clock = win32_get_wall_clock();

    DWORD64 last_cycle_count = __rdtsc();
    int debug_last_marker_index = 0;
    Win32DebugTimeMarker debug_markers[30] = {};

    Win32GameCode game = win32_load_game_code(source_game_code_dll_full_path, temp_game_code_dll_full_path);

    while (RUNNING) {
        new_input->dt = target_seconds_per_frame;

        FILETIME new_dll_last_write_time = win32_get_last_write_time(source_game_code_dll_full_path);
        if (CompareFileTime(&new_dll_last_write_time, &game.library_last_write_time) != 0) {
            win32_unload_game_code(&game);
            game = win32_load_game_code(source_game_code_dll_full_path, temp_game_code_dll_full_path);
        }

        GameControllerInput *old_keyboard_controller = GetController(old_input, 0);
        GameControllerInput *new_keyboard_controller = GetController(new_input, 0);
        *new_keyboard_controller = {};
        new_keyboard_controller->is_connected = true;
        for (int index = 0; index < ARRAY_COUNT(new_keyboard_controller->buttons); ++index) {
            new_keyboard_controller->buttons[index].ended_down = old_keyboard_controller->buttons[index].ended_down;
        }

        win32_process_pending_messages(new_keyboard_controller, &win32_state);

        if (!PAUSE) {
            // Note: Input
            POINT mouse_p;
            GetCursorPos(&mouse_p);
            ScreenToClient(window, &mouse_p);
            new_input->mouse_x = mouse_p.x;
            new_input->mouse_y = mouse_p.y;
            new_input->mouse_z = 0;
            win32_process_keyboard_message(&new_input->mouse_buttons[0], (GetKeyState(VK_LBUTTON) & (1 << 15)) != 0);
            win32_process_keyboard_message(&new_input->mouse_buttons[1], (GetKeyState(VK_MBUTTON) & (1 << 15)) != 0);
            win32_process_keyboard_message(&new_input->mouse_buttons[2], (GetKeyState(VK_RBUTTON) & (1 << 15)) != 0);
            win32_process_keyboard_message(&new_input->mouse_buttons[3], (GetKeyState(VK_XBUTTON1) & (1 << 15)) != 0);
            win32_process_keyboard_message(&new_input->mouse_buttons[4], (GetKeyState(VK_XBUTTON2) & (1 << 15)) != 0);

            DWORD max_controller_count = XUSER_MAX_COUNT;
            if (max_controller_count > ARRAY_COUNT(new_input->controllers) - 1) {
                max_controller_count = ARRAY_COUNT(new_input->controllers) - 1;
            }

            for (DWORD index = 0; index < max_controller_count; ++index) {
                DWORD x_input_controller_index = index + 1;
                GameControllerInput *old_controller = GetController(old_input, x_input_controller_index);
                GameControllerInput *new_controller = GetController(new_input, x_input_controller_index);

                XINPUT_STATE controller_state;
                if (XInputGetState(index, &controller_state) == ERROR_SUCCESS) {
                    new_controller->is_connected = true;
                    new_controller->is_analog = old_controller->is_analog;

                    XINPUT_GAMEPAD *pad = &controller_state.Gamepad;

                    new_controller->stick_average_x = win32_process_x_input_stick_value(pad->sThumbLX,
                                                                                        XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);
                    new_controller->stick_average_y = win32_process_x_input_stick_value(pad->sThumbLY,
                                                                                        XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);

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

                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_down, XINPUT_GAMEPAD_A,
                                                         &new_controller->action_down);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_right, XINPUT_GAMEPAD_B,
                                                         &new_controller->action_right);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_left, XINPUT_GAMEPAD_X,
                                                         &new_controller->action_left);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->action_up, XINPUT_GAMEPAD_Y,
                                                         &new_controller->action_up);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->left_shoulder,
                                                         XINPUT_GAMEPAD_LEFT_SHOULDER, &new_controller->left_shoulder);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->right_shoulder,
                                                         XINPUT_GAMEPAD_RIGHT_SHOULDER,
                                                         &new_controller->right_shoulder);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->back, XINPUT_GAMEPAD_BACK,
                                                         &new_controller->back);
                    win32_process_xinput_digitial_button(pad->wButtons, &old_controller->back, XINPUT_GAMEPAD_START,
                                                         &new_controller->start);
                } else {
                    new_controller->is_connected = false;
                }
            }

            GameOffscreenBuffer offscreen_buffer = {};
            offscreen_buffer.memory = BACK_BUFFER.memory;
            offscreen_buffer.width = BACK_BUFFER.width;
            offscreen_buffer.height = BACK_BUFFER.height;
            offscreen_buffer.pitch = BACK_BUFFER.pitch;
            offscreen_buffer.bytes_per_pixel = BACK_BUFFER.bytes_per_pixel;

            if (win32_state.input_recording_index) {
                win32_record_input(&win32_state, new_input);
            }
            if (win32_state.input_playback_index) {
                win32_playback_input(&win32_state, new_input);
            }
            if (game.game_update_and_render) {
                game.game_update_and_render(&game_memory, new_input, &offscreen_buffer);
            }

            LARGE_INTEGER audio_wall_clock = win32_get_wall_clock();
            float from_begin_to_audio_seconds = win32_get_seconds_elapsed(flip_wall_clock, audio_wall_clock);

            // NOTE: Sound
            DWORD play_cursor, write_cursor;
            if (SUCCEEDED(SECONDARY_BUFFER->GetCurrentPosition(&play_cursor, &write_cursor))) {
                if (!is_sound_valid) {
                    is_sound_valid = true;
                    sound_output.running_sample_index = write_cursor / sound_output.bytes_per_sample;
                }
                DWORD byte_to_lock = (sound_output.running_sample_index * sound_output.bytes_per_sample) %
                                     sound_output.secondary_buffer_size;

                DWORD expected_sound_bytes_per_frame =
                        (DWORD) ((float) sound_output.samples_per_second * (float) sound_output.bytes_per_sample / (float) game_update_hz);
                float seconds_left_until_flip = (target_seconds_per_frame - from_begin_to_audio_seconds);
                DWORD expected_bytes_until_flip = (DWORD)((seconds_left_until_flip / target_seconds_per_frame) * (float) expected_sound_bytes_per_frame);
                DWORD expected_frame_boundary_byte = play_cursor + expected_bytes_until_flip;
                DWORD safe_write_cursor = write_cursor;
                if (safe_write_cursor < play_cursor) {
                    safe_write_cursor += sound_output.secondary_buffer_size;
                }
                ASSERT(safe_write_cursor >= play_cursor);
                safe_write_cursor += sound_output.safety_bytes;
                bool audio_cart_is_low_latent = safe_write_cursor < expected_frame_boundary_byte;

                DWORD target_cursor = 0;
                if (audio_cart_is_low_latent) {
                    target_cursor = expected_frame_boundary_byte + expected_sound_bytes_per_frame;
                } else {
                    target_cursor = (write_cursor + expected_sound_bytes_per_frame + sound_output.safety_bytes);
                }
                target_cursor = target_cursor % sound_output.secondary_buffer_size;

                DWORD bytes_to_write = 0;
                if (byte_to_lock > target_cursor) {
                    bytes_to_write = (int) ((sound_output.secondary_buffer_size - byte_to_lock) + target_cursor);
                } else {
                    bytes_to_write = target_cursor - byte_to_lock;
                }

#if HANDMADE_INTERNAL
                Win32DebugTimeMarker *marker = &debug_markers[debug_last_marker_index];
                marker->output_play_cursor = play_cursor;
                marker->output_write_cursor = write_cursor;
                marker->output_location = byte_to_lock;
                marker->output_byte_count = bytes_to_write;
                marker->expected_flip_play_cursor = expected_frame_boundary_byte;

                DWORD unwrapped_write_cursor = write_cursor;
                if (unwrapped_write_cursor < play_cursor) {
                    unwrapped_write_cursor += sound_output.secondary_buffer_size;
                }
                DWORD audio_latency_bytes = unwrapped_write_cursor - play_cursor;
                float audio_latency_seconds = ((float) sound_output.samples_per_second / (float) audio_latency_bytes) /
                                              (float) sound_output.bytes_per_sample;
#endif

                GameSoundBuffer sound_buffer = {};
                sound_buffer.samples = samples;
                sound_buffer.sample_count = bytes_to_write / sound_output.bytes_per_sample;
                sound_buffer.samples_per_second = sound_output.samples_per_second;
                if (game.game_get_sound_samples) {
                    game.game_get_sound_samples(&game_memory, &sound_buffer);
                }
                win32_fill_sound_buffer(&sound_output, byte_to_lock, bytes_to_write, &sound_buffer);
            } else {
                is_sound_valid = false;
            }

            // NOTE: VSync
            LARGE_INTEGER work_counter = win32_get_wall_clock();
            float work_seconds_elapsed = win32_get_seconds_elapsed(last_counter, work_counter);
            float seconds_elapsed_for_frame = work_seconds_elapsed;
            if (seconds_elapsed_for_frame < target_seconds_per_frame) {
                if (sleep_is_granular) {
                    DWORD ms = (DWORD) (1000.0f * (target_seconds_per_frame - seconds_elapsed_for_frame));
                    if (ms > 0) {
                        Sleep(ms);
                    }
                }
                do {
                    seconds_elapsed_for_frame = win32_get_seconds_elapsed(last_counter, win32_get_wall_clock());
                } while (seconds_elapsed_for_frame < target_seconds_per_frame);
            } else {
                // TODO: MISSED FRAME RATE
                // TODO: Logging
            }

            LARGE_INTEGER end_counter = win32_get_wall_clock();
            float ms_per_frame = 1000.0f * win32_get_seconds_elapsed(last_counter, end_counter);
            last_counter = end_counter;

            device_context = GetDC(window);
            Win32WindowDimension dimension = win32_get_window_dimension(window);

            win32_display_buffer_in_window(
                    device_context,
                    dimension.width,
                    dimension.height,
                    &BACK_BUFFER
            );
            ReleaseDC(window, device_context);

            flip_wall_clock = win32_get_wall_clock();

#if HANDMADE_INTERNAL
            {
                if (SUCCEEDED(SECONDARY_BUFFER->GetCurrentPosition(&play_cursor, &write_cursor))) {
                    ASSERT(debug_last_marker_index < ARRAY_COUNT(debug_markers));
                    Win32DebugTimeMarker *marker = &debug_markers[debug_last_marker_index];
                    marker->flip_play_cursor = play_cursor;
                    marker->flip_write_cursor = write_cursor;
                }
            }
#endif

            GameInput *tmp_input = new_input;
            new_input = old_input;
            old_input = tmp_input;

#if 1
            DWORD64 end_cycle_count = __rdtsc();
            int cycles_elapsed = (int) (end_cycle_count - last_cycle_count);
            last_cycle_count = end_cycle_count;

            float fps = 0.0; //(float) PERF_COUNT_FREQUENCY / (float) counter_elapsed;
            char buf[1024];
            sprintf(buf, "%.2fms/f, %.2ff/s, %.2fmc/f\n", ms_per_frame, fps, (float) cycles_elapsed / 1000000.0f);
            OutputDebugString(buf);
#endif

#if HANDMADE_INTERNAL
            debug_last_marker_index++;
            if (debug_last_marker_index >= ARRAY_COUNT(debug_markers)) {
                debug_last_marker_index = 0;
            }
#endif
        }
    }

    return 0;
}
