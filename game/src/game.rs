use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
    player_tile_map_x: i32,
    player_tile_map_y: i32,
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
        let tile_map_00 = TileMap {
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_map_01 = TileMap {
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_map_10 = TileMap {
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_map_11 = TileMap {
            #[rustfmt::skip]
            tiles: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        };
        let tile_map_upper_left_x = -30.0;
        let tile_map_upper_left_y = 0.0;
        let world = World {
            tile_side_in_meters: 1.4,
            tile_side_in_pixels: 60,
            upper_left_x: tile_map_upper_left_x,
            upper_left_y: tile_map_upper_left_y,
            tile_count_x: tile_map_count_x,
            tile_count_y: tile_map_count_y,
            tile_map_count_x: 2,
            tile_map_count_y: 2,
            tile_maps: vec![tile_map_00, tile_map_10, tile_map_01, tile_map_11],
        };
        for tile_map in world.tile_maps.iter() {
            assert_eq!(
                tile_map.tiles.len(),
                (world.tile_count_x * world.tile_count_y) as usize
            );
        }

        let player_width = 0.75 * world.tile_side_in_pixels as f32;
        let player_height = world.tile_side_in_pixels as f32;

        let tile_map = world
            .get_tile_map(self.player_tile_map_x, self.player_tile_map_y)
            .unwrap();

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

                let player_pos = RawPosition {
                    tile_map_x: self.player_tile_map_x,
                    tile_map_y: self.player_tile_map_y,
                    x_within_tile_map: new_player_x,
                    y_within_tile_map: new_player_y,
                };
                let mut player_left = player_pos;
                player_left.x_within_tile_map -= 0.5 * player_width;
                let mut player_right = player_pos;
                player_right.x_within_tile_map += 0.5 * player_width;
                if world.is_world_point_empty(player_left)
                    && world.is_world_point_empty(player_right)
                    && world.is_world_point_empty(player_pos)
                {
                    let can_pos = world.get_canonical_position(player_pos);
                    self.player_tile_map_x = can_pos.tile_map_x;
                    self.player_tile_map_y = can_pos.tile_map_y;
                    self.player_x = world.upper_left_x
                        + can_pos.tile_x as f32 * world.tile_side_in_pixels as f32
                        + can_pos.x_within_tile;
                    self.player_y = world.upper_left_y
                        + can_pos.tile_y as f32 * world.tile_side_in_pixels as f32
                        + can_pos.y_within_tile;
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
            .chunks_exact(tile_map.tile_count_x as usize)
            .enumerate()
        {
            for (x, tile_value) in row.iter().enumerate() {
                let gray = if *tile_value == 1 { 1.0 } else { 0.5 };

                let min_x = tile_map.upper_left_x + x as f32 * tile_map.tile_side_in_pixels as f32;
                let min_y = tile_map.upper_left_y + y as f32 * tile_map.tile_side_in_pixels as f32;
                let max_x = min_x + tile_map.tile_side_in_pixels as f32;
                let max_y = min_y + tile_map.tile_side_in_pixels as f32;
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
    pub tiles: Vec<i32>,
}

impl TileMap {}

struct TileMapView<'a> {
    pub tiles: &'a Vec<i32>,
    pub upper_left_x: f32,
    pub upper_left_y: f32,
    pub tile_count_x: i32,
    pub tile_count_y: i32,
    pub tile_side_in_pixels: i32,
}

impl<'a> TileMapView<'a> {
    pub fn is_tile_map_point_empty(&self, test_tile_x: i32, test_tile_y: i32) -> bool {
        let mut is_empty = false;
        if test_tile_x >= 0
            && (test_tile_x) < self.tile_count_x
            && test_tile_y >= 0
            && (test_tile_y) < self.tile_count_y
        {
            let tile_value = self
                .tiles
                .get((test_tile_y * self.tile_count_x + test_tile_x) as usize)
                .unwrap();
            is_empty = *tile_value == 0;
        }
        is_empty
    }
}

struct World {
    pub tile_side_in_meters: f32,
    pub tile_side_in_pixels: i32,

    pub upper_left_x: f32,
    pub upper_left_y: f32,
    pub tile_count_x: i32,
    pub tile_count_y: i32,
    pub tile_map_count_x: i32,
    pub tile_map_count_y: i32,
    pub tile_maps: Vec<TileMap>,
}

impl World {
    pub fn get_tile_map(&self, tile_map_x: i32, tile_map_y: i32) -> Option<TileMapView> {
        self.tile_maps
            .get((tile_map_y * self.tile_map_count_x + tile_map_x) as usize)
            .map(|tile_map| TileMapView {
                tiles: &tile_map.tiles,
                upper_left_x: self.upper_left_x,
                upper_left_y: self.upper_left_y,
                tile_count_x: self.tile_count_x,
                tile_count_y: self.tile_count_y,
                tile_side_in_pixels: self.tile_side_in_pixels,
            })
    }

    pub fn get_canonical_position(&self, pos: RawPosition) -> CanonicalPosition {
        let mut tile_map_x = pos.tile_map_x;
        let mut tile_map_y = pos.tile_map_y;

        let x = pos.x_within_tile_map - self.upper_left_x;
        let y = pos.y_within_tile_map - self.upper_left_y;
        let mut tile_x = (x / self.tile_side_in_pixels as f32).floor() as i32;
        let mut tile_y = (y / self.tile_side_in_pixels as f32).floor() as i32;
        let x_within_tile = x - tile_x as f32 * self.tile_side_in_pixels as f32;
        let y_within_tile = y - tile_y as f32 * self.tile_side_in_pixels as f32;

        assert!(x_within_tile >= 0.0 && x_within_tile < self.tile_side_in_pixels as f32);
        assert!(y_within_tile >= 0.0 && y_within_tile < self.tile_side_in_pixels as f32);

        while tile_x < 0 {
            tile_map_x -= 1;
            tile_x += self.tile_count_x;
        }

        while tile_x >= self.tile_count_x {
            tile_map_x += 1;
            tile_x -= self.tile_count_x;
        }

        while tile_y < 0 {
            tile_map_y -= 1;
            tile_y += self.tile_count_y;
        }

        while tile_y >= self.tile_count_y {
            tile_map_y += 1;
            tile_y -= self.tile_count_y;
        }

        CanonicalPosition {
            tile_map_x,
            tile_map_y,
            tile_x,
            tile_y,
            x_within_tile,
            y_within_tile,
        }
    }

    pub fn is_world_point_empty(&self, pos: RawPosition) -> bool {
        let can_pos = self.get_canonical_position(pos);
        let tile_map = self
            .get_tile_map(can_pos.tile_map_x, can_pos.tile_map_y)
            .unwrap();
        tile_map.is_tile_map_point_empty(can_pos.tile_x, can_pos.tile_y)
    }
}

#[derive(Copy, Clone)]
struct CanonicalPosition {
    pub tile_map_x: i32,
    pub tile_map_y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub x_within_tile: f32,
    pub y_within_tile: f32,
}

#[derive(Copy, Clone)]
struct RawPosition {
    pub tile_map_x: i32,
    pub tile_map_y: i32,
    pub x_within_tile_map: f32,
    pub y_within_tile_map: f32,
}
