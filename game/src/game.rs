use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
    player_p: CanonicalPosition,
}

impl GameState {
    pub fn init(&mut self) {
        self.player_p.tile_map_x = 0;
        self.player_p.tile_map_y = 0;
        self.player_p.tile_x = 3;
        self.player_p.tile_y = 3;
        self.player_p.tile_rel_x = 0.0;
        self.player_p.tile_rel_y = 0.0;
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
        let tile_side_in_meters = 1.4;
        let tile_side_in_pixels = 60;
        let world = World {
            tile_side_in_meters,
            tile_side_in_pixels,
            meters_to_pixels: tile_side_in_pixels as f32 / tile_side_in_meters,
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

        let player_height = 1.4;
        let player_width = 0.75 * player_height;

        let tile_map = world
            .get_tile_map(self.player_p.tile_map_x, self.player_p.tile_map_y)
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
                d_player_x *= 2.0;
                d_player_y *= 2.0;
                let mut new_player_p = self.player_p;
                new_player_p.tile_rel_x += d_player_x * input.dt;
                new_player_p.tile_rel_y += d_player_y * input.dt;
                new_player_p = world.recanonicalize_position(new_player_p);

                let mut player_left = new_player_p;
                player_left.tile_rel_x -= 0.5 * player_width;
                player_left = world.recanonicalize_position(player_left);

                let mut player_right = new_player_p;
                player_right.tile_rel_x += 0.5 * player_width;
                player_right = world.recanonicalize_position(player_right);

                if world.is_world_point_empty(player_left)
                    && world.is_world_point_empty(player_right)
                    && world.is_world_point_empty(new_player_p)
                {
                    self.player_p = new_player_p;
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
                let mut gray = 0.5;
                if *tile_value == 1 {
                    gray = 1.0;
                }

                if x == self.player_p.tile_x as usize && y == self.player_p.tile_y as usize {
                    gray = 0.0;
                }

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
        let player_left = world.upper_left_x
            + (self.player_p.tile_x * world.tile_side_in_pixels) as f32
            + world.meters_to_pixels * self.player_p.tile_rel_x
            - 0.5 * world.meters_to_pixels * player_width;
        let player_top = world.upper_left_y
            + (self.player_p.tile_y * world.tile_side_in_pixels) as f32
            + world.meters_to_pixels * self.player_p.tile_rel_y
            - world.meters_to_pixels * player_height;
        let player_right = player_left + world.meters_to_pixels * player_width;
        let player_bottom = player_top + world.meters_to_pixels * player_height;
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
    pub meters_to_pixels: f32,

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

    fn recanonicalize_coord(
        &self,
        tile_count: i32,
        tile_map: &mut i32,
        tile: &mut i32,
        tile_rel: &mut f32,
    ) {
        let offset = (*tile_rel / self.tile_side_in_meters).floor() as i32;
        *tile += offset;
        *tile_rel -= offset as f32 * self.tile_side_in_meters;

        // TODO: Fix rounding bug
        assert!(*tile_rel >= 0.0 && *tile_rel <= self.tile_side_in_meters);

        while *tile < 0 {
            *tile += tile_count;
            *tile_map -= 1;
        }

        while *tile >= tile_count {
            *tile -= tile_count;
            *tile_map += 1;
        }
    }

    pub fn recanonicalize_position(&self, pos: CanonicalPosition) -> CanonicalPosition {
        let mut result = pos;

        self.recanonicalize_coord(
            self.tile_count_x,
            &mut result.tile_map_x,
            &mut result.tile_x,
            &mut result.tile_rel_x,
        );
        self.recanonicalize_coord(
            self.tile_count_y,
            &mut result.tile_map_y,
            &mut result.tile_y,
            &mut result.tile_rel_y,
        );

        result
    }

    pub fn is_world_point_empty(&self, can_pos: CanonicalPosition) -> bool {
        let tile_map = self
            .get_tile_map(can_pos.tile_map_x, can_pos.tile_map_y)
            .unwrap();
        tile_map.is_tile_map_point_empty(can_pos.tile_x, can_pos.tile_y)
    }
}

#[derive(Copy, Clone, Default)]
struct CanonicalPosition {
    pub tile_map_x: i32,
    pub tile_map_y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub tile_rel_x: f32,
    pub tile_rel_y: f32,
}
