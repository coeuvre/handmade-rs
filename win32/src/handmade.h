#ifndef HANDMADE_H
#define HANDMADE_H

#ifdef __cplusplus
extern "C" {
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

    GameButtonState up;
    GameButtonState down;
    GameButtonState left;
    GameButtonState right;
    GameButtonState left_shoulder;
    GameButtonState right_shoulder;
} GameControllerInput;

typedef struct GameInput {
    GameControllerInput controllers[4];
} GameInput;

#define ARRAY_COUNT(array) (sizeof(array) / sizeof((array)[0]))


void game_update_and_render(GameMemory *memory, GameInput *input, GameOffscreenBuffer *offscreen_buffer, GameSoundBuffer *sound_buffer);

#ifdef __cplusplus
}
#endif

#endif // HANDMADE_H