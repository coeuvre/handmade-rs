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

pub fn game_update_and_render(
    input: &GameInput,
    buffer: &mut GameOffScreenBuffer,
    sound_buffer: &mut GameSoundOutputBuffer,
) {
    static mut BLUE_OFFSET: i32 = 0;
    static mut GREEN_OFFSET: i32 = 0;
    static mut TONE_HZ: u32 = 256;

    unsafe { game_output_sound(sound_buffer, TONE_HZ); }

    let input0 = &input.controllers[0];

    if input0.is_analog {
        unsafe {
            BLUE_OFFSET += (4.0 * input0.end_x) as i32;
            TONE_HZ = 256u32.overflowing_add((128.0 * input0.end_y) as u32).0;
        }
    }

    if input0.down.ended_down {
        unsafe { GREEN_OFFSET += 1; }
    }

    unsafe {
        render_weird_gradient(buffer, BLUE_OFFSET, GREEN_OFFSET);
    }
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
