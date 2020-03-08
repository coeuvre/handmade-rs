extern crate base;

use base::math::V2;

// pub fn render_weird_gradient(memory: *mut u8, width: i32, height: i32, pitch: i32, x_offset: i32, y_offset: i32) {
//     let mut row = memory;
//     for y in 0..height {
//         let mut pixel = row as *mut u32;
//         for x in 0..width {
//             let b = x + x_offset;
//             let g = y + y_offset;
//             unsafe {
//                 *pixel = (((g & 0xFF) << 8) | (b & 0xFF)) as u32;
//                 pixel = pixel.offset(1);
//             }
//         }
//         unsafe {
//             row = row.offset(pitch as isize);
//         }
//     }
// }

pub struct RenderBuffer<'a> {
    pub bytes: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bytes_per_pixel: usize,
}

pub fn draw_rectangle(buffer: &mut RenderBuffer, min: V2, max: V2, r: f32, g: f32, b: f32) {
    assert_eq!(buffer.bytes_per_pixel, 4);

    let min_x = min.x.round().max(0.0) as isize;
    let min_y = min.y.round().max(0.0) as isize;
    let max_x = (max.x.round() as isize).min(buffer.width as isize);
    let max_y = (max.y.round() as isize).min(buffer.height as isize);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let bytes_per_pixel = buffer.bytes_per_pixel;
    let width = (max_x - min_x) as usize;
    let height = (max_y - min_y) as usize;
    let min_x = min_x as usize;
    let min_y = min_y as usize;

    let r = (r * 255.0).round() as u32;
    let g = (g * 255.0).round() as u32;
    let b = (b * 255.0).round() as u32;
    // PATTERN: BB GG RR AA
    //          0xAARRGGBB
    let color = (r << 16) | (g << 8) | (b << 0);

    for row in buffer
        .bytes
        .chunks_exact_mut(buffer.pitch)
        .skip(min_y)
        .take(height)
    {
        for pixel in row
            .chunks_exact_mut(bytes_per_pixel)
            .skip(min_x)
            .take(width)
        {
            unsafe {
                *(pixel.as_mut_ptr() as *mut u32) = color;
            }
        }
    }
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
                let src_val = *src;
                let dst_val = *(dst.as_ptr() as *const u32);
                let a = ((src_val >> 24) & 0xFF) as f32 / 255.0;
                let sr = ((src_val >> 16) & 0xFF) as f32;
                let sg = ((src_val >> 8) & 0xFF) as f32;
                let sb = ((src_val >> 0) & 0xFF) as f32;
                let dr = ((dst_val >> 16) & 0xFF) as f32;
                let dg = ((dst_val >> 8) & 0xFF) as f32;
                let db = ((dst_val >> 0) & 0xFF) as f32;
                let r = (1.0 - a) * dr + a * sr;
                let g = (1.0 - a) * dg + a * sg;
                let b = (1.0 - a) * db + a * sb;
                *(dst.as_mut_ptr() as *mut u32) = (((r + 0.5) as u32) << 16)
                    | (((g + 0.5) as u32) << 8)
                    | (((b + 0.5) as u32) << 0);
            }
        }
    }
}
