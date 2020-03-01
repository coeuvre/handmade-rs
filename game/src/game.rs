use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
}

impl GameState {
    pub fn init(&mut self) {
    }

    pub fn update_and_render(
        &mut self,
        input: &GameInput,
        offscreen_buffer: &mut GameOffscreenBuffer,
    ) {
        let mut render_buffer: RenderBuffer = offscreen_buffer.into();
        let screen_width = render_buffer.width;
        let screen_height = render_buffer.height;
        draw_rectangle(&mut render_buffer, 0.0, 0.0, screen_width as f32, screen_height as f32, 0x00FF00FF);
        draw_rectangle(&mut render_buffer, input.mouse_x as f32, input.mouse_y as f32, input.mouse_x as f32 + 10.0, input.mouse_y as f32 + 10.0, 0xFFFFFFFF);
    }

    pub fn get_sound_samples(&mut self, sound_buffer: &mut GameSoundBuffer) {
        self.game_output_sound(sound_buffer, 400);
    }

    fn game_output_sound(&mut self, _buffer: &mut GameSoundBuffer, _tone_hz: u32) {
        // let tone_volume = 3000;
        // let wave_period = buffer.samples_per_second / tone_hz;
        //
        // let mut sample_out = buffer.samples as *mut i16;
        //
        // for _ in 0..buffer.sample_count {
        //     unsafe {
        //         let sine_value = self.t_sine.sin();
        //         let sample_value = (sine_value * tone_volume as f32) as i16;
        //         (*sample_out) = sample_value;
        //         sample_out = sample_out.add(1);
        //         (*sample_out) = sample_value;
        //         sample_out = sample_out.add(1);
        //
        //         self.t_sine += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
        //         if self.t_sine > 2.0 * std::f32::consts::PI {
        //             self.t_sine -= 2.0 * std::f32::consts::PI;
        //         }
        //     }
        // }
    }

}

impl<'a> Into<RenderBuffer<'a>> for &'a mut GameOffscreenBuffer {
    fn into(self) -> RenderBuffer<'a> {
        RenderBuffer {
            bytes: unsafe { std::slice::from_raw_parts_mut(self.memory as *mut u8, (self.pitch * self.height) as usize) },
            width: self.width as usize,
            height: self.height as usize,
            pitch: self.pitch as usize,
            bytes_per_pixel: self.bytes_per_pixel as usize,
        }
    }
}
