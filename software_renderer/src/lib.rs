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

pub fn draw_rectangle(
    buffer: &mut RenderBuffer,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    r: f32,
    g: f32,
    b: f32,
) {
    assert_eq!(buffer.bytes_per_pixel, 4);

    let min_x = min_x.round().max(0.0) as isize;
    let min_y = min_y.round().max(0.0) as isize;
    let max_x = (max_x.round() as isize).min(buffer.width as isize);
    let max_y = (max_y.round() as isize).min(buffer.height as isize);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let bytes_per_pixel = buffer.bytes_per_pixel;
    let min_row = min_y as usize * buffer.pitch;
    let max_row = max_y as usize * buffer.pitch;
    let min_col = min_x as usize * bytes_per_pixel;
    let max_col = max_x as usize * bytes_per_pixel;

    let r = (r * 255.0).round() as u8;
    let g = (g * 255.0).round() as u8;
    let b = (b * 255.0).round() as u8;

    for row in buffer.bytes[min_row..max_row].chunks_exact_mut(buffer.pitch) {
        let row = unsafe { row.get_unchecked_mut(min_col..max_col) };
        for pixel in row.chunks_exact_mut(bytes_per_pixel) {
            // PATTERN: BB GG RR AA
            unsafe {
                *pixel.get_unchecked_mut(0) = b;
                *pixel.get_unchecked_mut(1) = g;
                *pixel.get_unchecked_mut(2) = r;
            }
        }
    }
}
