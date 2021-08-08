use crate::vec2::{Vec2,vec2};
use crate::data;
use crate::Framebuffer;

#[repr(C)]
pub struct Foreground {
    pub width: usize,
    pub height: usize,
    pub blocks: [u8; 4096],
}

impl Foreground {
    pub fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn block_at(&self, pos: Vec2<i32>) -> usize {
        pos.x as usize + pos.y as usize * self.width
    }
    pub fn in_bounds(&self, pos: Vec2<i32>) -> bool {
        self.width() > pos.x as usize && self.height() > pos.y as usize
    }
    pub fn mut_ptr_at(&mut self, pos: Vec2<i32>) -> *mut u8 {
        unsafe { self.blocks.as_mut_ptr().add(self.block_at(pos)) }
    }
    pub fn render(&self, offset: Vec2<i32>, fb: &mut Framebuffer) {
        for (pos,i) in fb.pixels() {
            //if pos.y == 0 { continue; }
            let pos = pos + offset;
            if !self.in_bounds(pos >> 4) {
                *i = 0x00000000;
                continue;
            }
            let block = self.blocks[self.block_at(pos >> 4)] as usize;
            let block_top = self.blocks[self.block_at((pos >> 4) + vec2(0,1))] as usize;
            let block_top = if block_top >= 0x30 && block_top < 0x38 {
                (block_top - 0x10) & 0xF3
            } else {
                0
            };
            let inner = pos & 0x0F;
            let gfx = &data::BLOCKS.data()[block*256..];
            let gfx2 = &data::BLOCKS.data()[block_top*256..];
            let px = gfx[inner.x as usize + inner.y as usize * 16] as usize;
            let px2 = gfx2[inner.x as usize + inner.y as usize * 16] as usize;
            if px2 != 0 {
                let color = data::BLOCKS.pal()[px2];
                *i = color;
            } else if px != 0 {
                let color = data::BLOCKS.pal()[px];
                *i = color;
            }
        }
    }
}

pub enum Collision {
    None,
    Solid,
    Semisolid,
    Gem
}

impl Collision {
    pub fn is_semisolid(&self) -> bool {
        matches!(self, Collision::Semisolid)
    }
    pub fn is_solid(&self) -> bool {
        matches!(self, Collision::Solid)
    }
}

pub fn collision(b: u8) -> Collision {
    use Collision::*;
    match b {
        0x01 => Solid,
        0x04 => Solid,
        0x10 => Gem,
        0x30 ..= 0x33 => Solid,
        0x40 ..= 0x43 => Solid,
        0x50 ..= 0x53 => Solid,
        0x60 ..= 0x63 => Solid,
        0x34 ..= 0x37 => Semisolid,
        _ => None
    }
}
