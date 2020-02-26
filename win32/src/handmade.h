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
    GameControllerInput controllers[5];
} GameInput;

inline GameControllerInput *GetController(GameInput *input, int index) {
    ASSERT(index < ARRAY_COUNT(input->controllers));
    GameControllerInput *controller = &input->controllers[index];
    return controller;
}

void game_update_and_render(GameMemory *memory, GameInput *input, GameOffscreenBuffer *offscreen_buffer);
void game_get_sound_samples(GameMemory *memory, GameSoundBuffer *sound_buffer);

#ifdef __cplusplus
}
#endif

#endif // HANDMADE_H