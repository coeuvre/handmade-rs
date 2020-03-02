use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
    player_x: f32,
    player_y: f32,
}

impl GameState {
    pub fn init(&mut self) {
        self.player_x = 150.0;
        self.player_y = 150.0;
    }

    pub fn update_and_render(
        &mut self,
        input: &GameInput,
        offscreen_buffer: &mut GameOffscreenBuffer,
    ) {
        let tile_map_count_x = 17;
        let tile_map_count_y = 9;
        let tile_map_upper_left_x = -30.0;
        let tile_map_upper_left_y = 0.0;
        let tile_map_tile_width = 60.0;
        let tile_map_tile_height = 60.0;
        let tile_map_0 = TileMap {
            count_x: tile_map_count_x,
            count_y: tile_map_count_y,
            upper_left_x: tile_map_upper_left_x,
            upper_left_y: tile_map_upper_left_y,
            tile_width: tile_map_tile_width,
            tile_height: tile_map_tile_height,
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_map_1 = TileMap {
            count_x: tile_map_count_x,
            count_y: tile_map_count_y,
            upper_left_x: tile_map_upper_left_x,
            upper_left_y: tile_map_upper_left_y,
            tile_width: tile_map_tile_width,
            tile_height: tile_map_tile_height,
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_maps = vec![tile_map_0, tile_map_1];
        for tile_map in tile_maps.iter() {
            assert_eq!(
                tile_map.tiles.len(),
                (tile_map.count_x * tile_map.count_y) as usize
            );
        }

        let tile_map = &tile_maps[0];

        let player_width = 0.75 * tile_map.tile_width;
        let player_height = tile_map.tile_height;

        for controller in input.controllers.iter() {
            if controller.is_analog == 0 {
                let mut d_player_x = 0.0;
                let mut d_player_y = 0.0;
                if controller.move_up.ended_down != 0 {
                    d_player_y = -1.0;
                }
                if controller.move_down.ended_down != 0 {
                    d_player_y = 1.0;
                }
                if controller.move_left.ended_down != 0 {
                    d_player_x = -1.0;
                }
                if controller.move_right.ended_down != 0 {
                    d_player_x = 1.0;
                }
                d_player_x *= 128.0;
                d_player_y *= 128.0;
                let new_player_x = self.player_x + d_player_x * input.dt;
                let new_player_y = self.player_y + d_player_y * input.dt;

                if tile_map.is_tile_map_point_empty(new_player_x - 0.5 * player_width, new_player_y)
                    && tile_map
                        .is_tile_map_point_empty(new_player_x + 0.5 * player_width, new_player_y)
                    && tile_map.is_tile_map_point_empty(new_player_x, new_player_y)
                {
                    self.player_x = new_player_x;
                    self.player_y = new_player_y;
                }
            }
        }

        let mut render_buffer: RenderBuffer = offscreen_buffer.into();
        let screen_width = render_buffer.width;
        let screen_height = render_buffer.height;

        draw_rectangle(
            &mut render_buffer,
            0.0,
            0.0,
            screen_width as f32,
            screen_height as f32,
            0.0,
            0.0,
            0.0,
        );

        for (y, row) in tile_map
            .tiles
            .chunks_exact(tile_map.count_x as usize)
            .enumerate()
        {
            for (x, tile_value) in row.iter().enumerate() {
                let gray = if *tile_value == 1 { 1.0 } else { 0.5 };

                let min_x = tile_map.upper_left_x + x as f32 * tile_map.tile_width;
                let min_y = tile_map.upper_left_y + y as f32 * tile_map.tile_height;
                let max_x = min_x + tile_map.tile_width;
                let max_y = min_y + tile_map.tile_height;
                draw_rectangle(
                    &mut render_buffer,
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                    gray,
                    gray,
                    gray,
                );
            }
        }

        let player_r = 1.0;
        let player_g = 1.0;
        let player_b = 0.0;
        let player_left = self.player_x - 0.5 * player_width;
        let player_top = self.player_y - player_height;
        let player_right = player_left + player_width;
        let player_bottom = player_top + player_height;
        draw_rectangle(
            &mut render_buffer,
            player_left,
            player_top,
            player_right,
            player_bottom,
            player_r,
            player_g,
            player_b,
        );
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

impl<'a> From<&'a mut GameOffscreenBuffer> for RenderBuffer<'a> {
    fn from(buffer: &'a mut GameOffscreenBuffer) -> Self {
        assert!(buffer.pitch >= buffer.width * buffer.bytes_per_pixel);
        RenderBuffer {
            bytes: unsafe {
                std::slice::from_raw_parts_mut(
                    buffer.memory as *mut u8,
                    (buffer.pitch * buffer.height) as usize,
                )
            },
            width: buffer.width as usize,
            height: buffer.height as usize,
            pitch: buffer.pitch as usize,
            bytes_per_pixel: buffer.bytes_per_pixel as usize,
        }
    }
}

#[derive(Clone)]
struct TileMap {
    pub count_x: i32,
    pub count_y: i32,

    pub upper_left_x: f32,
    pub upper_left_y: f32,
    pub tile_width: f32,
    pub tile_height: f32,

    pub tiles: Vec<i32>,
}

impl TileMap {
    fn is_tile_map_point_empty(&self, test_x: f32, test_y: f32) -> bool {
        let test_tile_x = ((test_x - self.upper_left_x) / self.tile_width) as i32;
        let test_tile_y = ((test_y - self.upper_left_y) / self.tile_height) as i32;

        let mut is_empty = false;
        if test_tile_x >= 0
            && (test_tile_x) < self.count_x
            && test_tile_y >= 0
            && (test_tile_y) < self.count_y
        {
            let tile_value = self
                .tiles
                .get((test_tile_y * self.count_x + test_tile_x) as usize)
                .unwrap();
            is_empty = *tile_value == 0;
        }
        is_empty
    }
}
