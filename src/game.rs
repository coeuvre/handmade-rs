pub struct GameMemory {
    pub is_initialized: bool,
    pub permanent_storage_size: usize,
    pub permanent_storage: *mut u8,
    pub transient_storage_size: usize,
    pub transient_storage: *mut u8,
}

pub struct GameOffScreenBuffer {
    pub memory: *mut u8,
    pub width: i32,
    pub height: i32,
    pub pitch: i32,
}

pub struct GameSoundOutputBuffer {
    pub samples: *mut i16,
    pub sample_count: u32,
    pub samples_per_second: u32,
}

pub struct GameButtonState {
    pub half_transition_count: u32,
    pub ended_down: bool,
}

pub struct GameControllerInput {
    pub is_analog: bool,

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

pub struct GameInput {
    pub controllers: [GameControllerInput; 4],
}

struct GameState {
    blue_offset: i32,
    green_offset: i32,
    tone_hz: u32,
}

pub fn game_update_and_render(
    memory: &mut GameMemory,
    input: &GameInput,
    buffer: &mut GameOffScreenBuffer,
    sound_buffer: &mut GameSoundOutputBuffer,
) {
    let game_state = unsafe { &mut *(memory.permanent_storage as *mut GameState) };
    if !memory.is_initialized {
        game_state.tone_hz = 256;

        memory.is_initialized = true;
    }

    game_output_sound(sound_buffer, game_state.tone_hz);

    let input0 = &input.controllers[0];

    if input0.is_analog {
        game_state.blue_offset += (4.0 * input0.end_x) as i32;
        game_state.tone_hz = 256u32.overflowing_add((128.0 * input0.end_y) as u32).0;
    }

    if input0.down.ended_down {
        game_state.green_offset += 1;
    }

    render_weird_gradient(buffer, game_state.blue_offset, game_state.green_offset);
}

fn render_weird_gradient(buffer: &mut GameOffScreenBuffer, x_offset: i32, y_offset: i32) {
    let mut row = buffer.memory;
    for y in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for x in 0..buffer.width {
            let b = x + x_offset;
            let g = y + y_offset;
            unsafe {
                *pixel = (((g & 0xFF) << 8) | (b & 0xFF)) as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe {
            row = row.offset(buffer.pitch as isize);
        }
    }
}

fn game_output_sound(buffer: &mut GameSoundOutputBuffer, tone_hz: u32) {
    static mut T_SINE: f32 = 0.0;
    let tone_volume = 3000;
    let wave_period = buffer.samples_per_second / tone_hz;

    let mut sample_out = buffer.samples;

    for _ in 0..buffer.sample_count {
        unsafe {
            let sine_value = T_SINE.sin();
            let sample_value = (sine_value * tone_volume as f32) as i16;
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);

            T_SINE += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
        }
    }
}
