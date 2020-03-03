use core::num::Wrapping;
use game::{ArenaArray, MemoryArena};

#[derive(Copy, Clone, Default)]
pub struct TileMapPosition {
    pub abs_tile_x: u32,
    pub abs_tile_y: u32,
    pub tile_rel_x: f32,
    pub tile_rel_y: f32,
}

pub struct TileMap {
    pub tile_side_in_meters: f32,
    pub tile_side_in_pixels: i32,
    pub meters_to_pixels: f32,

    pub chunk_shift: u32,
    pub chunk_mask: u32,
    pub chunk_dim: u32,

    pub tile_chunk_count_x: u32,
    pub tile_chunk_count_y: u32,
    pub tile_chunks: ArenaArray<TileChunk>,
}

impl TileMap {
    fn get_tile_chunk(&self, tile_chunk_x: u32, tile_chunk_y: u32) -> Option<&TileChunk> {
        self.tile_chunks.get(
            (Wrapping(tile_chunk_y) * Wrapping(self.tile_chunk_count_x) + Wrapping(tile_chunk_x)).0
                as usize,
        )
    }

    fn get_tile_chunk_mut(
        &mut self,
        tile_chunk_x: u32,
        tile_chunk_y: u32,
    ) -> Option<&mut TileChunk> {
        self.tile_chunks
            .get_mut((tile_chunk_y * self.tile_chunk_count_x + tile_chunk_x) as usize)
    }

    fn recanonicalize_coord(&self, tile: &mut u32, tile_rel: &mut f32) {
        let offset = (*tile_rel / self.tile_side_in_meters).round() as i32;
        // allow wrapping
        *tile = (Wrapping(*tile as i32) + Wrapping(offset)).0 as u32;
        *tile_rel -= offset as f32 * self.tile_side_in_meters;

        // TODO: Fix rounding bug
        assert!(
            *tile_rel >= -0.5 * self.tile_side_in_meters
                && *tile_rel <= 0.5 * self.tile_side_in_meters
        );
    }

    pub fn recanonicalize_position(&self, pos: TileMapPosition) -> TileMapPosition {
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

    pub fn is_point_empty(&self, pos: TileMapPosition) -> bool {
        if let Some(tile_value) = self.get_tile_value(pos.abs_tile_x, pos.abs_tile_y) {
            return tile_value == 0;
        }
        true
    }

    pub fn set_tile_value(
        &mut self,
        arena: &mut MemoryArena,
        abs_tile_x: u32,
        abs_tile_y: u32,
        tile_value: i32,
    ) {
        let chunk_pos = self.get_chunk_position(abs_tile_x, abs_tile_y);
        // TODO: on-demand tile chunk creation
        let tile_chunk = self
            .get_tile_chunk_mut(chunk_pos.tile_chunk_x, chunk_pos.tile_chunk_y)
            .unwrap();
        tile_chunk.set_tile_value(chunk_pos.rel_tile_x, chunk_pos.rel_tile_y, tile_value);
    }
}

pub struct TileChunk {
    pub tiles: ArenaArray<i32>,
    pub chunk_dim: u32,
}

impl TileChunk {
    pub fn get_tile_value(&self, tile_x: u32, tile_y: u32) -> Option<&i32> {
        self.tiles.get((tile_y * self.chunk_dim + tile_x) as usize)
    }

    pub fn set_tile_value(&mut self, tile_x: u32, tile_y: u32, tile_value: i32) {
        if let Some(tile) = self
            .tiles
            .get_mut((tile_y * self.chunk_dim + tile_x) as usize)
        {
            *tile = tile_value;
        }
    }
}

struct TileChunkPosition {
    tile_chunk_x: u32,
    tile_chunk_y: u32,
    rel_tile_x: u32,
    rel_tile_y: u32,
}
