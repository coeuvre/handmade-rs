use std::os::raw::{c_int, c_void};

mod game;

#[repr(C)]
pub struct GameMemory {
    is_initialized: c_int,
    permanent_storage_size: isize,
    permanent_storage: *mut c_void,
    transient_storage_size: isize,
    transient_storage: *mut c_void,
}

#[repr(C)]
pub struct GameOffscreenBuffer {
    pub memory: *mut c_void,
    pub width: c_int,
    pub height: c_int,
    pub pitch: c_int,
}

#[repr(C)]
pub struct GameSoundBuffer {
    pub samples: *mut c_void,
    pub sample_count: c_int,
    pub samples_per_second: c_int,
}

#[repr(C)]
pub struct GameButtonState {
    pub half_transition_count: c_int,
    pub ended_down: c_int,
}

#[repr(C)]
pub struct GameControllerInput {
    pub is_analog: c_int,

    pub start_x: f32,
    pub start_y: f32,

    pub min_x: f32,
    pub min_y: f32,

    pub max_x: f32,
    pub max_y: f32,

    pub end_x: f32,
    pub end_y: f32,

    pub up: GameButtonState,
    pub down: GameButtonState,
    pub left: GameButtonState,
    pub right: GameButtonState,
    pub left_shoulder: GameButtonState,
    pub right_shoulder: GameButtonState,
}

#[repr(C)]
pub struct GameInput {
    pub controllers: [GameControllerInput; 4],
}

#[no_mangle]
pub unsafe extern "C" fn game_update_and_render(
    memory: *mut GameMemory,
    input: *const GameInput,
    offscreen_buffer: *mut GameOffscreenBuffer,
    sound_buffer: *mut GameSoundBuffer,
) {
    let memory = &mut *memory;
    let game_state = &mut *(memory.permanent_storage as *mut game::GameState);
    if memory.is_initialized == 0 {
        game_state.init();
        memory.is_initialized = 1;
    }

    game_state.update_and_render(&* input, &mut *offscreen_buffer, &mut *sound_buffer);
}
