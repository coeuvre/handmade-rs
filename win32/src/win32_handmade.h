#ifndef WIN32_HANDMADE
#define WIN32_HANDMADE

#include <Windows.h>
#include <Xinput.h>
#include <dsound.h>

struct Win32WindowDimension {
    int width;
    int height;
};

struct Win32GameCode {
    HMODULE library;
    FILETIME library_last_write_time;
    GameUpdateAndRender *game_update_and_render;
    GameGetSoundSamples *game_get_sound_samples;

    int is_valid;
};

struct Win32OffscreenBuffer {
    BITMAPINFO info;
    void *memory;
    int width;
    int height;
    int pitch;
    int bytes_per_pixel;
};

struct Win32SoundOutput {
    int samples_per_second;
    int bytes_per_sample;
    DWORD secondary_buffer_size;
    uint32_t running_sample_index;
    uint32_t safety_bytes;
};

struct Win32DebugTimeMarker {
    DWORD output_play_cursor;
    DWORD output_write_cursor;
    DWORD output_location;
    DWORD output_byte_count;

    DWORD expected_flip_play_cursor;
    DWORD flip_play_cursor;
    DWORD flip_write_cursor;
};

#define WIN32_STATE_FILE_NAME_COUNT MAX_PATH

struct Win32ReplayBuffer {
    char file_name[WIN32_STATE_FILE_NAME_COUNT];
    HANDLE file_handle;
    HANDLE memory_map;
    void *memory;
};

struct Win32State {
    size_t total_size;
    void *game_memory_block;
    Win32ReplayBuffer replay_buffers[4];

    HANDLE input_recording_handle;
    int input_recording_index = 0;

    HANDLE input_playback_handle;
    int input_playback_index = 0;

    char exe_file_name[WIN32_STATE_FILE_NAME_COUNT];
    char *one_past_last_exe_file_name_slash;
};

bool RUNNING;
bool PAUSE;
bool DEBUG_SHOW_CURSOR;
WINDOWPLACEMENT WINDOW_POSITION = {sizeof(WINDOW_POSITION)};

Win32OffscreenBuffer BACK_BUFFER;
LPDIRECTSOUNDBUFFER SECONDARY_BUFFER;

#endif // WIN32_HANDMADE
