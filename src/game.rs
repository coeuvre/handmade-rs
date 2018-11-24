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

pub fn game_update_and_render(buffer: &mut GameOffScreenBuffer, sound_buffer: &mut GameSoundOutputBuffer) {
    game_output_sound(sound_buffer);
    render_weird_gradient(buffer, 0, 0);
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

fn game_output_sound(buffer: &mut GameSoundOutputBuffer) {
    static mut t_sine: f32 = 0.0;
    let tone_volume = 3000;
    let tone_hz = 256;
    let wave_period = buffer.samples_per_second / tone_hz;

    let mut sample_out = buffer.samples;

    for sample_index in 0..buffer.sample_count {
        unsafe {
        let sine_value = t_sine.sin();
        let sample_value = (sine_value * tone_volume as f32) as i16;
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);

            t_sine += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
        }
    }

}
