pub struct GameOffScreenBuffer {
    pub memory: *mut u8,
    pub width: i32,
    pub height: i32,
    pub pitch: i32,
}

pub fn game_update_and_render(buffer: &mut GameOffScreenBuffer) {
    render_weird_gradient(buffer, 0, 0);
}

fn render_weird_gradient(buffer: &mut GameOffScreenBuffer, x_offset: i32, y_offset: i32) {
    let mut row = buffer.memory;
    for y in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for x in 0..buffer.width {
            let b = x + x_offset;
            let g = y + y_offset;
            unsafe {
                *pixel = (((g & 0xFF) << 8) | (b & 0xFF)) as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe {
            row = row.offset(buffer.pitch as isize);
        }
    }
}
