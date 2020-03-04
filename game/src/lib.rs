#![no_std]

extern crate libc;
extern crate software_renderer;

use core::ffi::c_void;
use libc::c_int;

mod game;
mod random;
mod tile_map;

use game::{GameState, MemoryArena};

#[repr(C)]
pub struct GameMemory {
    is_initialized: c_int,
    permanent_storage_size: usize,
    permanent_storage: *mut c_void,
    transient_storage_size: usize,
    transient_storage: *mut c_void,
}

#[repr(C)]
pub struct GameOffscreenBuffer {
    pub memory: *mut c_void,
    pub width: c_int,
    pub height: c_int,
    pub pitch: c_int,
    pub bytes_per_pixel: c_int,
}

#[repr(C)]
pub struct GameSoundBuffer {
    pub samples: *mut c_void,
    pub sample_count: u32,
    pub samples_per_second: u32,
}

#[repr(C)]
pub struct GameButtonState {
    pub half_transition_count: c_int,
    pub ended_down: c_int,
}

#[repr(C)]
pub struct GameControllerInput {
    pub is_connected: c_int,
    pub is_analog: c_int,
    pub stick_average_x: f32,
    pub stick_average_y: f32,

    pub move_up: GameButtonState,
    pub move_down: GameButtonState,
    pub move_left: GameButtonState,
    pub move_right: GameButtonState,

    pub action_up: GameButtonState,
    pub action_down: GameButtonState,
    pub action_left: GameButtonState,
    pub action_right: GameButtonState,

    pub left_shoulder: GameButtonState,
    pub right_shoulder: GameButtonState,

    pub back: GameButtonState,
    pub start: GameButtonState,
}

#[repr(C)]
pub struct GameInput {
    pub mouse_buttons: [GameButtonState; 5],
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_z: i32,
    pub dt: f32,
    pub controllers: [GameControllerInput; 4],
}

#[no_mangle]
pub unsafe extern "C" fn game_update_and_render(
    memory: *mut GameMemory,
    input: *const GameInput,
    offscreen_buffer: *mut GameOffscreenBuffer,
) {
    let memory = &mut *memory;
    let mut permanent_storage = MemoryArena::from_raw_parts(
        memory.permanent_storage as *mut u8,
        memory.permanent_storage_size,
    );
    let mut game_state = permanent_storage.alloc_uninit();
    if memory.is_initialized == 0 {
        *game_state = GameState::new(&mut permanent_storage);
        memory.is_initialized = 1;
    }

    game_state.update_and_render(&*input, &mut *offscreen_buffer);
}

#[no_mangle]
pub unsafe extern "C" fn game_get_sound_samples(
    memory: *mut GameMemory,
    sound_buffer: *mut GameSoundBuffer,
) {
    let memory = &mut *memory;
    let game_state = &mut *(memory.permanent_storage as *mut game::GameState);
    game_state.get_sound_samples(&mut *sound_buffer)
}
