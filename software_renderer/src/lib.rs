pub fn render_weird_gradient(memory: *mut u8, width: i32, height: i32, pitch: i32, x_offset: i32, y_offset: i32) {
    let mut row = memory;
    for y in 0..height {
        let mut pixel = row as *mut u32;
        for x in 0..width {
            let b = x + x_offset;
            let g = y + y_offset;
            unsafe {
                *pixel = (((g & 0xFF) << 8) | (b & 0xFF)) as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe {
            row = row.offset(pitch as isize);
        }
    }
}
