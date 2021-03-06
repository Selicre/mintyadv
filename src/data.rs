use crate::lz4;
use crate::entity::EntityEntry;
use crate::vec2::{vec2,Vec2};

pub struct DataDef {
    pub offset: usize,
    pub pal: usize,
}

impl DataDef {
    pub fn data(&self) -> &'static [u8] {
        unsafe { &DATA[self.offset..] }
    }
    pub fn pal(&self) -> &'static [u32] {
        &PAL_DATA[self.pal..]
    }
}

pub struct LevelDef {
    pub offset: usize,
    pub width: u8,
    pub height: u8,
    pub start_pos: Vec2<i32>
}

impl LevelDef {
    pub fn data(&self) -> &'static [u8] {
        unsafe { &DATA[self.offset..] }
    }
}

pub fn init() {
    unsafe { lz4::decompress(&DATA_LZ4, &mut DATA) };
}

include!(concat!(env!("OUT_DIR"), "/data.rs"));
