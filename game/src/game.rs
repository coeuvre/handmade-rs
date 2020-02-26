use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

pub struct GameState {
    blue_offset: i32,
    green_offset: i32,
    tone_hz: u32,
}

impl GameState {
    pub fn init(&mut self) {
        self.tone_hz = 512;
    }

    pub fn update_and_render(
        &mut self,
        input: &GameInput,
        offscreen_buffer: &mut GameOffscreenBuffer,
    ) {
        for controller in input.controllers.iter() {
            if controller.is_analog == 1 {
                self.blue_offset += (4.0 * controller.stick_average_x) as i32;
                self.tone_hz = 512u32.overflowing_add((128.0 * controller.stick_average_y) as u32).0;
            } else {
                if controller.move_left.ended_down == 1 {
                    self.blue_offset -= 1;
                }
                if controller.move_right.ended_down == 1 {
                    self.blue_offset += 1;
                }
            }
    
            if controller.action_down.ended_down == 1 {
                self.green_offset += 1;
            }
        }
        
        render_weird_gradient(offscreen_buffer, self.blue_offset, self.green_offset);
    }

    pub fn get_sound_samples(&mut self, sound_buffer: &mut GameSoundBuffer) {
        game_output_sound(sound_buffer, self.tone_hz);
    }

}

fn render_weird_gradient(buffer: &mut GameOffscreenBuffer, x_offset: i32, y_offset: i32) {
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

fn game_output_sound(buffer: &mut GameSoundBuffer, tone_hz: u32) {
    static mut T_SINE: f32 = 0.0;
    let tone_volume = 3000;
    let wave_period = buffer.samples_per_second / tone_hz;

    let mut sample_out = buffer.samples as *mut i16;

    for _ in 0..buffer.sample_count {
        unsafe {
            let sine_value = T_SINE.sin();
            let sample_value = (sine_value * tone_volume as f32) as i16;
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);

            T_SINE += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
            if T_SINE > 2.0 * std::f32::consts::PI {
                T_SINE -= 2.0 * std::f32::consts::PI;
            }
        }
    }
}
