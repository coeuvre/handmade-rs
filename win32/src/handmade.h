#ifndef HANDMADE_H
#define HANDMADE_H

#include <stdint.h>
#include <stdio.h>
#include <assert.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ASSERT(expr) assert(expr)
#define ARRAY_COUNT(array) (sizeof(array) / sizeof((array)[0]))

#define KILOBYTES(value) ((value) * 1024LL)
#define MEGABYTES(value) (KILOBYTES(value) * 1024LL)
#define GIGABYTES(value) (MEGABYTES(value) * 1024LL)
#define TERABYTES(value) (GIGABYTES(value) * 1024LL)


inline uint32_t
safe_truncate_uint64(uint64_t value) {
    ASSERT(value <= 0xFFFFFFFF);
    return (uint32_t) value;
}

#ifdef HANDMADE_INTERNAL
typedef struct DebugReadFileResult {
    uint32_t content_size;
    void *contents;
} DebugReadFileResult;

#define DEBUG_PLATFORM_READ_ENTIRE_FILE(name) DebugReadFileResult name(char *file_name)
typedef DEBUG_PLATFORM_READ_ENTIRE_FILE(DebugPlatformReadEntireFile);

#define DEBUG_PLATFORM_FREE_FILE_MEMORY(name) void name(void *memory)
typedef DEBUG_PLATFORM_FREE_FILE_MEMORY(DebugPlatformFreeFileMemory);

#define DEBUG_PLATFORM_WRITE_ENTIRE_FILE(name) int name(char *file_name, uint32_t memory_size, void *memory)
typedef DEBUG_PLATFORM_WRITE_ENTIRE_FILE(DebugPlatformWriteEntireFile);
#endif

typedef struct GameMemory {
    int is_initialized;
    size_t permanent_storage_size;
    void *permanent_storage;
    size_t transient_storage_size;
    void *transient_storage;

    DebugPlatformReadEntireFile *debug_platform_read_entire_file;
    DebugPlatformFreeFileMemory *debug_platform_free_file_memory;
    DebugPlatformWriteEntireFile *debug_platform_write_entire_file;
} GameMemory;

typedef struct GameOffscreenBuffer {
    void *memory;
    int width;
    int height;
    int pitch;
    int bytes_per_pixel;
} GameOffscreenBuffer;

typedef struct GameSoundBuffer {
    void *samples;
    uint32_t sample_count;
    uint32_t samples_per_second;
} GameSoundBuffer;


typedef struct GameButtonState {
    int half_transition_count;
    int ended_down;
} GameButtonState;

typedef struct GameControllerInput {
    int is_connected;
    int is_analog;

    float stick_average_x;
    float stick_average_y;

    union {
        GameButtonState buttons[12];
        struct {
            GameButtonState move_up;
            GameButtonState move_down;
            GameButtonState move_left;
            GameButtonState move_right;

            GameButtonState action_up;
            GameButtonState action_down;
            GameButtonState action_left;
            GameButtonState action_right;

            GameButtonState left_shoulder;
            GameButtonState right_shoulder;

            GameButtonState back;
            GameButtonState start;
        };
    };

} GameControllerInput;

typedef struct GameInput {
    GameButtonState mouse_buttons[5];
    int mouse_x;
    int mouse_y;
    int mouse_z;

    GameControllerInput controllers[5];
} GameInput;

inline GameControllerInput *GetController(GameInput *input, int index) {
    ASSERT(index < ARRAY_COUNT(input->controllers));
    GameControllerInput *controller = &input->controllers[index];
    return controller;
}

#define GAME_UPDATE_AND_RENDER(name) void name(GameMemory *memory, GameInput *input, GameOffscreenBuffer *offscreen_buffer)
typedef GAME_UPDATE_AND_RENDER(GameUpdateAndRender);

#define GAME_GET_SOUND_SAMPLES(name) void name(GameMemory *memory, GameSoundBuffer *sound_buffer)
typedef GAME_GET_SOUND_SAMPLES(GameGetSoundSamples);

#ifdef __cplusplus
}
#endif

#endif // HANDMADE_H