use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

#[derive(Default)]
pub struct GameState {
    player_p: WorldPosition,
}

impl GameState {
    pub fn init(&mut self) {
        self.player_p.abs_tile_x = 3;
        self.player_p.abs_tile_y = 3;
        self.player_p.tile_rel_x = 0.0;
        self.player_p.tile_rel_y = 0.0;
    }

    pub fn update_and_render(
        &mut self,
        input: &GameInput,
        offscreen_buffer: &mut GameOffscreenBuffer,
    ) {
        #[rustfmt::skip]
        let temp_tiles = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];

        let mut tiles = vec![0; 256 * 256];
        for (y, row) in temp_tiles.chunks_exact(33).enumerate() {
            for (x, tile_value) in row.iter().enumerate() {
                tiles[y * 256 + x] = *tile_value;
            }
        }

        let tile_side_in_meters = 1.4;
        let tile_side_in_pixels = 60;

        let center_x = offscreen_buffer.width as f32 / 2.0;
        let center_y = offscreen_buffer.height as f32 / 2.0;

        let tile_chunk = TileChunk { tiles };
        let chunk_shift = 8;
        let world = World {
            tile_side_in_meters,
            tile_side_in_pixels,
            meters_to_pixels: tile_side_in_pixels as f32 / tile_side_in_meters,

            chunk_shift,
            chunk_mask: (1 << chunk_shift) - 1,
            chunk_dim: 1 << chunk_shift,

            tile_chunk_count_x: 1,
            tile_chunk_count_y: 1,
            tile_chunks: vec![tile_chunk],
        };
        for tile_map in world.tile_chunks.iter() {
            assert_eq!(
                tile_map.tiles.len(),
                (world.chunk_dim * world.chunk_dim) as usize
            );
        }

        let player_height = 1.4;
        let player_width = 0.75 * player_height;

        for controller in input.controllers.iter() {
            if controller.is_analog == 0 {
                let mut d_player_x = 0.0;
                let mut d_player_y = 0.0;
                if controller.move_up.ended_down != 0 {
                    d_player_y = 1.0;
                }
                if controller.move_down.ended_down != 0 {
                    d_player_y = -1.0;
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

        for rel_y in -10..=10 {
            for rel_x in -20..=20 {
                let x = (self.player_p.abs_tile_x as i32 + rel_x) as u32;
                let y = (self.player_p.abs_tile_y as i32 + rel_y) as u32;

                let tile_value = world.get_tile_value(x, y).unwrap_or(0);

                let mut gray = 0.5;
                if tile_value == 1 {
                    gray = 1.0;
                }

                if x == self.player_p.abs_tile_x && y == self.player_p.abs_tile_y {
                    gray = 0.0;
                }

                let min_x = center_x + rel_x as f32 * world.tile_side_in_pixels as f32;
                let min_y = center_y - rel_y as f32 * world.tile_side_in_pixels as f32;
                let max_x = min_x + world.tile_side_in_pixels as f32;
                let max_y = min_y - world.tile_side_in_pixels as f32;
                draw_rectangle(
                    &mut render_buffer,
                    min_x,
                    max_y,
                    max_x,
                    min_y,
                    gray,
                    gray,
                    gray,
                );
            }
        }

        let player_r = 1.0;
        let player_g = 1.0;
        let player_b = 0.0;
        let player_left = center_x + world.meters_to_pixels * self.player_p.tile_rel_x
            - 0.5 * world.meters_to_pixels * player_width;
        let player_top = center_y
            - world.meters_to_pixels * self.player_p.tile_rel_y
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
struct TileChunk {
    pub tiles: Vec<i32>,
}

impl TileChunk {}

struct TileChunkRef<'a> {
    pub tiles: &'a Vec<i32>,
    pub chunk_dim: u32,
}

impl<'a> TileChunkRef<'a> {
    pub fn get_tile_value(&self, tile_x: u32, tile_y: u32) -> Option<&i32> {
        self.tiles.get((tile_y * self.chunk_dim + tile_x) as usize)
    }
}

struct World {
    pub tile_side_in_meters: f32,
    pub tile_side_in_pixels: i32,
    pub meters_to_pixels: f32,

    pub chunk_shift: u32,
    pub chunk_mask: u32,
    pub chunk_dim: u32,

    pub tile_chunk_count_x: u32,
    pub tile_chunk_count_y: u32,
    pub tile_chunks: Vec<TileChunk>,
}

impl World {
    fn get_tile_chunk(&self, tile_chunk_x: u32, tile_chunk_y: u32) -> Option<TileChunkRef> {
        self.tile_chunks
            .get((tile_chunk_y * self.tile_chunk_count_x + tile_chunk_x) as usize)
            .map(|tile_chunk| TileChunkRef {
                tiles: &tile_chunk.tiles,
                chunk_dim: self.chunk_dim,
            })
    }

    fn recanonicalize_coord(&self, tile: &mut u32, tile_rel: &mut f32) {
        let offset = (*tile_rel / self.tile_side_in_meters).floor() as i32;
        // allow wrapping
        *tile = (*tile as i32 + offset) as u32;
        *tile_rel -= offset as f32 * self.tile_side_in_meters;

        // TODO: Fix rounding bug
        assert!(*tile_rel >= 0.0 && *tile_rel <= self.tile_side_in_meters);
    }

    pub fn recanonicalize_position(&self, pos: WorldPosition) -> WorldPosition {
        let mut result = pos;

        self.recanonicalize_coord(&mut result.abs_tile_x, &mut result.tile_rel_x);
        self.recanonicalize_coord(&mut result.abs_tile_y, &mut result.tile_rel_y);

        result
    }

    fn get_chunk_position(&self, abs_tile_x: u32, abs_tile_y: u32) -> TileChunkPosition {
        let tile_chunk_x = abs_tile_x >> self.chunk_shift;
        let tile_chunk_y = abs_tile_y >> self.chunk_shift;
        let rel_tile_x = abs_tile_x & self.chunk_mask;
        let rel_tile_y = abs_tile_y & self.chunk_mask;

        TileChunkPosition {
            tile_chunk_x,
            tile_chunk_y,
            rel_tile_x,
            rel_tile_y,
        }
    }

    pub fn get_tile_value(&self, abs_tile_x: u32, abs_tile_y: u32) -> Option<i32> {
        let chunk_pos = self.get_chunk_position(abs_tile_x, abs_tile_y);
        self.get_tile_chunk(chunk_pos.tile_chunk_x, chunk_pos.tile_chunk_y)
            .and_then(|tile_chunk| {
                tile_chunk
                    .get_tile_value(chunk_pos.rel_tile_x, chunk_pos.rel_tile_y)
                    .map(|x| *x)
            })
    }

    pub fn is_world_point_empty(&self, pos: WorldPosition) -> bool {
        self.get_tile_value(pos.abs_tile_x, pos.abs_tile_y) == Some(0)
    }
}

struct TileChunkPosition {
    tile_chunk_x: u32,
    tile_chunk_y: u32,
    rel_tile_x: u32,
    rel_tile_y: u32,
}

#[derive(Copy, Clone, Default)]
struct WorldPosition {
    pub abs_tile_x: u32,
    pub abs_tile_y: u32,
    pub tile_rel_x: f32,
    pub tile_rel_y: f32,
}
