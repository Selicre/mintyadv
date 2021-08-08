use crate::data::DataDef;
use crate::framebuffer::Framebuffer;
use crate::vec2::{Vec2, vec2};

pub fn draw_text(d: DataDef, text: &[u8], pos: Vec2<i32>, fb: &mut Framebuffer) {
    for (e,i) in text.iter().enumerate() {
        let s = (*i as usize - 0x20) * 64;
        draw_symbol(&d.data()[s..], pos+vec2(e as i32 * 8, 0), fb);
    }
}

pub fn draw_symbol(data: &[u8], pos: Vec2<i32>, fb: &mut Framebuffer) {
    for i in 0usize..64 {
        let x = i & 0x7;
        let y = i >> 3;
        if data[i] != 0 {
            fb.pixel(pos + vec2(x as i32 + 1, y as i32 + 1)).map(|c| *c = 0xFF000000);
            fb.pixel(pos + vec2(x as i32, y as i32)).map(|c| *c = 0xFFFFFFFF);
        }
    }
}

