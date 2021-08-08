use crate::vec2::Vec2;
use crate::data;
use crate::Framebuffer;
pub struct Background {
}

impl Background {
    pub fn render(&self, offset: Vec2<i32>, fb: &mut Framebuffer) {
        let mut offset = offset / 4;
        offset.y = 0;
        for (pos,i) in fb.pixels() {
            //if pos.y == 0 { continue; }
            let pos = pos + offset;
            let b = ((pos.x % 480) + pos.y * 480) as usize;
            let px = data::BG.data()[b] as usize;
            let color = data::BG.pal()[px];
            *i = color;
        }
    }
}
