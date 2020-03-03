use software_renderer::*;

use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

use crate::tile_map::*;
use core::ops::{Deref, DerefMut};

struct World {
    tile_map: ArenaObject<TileMap>,
}

pub struct GameState {
    world_arena: MemoryArena,
    world: ArenaObject<World>,
    player_p: TileMapPosition,
}

pub struct MemoryArena {
    base: *mut u8,
    size: usize,
    used: usize,
}

pub struct ArenaArray<T> {
    ptr: *mut T,
    len: usize,
}

impl<T> ArenaArray<T> {
    pub fn from_raw_parts(ptr: *mut T, len: usize) -> ArenaArray<T> {
        ArenaArray { ptr, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> ArenaArrayIter<T> {
        ArenaArrayIter {
            array: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> ArenaArrayIterMut<T> {
        ArenaArrayIterMut {
            array: self,
            index: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { self.get_unchecked(index) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { self.get_unchecked_mut(index) })
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        &*self.ptr.offset(index as isize)
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut *self.ptr.offset(index as isize)
    }
}

pub struct ArenaArrayIter<'a, T> {
    array: &'a ArenaArray<T>,
    index: usize,
}

impl<'a, T> Iterator for ArenaArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            return None;
        }

        let item = unsafe { self.array.get_unchecked(self.index) };
        self.index += 1;
        Some(item)
    }
}

pub struct ArenaArrayIterMut<'a, T> {
    array: &'a mut ArenaArray<T>,
    index: usize,
}

impl<'a, T> Iterator for ArenaArrayIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            return None;
        }
        let item = unsafe { &mut *self.array.ptr.offset(self.index as isize) };
        self.index += 1;
        Some(item)
    }
}

pub struct ArenaObject<T: ?Sized> {
    ptr: *mut T,
}

impl<T> ArenaObject<T> {
    pub fn from_raw(ptr: *mut T) -> ArenaObject<T> {
        ArenaObject { ptr }
    }
}

impl<T> AsRef<T> for ArenaObject<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> AsMut<T> for ArenaObject<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Deref for ArenaObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for ArenaObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl MemoryArena {
    pub fn from_raw_parts(base: *mut u8, size: usize) -> MemoryArena {
        MemoryArena {
            base,
            size,
            used: 0,
        }
    }

    pub fn alloc<T>(&mut self, val: T) -> ArenaObject<T> {
        unsafe {
            let mut result = self.alloc_uninit::<T>();
            *result = val;
            result
        }
    }

    pub unsafe fn alloc_uninit<T>(&mut self) -> ArenaObject<T> {
        let size = core::mem::size_of::<T>();
        let memory = self.alloc_size(size);
        ArenaObject::from_raw(memory as *mut T)
    }

    pub fn alloc_array<T: Clone>(&mut self, val: T, len: usize) -> ArenaArray<T> {
        let mut array = unsafe { self.alloc_array_uninit::<T>(len) };
        for e in array.iter_mut() {
            *e = val.clone();
        }
        array
    }

    pub unsafe fn alloc_array_uninit<T>(&mut self, len: usize) -> ArenaArray<T> {
        let size = core::mem::size_of::<T>() * len;
        let memory = self.alloc_size(size);
        ArenaArray::from_raw_parts(memory as *mut T, len)
    }

    unsafe fn alloc_size(&mut self, size: usize) -> *mut u8 {
        assert!(self.used + size <= self.size);

        let memory = self.base.offset(self.used as isize);
        self.used += size;
        memory
    }

    pub fn reserve(&mut self, size: usize) -> MemoryArena {
        assert!(self.used + size <= self.size);
        let base = unsafe { self.base.offset(self.used as isize) };
        self.used += size;
        MemoryArena::from_raw_parts(base, size)
    }

    pub fn remaining(&self) -> usize {
        self.size - self.used
    }
}

impl GameState {
    pub unsafe fn new(permanent_storage: &mut MemoryArena) -> GameState {
        let mut world_arena = permanent_storage.reserve(permanent_storage.remaining());
        let mut tile_map = world_arena.alloc_uninit::<TileMap>();
        tile_map.tile_side_in_meters = 1.4;
        tile_map.tile_side_in_pixels = 60;
        tile_map.meters_to_pixels =
            tile_map.tile_side_in_pixels as f32 / tile_map.tile_side_in_meters;
        tile_map.chunk_shift = 4;
        tile_map.chunk_mask = (1 << tile_map.chunk_shift) - 1;
        tile_map.chunk_dim = 1 << tile_map.chunk_shift;
        tile_map.tile_chunk_count_x = 128;
        tile_map.tile_chunk_count_y = 128;
        tile_map.tile_chunks = world_arena.alloc_array_uninit::<TileChunk>(
            (tile_map.tile_chunk_count_x * tile_map.tile_chunk_count_y) as usize,
        );

        for tile_chunk_y in 0..tile_map.tile_chunk_count_y {
            for tile_chunk_x in 0..tile_map.tile_chunk_count_x {
                let chunk_dim = tile_map.chunk_dim;
                let tile_chunk_count_x = tile_map.tile_chunk_count_x;
                let tile_chunk = tile_map
                    .tile_chunks
                    .get_mut((tile_chunk_y * tile_chunk_count_x + tile_chunk_x) as usize)
                    .unwrap();
                tile_chunk.chunk_dim = chunk_dim;
                tile_chunk.tiles = world_arena.alloc_array(0, (chunk_dim * chunk_dim) as usize);
            }
        }

        for tile_chunk in tile_map.tile_chunks.iter() {
            assert_eq!(
                tile_chunk.tiles.len(),
                (tile_map.chunk_dim * tile_map.chunk_dim) as usize
            );
        }

        let tiles_per_width = 17;
        let tiles_per_height = 9;

        for screen_y in 0..32 {
            for screen_x in 0..32 {
                for tile_y in 0..tiles_per_height {
                    for tile_x in 0..tiles_per_width {
                        let abs_tile_x = screen_x * tiles_per_width + tile_x;
                        let abs_tile_y = screen_y * tiles_per_height + tile_y;

                        tile_map.set_tile_value(
                            &mut world_arena,
                            abs_tile_x,
                            abs_tile_y,
                            if tile_x == tile_y && tile_y % 2 == 0 {
                                1
                            } else {
                                0
                            },
                        );
                    }
                }
            }
        }

        let world = world_arena.alloc(World { tile_map });
        let mut player_p = TileMapPosition::default();
        player_p.abs_tile_x = 1;
        player_p.abs_tile_y = 3;
        player_p.tile_rel_x = 5.0;
        player_p.tile_rel_y = 5.0;
        GameState {
            world_arena,
            world,
            player_p,
        }
    }

    pub fn update_and_render(
        &mut self,
        input: &GameInput,
        offscreen_buffer: &mut GameOffscreenBuffer,
    ) {
        let screen_center_x = offscreen_buffer.width as f32 / 2.0;
        let screen_center_y = offscreen_buffer.height as f32 / 2.0;

        let ref tile_map = self.world.tile_map;

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
                let mut player_speed = 2.0;
                if controller.action_up.ended_down != 0 {
                    player_speed = 10.0;
                }
                d_player_x *= player_speed;
                d_player_y *= player_speed;
                let mut new_player_p = self.player_p;
                new_player_p.tile_rel_x += d_player_x * input.dt;
                new_player_p.tile_rel_y += d_player_y * input.dt;
                new_player_p = tile_map.recanonicalize_position(new_player_p);

                let mut player_left = new_player_p;
                player_left.tile_rel_x -= 0.5 * player_width;
                player_left = tile_map.recanonicalize_position(player_left);

                let mut player_right = new_player_p;
                player_right.tile_rel_x += 0.5 * player_width;
                player_right = tile_map.recanonicalize_position(player_right);

                if tile_map.is_point_empty(player_left)
                    && tile_map.is_point_empty(player_right)
                    && tile_map.is_point_empty(new_player_p)
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

                let tile_value = tile_map.get_tile_value(x, y).unwrap_or(0);

                let mut gray = 0.5;
                if tile_value == 1 {
                    gray = 1.0;
                }

                if x == self.player_p.abs_tile_x && y == self.player_p.abs_tile_y {
                    gray = 0.0;
                }

                let cen_x = screen_center_x + rel_x as f32 * tile_map.tile_side_in_pixels as f32
                    - tile_map.meters_to_pixels * self.player_p.tile_rel_x;
                let cen_y = screen_center_y - rel_y as f32 * tile_map.tile_side_in_pixels as f32
                    + tile_map.meters_to_pixels * self.player_p.tile_rel_y;
                let min_x = cen_x - 0.5 * tile_map.tile_side_in_pixels as f32;
                let min_y = cen_y - 0.5 * tile_map.tile_side_in_pixels as f32;
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
        let player_left = screen_center_x - 0.5 * tile_map.meters_to_pixels * player_width;
        let player_top = screen_center_y - tile_map.meters_to_pixels * player_height;
        let player_right = player_left + tile_map.meters_to_pixels * player_width;
        let player_bottom = player_top + tile_map.meters_to_pixels * player_height;
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
                core::slice::from_raw_parts_mut(
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
