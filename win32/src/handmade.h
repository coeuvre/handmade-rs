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

DebugReadFileResult debug_platform_read_entire_file(char *filename);
void debug_platform_free_file_memory(void *memory);

int debug_platform_write_entire_file(char *filename, uint32_t memory_size, void *memory);
#endif

typedef struct GameMemory {
    int is_initialized;
    size_t permanent_storage_size;
    void *permanent_storage;
    size_t transient_storage_size;
    void *transient_storage;
} GameMemory;

typedef struct GameOffscreenBuffer {
    void *memory;
    int width;
    int height;
    int pitch;
} GameOffscreenBuffer;

typedef struct GameSoundBuffer {
    void *samples;
    int sample_count;
    int samples_per_second;
} GameSoundBuffer;


typedef struct GameButtonState {
    int half_transition_count;
    int ended_down;
} GameButtonState;

typedef struct GameControllerInput {
    int is_analog;

    float start_x;
    float start_y;

    float min_x;
    float min_y;

    float max_x;
    float max_y;

    float end_x;
    float end_y;

    union {
        GameButtonState buttons[6];
        struct {
            GameButtonState up;
            GameButtonState down;
            GameButtonState left;
            GameButtonState right;
            GameButtonState left_shoulder;
            GameButtonState right_shoulder;
        };
    };

} GameControllerInput;

typedef struct GameInput {
    GameControllerInput controllers[4];
} GameInput;

void game_update_and_render(GameMemory *memory, GameInput *input, GameOffscreenBuffer *offscreen_buffer, GameSoundBuffer *sound_buffer);

#ifdef __cplusplus
}
#endif

#endif // HANDMADE_H