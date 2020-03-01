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

pub fn draw_rectangle(buffer: &mut RenderBuffer, min_x: f32, min_y: f32, max_x: f32, max_y: f32, color: u32) {
    assert_eq!(buffer.bytes_per_pixel, 4);

    let min_x = min_x.round() as usize;
    let min_y = min_y.round() as usize;
    let mut max_x = max_x.round() as usize;
    let mut max_y = max_y.round() as usize;

    if max_x > buffer.width  { max_x = buffer.width; }
    if max_y > buffer.height { max_y = buffer.height; }

    if min_x >= max_x || min_y >= max_y { return; }

    let bytes_per_pixel = buffer.bytes_per_pixel;
    let min_row = min_y * buffer.pitch;
    let max_row = max_y * buffer.pitch;
    let min_col = min_x * bytes_per_pixel;
    let max_col = max_x * bytes_per_pixel;

    buffer.bytes[min_row..max_row].chunks_mut(buffer.pitch).for_each(|row| {
        row[min_col..max_col].chunks_mut(bytes_per_pixel).for_each(|pixel| {
            unsafe { *(pixel.as_mut_ptr() as *mut u32) = color; }
        });
    });
}

