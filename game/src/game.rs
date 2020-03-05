use core::ops::{Deref, DerefMut};

use software_renderer::*;

use crate::random::RANDOM_NUMBER_TABLE;
use crate::GameInput;
use crate::GameOffscreenBuffer;
use crate::GameSoundBuffer;

use crate::tile_map::*;
use core::ptr::null_mut;
use debug_platform_read_entire_file;

struct World {
    tile_map: ArenaObject<TileMap>,
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
    pub fn empty() -> ArenaArray<T> {
        ArenaArray {
            ptr: null_mut(),
            len: 0,
        }
    }

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

pub struct GameState {
    world_arena: MemoryArena,
    world: ArenaObject<World>,
    player_p: TileMapPosition,
    backdrop: LoadedBitmap,
    hero_head: LoadedBitmap,
    hero_cape: LoadedBitmap,
    hero_torso: LoadedBitmap,
}

impl GameState {
    pub unsafe fn new(permanent_storage: &mut MemoryArena) -> GameState {
        let backdrop = debug_load_bmp("test/test_background.bmp\0".as_ptr() as *const i8).unwrap();
        let hero_head =
            debug_load_bmp("test/test_hero_front_head.bmp\0".as_ptr() as *const i8).unwrap();
        let hero_cape =
            debug_load_bmp("test/test_hero_front_cape.bmp\0".as_ptr() as *const i8).unwrap();
        let hero_torso =
            debug_load_bmp("test/test_hero_front_torso.bmp\0".as_ptr() as *const i8).unwrap();

        let mut world_arena = permanent_storage.reserve(permanent_storage.remaining());
        let mut tile_map = world_arena.alloc_uninit::<TileMap>();
        tile_map.tile_side_in_meters = 1.4;
        tile_map.chunk_shift = 4;
        tile_map.chunk_mask = (1 << tile_map.chunk_shift) - 1;
        tile_map.chunk_dim = 1 << tile_map.chunk_shift;
        tile_map.tile_chunk_count_x = 128;
        tile_map.tile_chunk_count_y = 128;
        tile_map.tile_chunk_count_z = 2;
        tile_map.tile_chunks = world_arena.alloc_array_uninit::<TileChunk>(
            (tile_map.tile_chunk_count_x
                * tile_map.tile_chunk_count_y
                * tile_map.tile_chunk_count_z) as usize,
        );

        for tile_chunk_z in 0..tile_map.tile_chunk_count_z {
            for tile_chunk_y in 0..tile_map.tile_chunk_count_y {
                for tile_chunk_x in 0..tile_map.tile_chunk_count_x {
                    let chunk_dim = tile_map.chunk_dim;
                    let tile_chunk = tile_map
                        .get_tile_chunk_mut(tile_chunk_x, tile_chunk_y, tile_chunk_z)
                        .unwrap();
                    tile_chunk.chunk_dim = chunk_dim;
                    tile_chunk.tiles = ArenaArray::empty();
                }
            }
        }

        // for tile_chunk in tile_map.tile_chunks.iter() {
        //     assert_eq!(
        //         tile_chunk.tiles.len(),
        //         (tile_map.chunk_dim * tile_map.chunk_dim) as usize
        //     );
        // }

        let tiles_per_width = 17;
        let tiles_per_height = 9;
        let mut screen_x = 0;
        let mut screen_y = 0;
        let mut random_number_index = 0;
        let mut door_left = false;
        let mut door_right = false;
        let mut door_top = false;
        let mut door_bottom = false;
        let mut door_up = false;
        let mut door_down = false;
        let mut abs_tile_z = 0;
        for _ in 0..100 {
            assert!(random_number_index < RANDOM_NUMBER_TABLE.len());
            let random_choice = if door_up || door_down {
                RANDOM_NUMBER_TABLE[random_number_index] % 2
            } else {
                RANDOM_NUMBER_TABLE[random_number_index] % 3
            };
            random_number_index += 1;

            let mut created_z_door = false;
            match random_choice {
                2 => {
                    created_z_door = true;
                    if abs_tile_z == 0 {
                        door_up = true;
                    } else {
                        door_down = true;
                    }
                }
                1 => {
                    door_right = true;
                }
                _ => {
                    door_top = true;
                }
            }

            for tile_y in 0..tiles_per_height {
                for tile_x in 0..tiles_per_width {
                    let abs_tile_x = screen_x * tiles_per_width + tile_x;
                    let abs_tile_y = screen_y * tiles_per_height + tile_y;

                    let mut tile_value = 1;
                    if tile_x == 0 && (tile_y != tiles_per_height / 2 || !door_left) {
                        tile_value = 2;
                    }

                    if tile_x == (tiles_per_width - 1)
                        && (tile_y != tiles_per_height / 2 || !door_right)
                    {
                        tile_value = 2;
                    }

                    if tile_y == 0 && (tile_x != tiles_per_width / 2 || !door_bottom) {
                        tile_value = 2;
                    }

                    if tile_y == (tiles_per_height - 1)
                        && (tile_x != tiles_per_width / 2 || !door_top)
                    {
                        tile_value = 2;
                    }

                    if tile_x == 10 && tile_y == 6 {
                        if door_up {
                            tile_value = 3;
                        } else if door_down {
                            tile_value = 4;
                        }
                    }

                    tile_map.set_tile_value(
                        &mut world_arena,
                        abs_tile_x,
                        abs_tile_y,
                        abs_tile_z,
                        tile_value,
                    );
                }
            }

            door_left = door_right;
            door_bottom = door_top;
            if created_z_door {
                door_up = !door_up;
                door_down = !door_down;
            } else {
                door_up = false;
                door_down = false;
            }

            door_right = false;
            door_top = false;

            match random_choice {
                2 => {
                    if abs_tile_z == 0 {
                        abs_tile_z = 1;
                    } else {
                        abs_tile_z = 0;
                    }
                }
                1 => {
                    screen_x += 1;
                }
                _ => {
                    screen_y += 1;
                }
            }
        }

        let world = world_arena.alloc(World { tile_map });
        let mut player_p = TileMapPosition::default();
        player_p.abs_tile_x = 1;
        player_p.abs_tile_y = 3;
        player_p.abs_tile_z = 0;
        player_p.offset_x = 5.0;
        player_p.offset_y = 5.0;
        GameState {
            world_arena,
            world,
            player_p,
            backdrop,
            hero_head,
            hero_cape,
            hero_torso,
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

        let tile_side_in_pixels = 60.0;
        let meters_to_pixels = tile_side_in_pixels / tile_map.tile_side_in_meters;

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
                new_player_p.offset_x += d_player_x * input.dt;
                new_player_p.offset_y += d_player_y * input.dt;
                new_player_p = tile_map.recanonicalize_position(new_player_p);

                let mut player_left = new_player_p;
                player_left.offset_x -= 0.5 * player_width;
                player_left = tile_map.recanonicalize_position(player_left);

                let mut player_right = new_player_p;
                player_right.offset_x += 0.5 * player_width;
                player_right = tile_map.recanonicalize_position(player_right);

                if tile_map.is_point_empty(player_left)
                    && tile_map.is_point_empty(player_right)
                    && tile_map.is_point_empty(new_player_p)
                {
                    if !self.player_p.is_on_same_tile(&new_player_p) {
                        match tile_map.get_tile_value(
                            new_player_p.abs_tile_x,
                            new_player_p.abs_tile_y,
                            new_player_p.abs_tile_z,
                        ) {
                            Some(3) => {
                                new_player_p.abs_tile_z += 1;
                            }
                            Some(4) => {
                                new_player_p.abs_tile_z -= 1;
                            }
                            _ => {}
                        }
                    }

                    self.player_p = new_player_p;
                }
            }
        }

        let mut render_buffer: RenderBuffer = offscreen_buffer.into();
        let screen_width = render_buffer.width;
        let screen_height = render_buffer.height;

        draw_bitmap(&mut render_buffer, &self.backdrop, 0.0, 0.0);
        // draw_rectangle(
        //     &mut render_buffer,
        //     0.0,
        //     0.0,
        //     screen_width as f32,
        //     screen_height as f32,
        //     1.0,
        //     0.0,
        //     0.0,
        // );

        for rel_y in -10..=10 {
            for rel_x in -20..=20 {
                let x = (self.player_p.abs_tile_x as i32 + rel_x) as u32;
                let y = (self.player_p.abs_tile_y as i32 + rel_y) as u32;

                if let Some(tile_value) = tile_map.get_tile_value(x, y, self.player_p.abs_tile_z) {
                    if tile_value < 2 {
                        continue;
                    }
                    let mut gray = match tile_value {
                        2 => 1.0,
                        3 => 0.25,
                        4 => 0.1,
                        _ => 0.5,
                    };

                    if x == self.player_p.abs_tile_x && y == self.player_p.abs_tile_y {
                        gray = 0.0;
                    }

                    let cen_x = screen_center_x + rel_x as f32 * tile_side_in_pixels as f32
                        - meters_to_pixels * self.player_p.offset_x;
                    let cen_y = screen_center_y - rel_y as f32 * tile_side_in_pixels as f32
                        + meters_to_pixels * self.player_p.offset_y;
                    let min_x = cen_x - 0.5 * tile_side_in_pixels as f32;
                    let min_y = cen_y - 0.5 * tile_side_in_pixels as f32;
                    let max_x = min_x + tile_side_in_pixels as f32;
                    let max_y = min_y + tile_side_in_pixels as f32;
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
        }

        let player_r = 1.0;
        let player_g = 1.0;
        let player_b = 0.0;
        let player_left = screen_center_x - 0.5 * meters_to_pixels * player_width;
        let player_top = screen_center_y - meters_to_pixels * player_height;
        let player_right = player_left + meters_to_pixels * player_width;
        let player_bottom = player_top + meters_to_pixels * player_height;
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
        draw_bitmap(&mut render_buffer, &self.hero_head, player_left, player_top);
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

#[repr(C, packed(1))]
struct BitmapHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    bitmap_offset: u32,
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    size_of_bitmap: u32,
    horz_resolution: i32,
    vert_resolution: i32,
    colors_used: u32,
    colors_important: u32,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

pub struct LoadedBitmap {
    pub pixels: *mut u32,
    pub width: usize,
    pub height: usize,
}

impl LoadedBitmap {
    pub fn rows(&self) -> impl Iterator<Item = &[u32]> {
        self.pixels().chunks_exact(self.width).rev()
    }

    pub fn pixels(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.pixels, self.width * self.height) }
    }

    pub fn pixels_mut(&mut self) -> &mut [u32] {
        unsafe { core::slice::from_raw_parts_mut(self.pixels, self.width * self.height) }
    }
}

unsafe fn debug_load_bmp(file_name: *const i8) -> Option<LoadedBitmap> {
    let result = debug_platform_read_entire_file(file_name);
    if result.content_size > 0 {
        let base = result.contents as *mut u8;
        let header = &*(base as *mut BitmapHeader);
        let mut bitmap = LoadedBitmap {
            pixels: base.offset(header.bitmap_offset as isize) as *mut u32,
            width: header.width as usize,
            height: header.height as usize,
        };

        let width = bitmap.width;
        for row in bitmap.pixels_mut().chunks_exact_mut(width as usize) {
            for pixel in row {
                *pixel = ((*pixel) >> 8) | ((*pixel) << 24);
            }
        }

        return Some(bitmap);
    }

    None
}

pub fn draw_bitmap(buffer: &mut RenderBuffer, bitmap: &LoadedBitmap, x: f32, y: f32) {
    assert_eq!(buffer.bytes_per_pixel, 4);

    let mut width = bitmap.width as isize;
    let mut height = bitmap.height as isize;
    let mut src_min_x = 0;
    let mut src_min_y = 0;
    let mut dst_min_x = x.round() as isize;
    let mut dst_min_y = y.round() as isize;
    if dst_min_x < 0 {
        width += dst_min_x;
        src_min_x -= dst_min_x;
        dst_min_x = 0;
    }
    if dst_min_y < 0 {
        height += dst_min_y;
        src_min_y -= dst_min_y;
        dst_min_y = 0;
    }

    let mut dst_max_x = dst_min_x + width;
    let mut dst_max_y = dst_min_y + height;
    if dst_max_x > buffer.width as isize {
        width -= dst_max_x - buffer.width as isize;
        dst_max_x = dst_min_x + width;
    }
    if dst_max_y > buffer.height as isize {
        height -= dst_max_y - buffer.height as isize;
        dst_max_y = dst_min_y + height;
    }

    if dst_min_x >= dst_max_x || dst_min_y >= dst_max_y {
        return;
    }

    for (dst_row, src_row) in buffer
        .bytes
        .chunks_exact_mut(buffer.pitch)
        .skip(dst_min_y as usize)
        .take(height as usize)
        .zip(bitmap.rows().skip(src_min_y as usize).take(height as usize))
    {
        for (dst, src) in dst_row
            .chunks_exact_mut(buffer.bytes_per_pixel)
            .skip(dst_min_x as usize)
            .take(width as usize)
            .zip(src_row.iter().skip(src_min_x as usize).take(width as usize))
        {
            unsafe {
                *(dst.as_mut_ptr() as *mut u32) = *src;
            }
        }
    }
}
