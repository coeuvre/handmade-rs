use software_renderer::render_weird_gradient;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
    blue_offset: i32,
    green_offset: i32,
    tone_hz: u32,
    t_sine: f32,

    player_x: i32,
    player_y: i32,
    t_jump: f32,
}

impl GameState {
    pub fn init(&mut self) {
        self.tone_hz = 512;

        self.player_x = 100;
        self.player_y = 100;
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
                if controller.move_left.ended_down != 0 {
                    self.blue_offset -= 1;
                }
                if controller.move_right.ended_down != 0 {
                    self.blue_offset += 1;
                }
            }

            self.player_x += (4.0 * controller.stick_average_x) as i32;
            self.player_y -= (4.0 * controller.stick_average_y) as i32;

            if self.t_jump > 0.0 {
                self.player_y -= (10.0 * self.t_jump.sin()) as i32;
            }
            if controller.action_down.ended_down != 0 {
                self.t_jump = 1.0;
            }

            self.t_jump -= 0.033;
        }

        render_weird_gradient(offscreen_buffer.memory as *mut u8, offscreen_buffer.width, offscreen_buffer.height, offscreen_buffer.pitch, self.blue_offset, self.green_offset);
        render_player(offscreen_buffer, self.player_x, self.player_y);
        render_player(offscreen_buffer, input.mouse_x, input.mouse_y);
        for (index, mouse_button) in input.mouse_buttons.iter().enumerate() {
            if mouse_button.ended_down != 0 {
                render_player(offscreen_buffer, (10 + index * 20) as i32, 10);
            }
        }
    }

    pub fn get_sound_samples(&mut self, sound_buffer: &mut GameSoundBuffer) {
        self.game_output_sound(sound_buffer, self.tone_hz);
    }

    fn game_output_sound(&mut self, buffer: &mut GameSoundBuffer, tone_hz: u32) {
        let tone_volume = 3000;
        let wave_period = buffer.samples_per_second / tone_hz;

        let mut sample_out = buffer.samples as *mut i16;

        for _ in 0..buffer.sample_count {
            unsafe {
                let sine_value = self.t_sine.sin();
                let sample_value = (sine_value * tone_volume as f32) as i16;
                (*sample_out) = sample_value;
                sample_out = sample_out.add(1);
                (*sample_out) = sample_value;
                sample_out = sample_out.add(1);

                self.t_sine += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
                if self.t_sine > 2.0 * std::f32::consts::PI {
                    self.t_sine -= 2.0 * std::f32::consts::PI;
                }
            }
        }
    }

}

fn render_player(buffer: &mut GameOffscreenBuffer, x: i32, y: i32) {
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(buffer.memory as *mut u8, (buffer.pitch * buffer.height) as usize)
    };

    let mut row_opt = bytes.get_mut((x * buffer.bytes_per_pixel + y * buffer.pitch) as usize..);

    for _ in y..y + 10 {
        if let Some(row) = row_opt {
            let mut pixels_opt = row.get_mut(0..);

            for _ in x..x + 10 {
                if let Some(pixels) = pixels_opt {
                    let color = &mut pixels[0..4];
                    color[0] = 0xFF;
                    color[1] = 0xFF;
                    color[2] = 0xFF;
                    color[3] = 0xFF;

                    pixels_opt = pixels.get_mut(4..);
                } else {
                    break;
                }
            }
            row_opt = row.get_mut(buffer.pitch as usize..);
        } else {
            break;
        }
    }
}
