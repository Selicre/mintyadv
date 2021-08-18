use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::data;
use crate::vec2::{Vec2, vec2};


pub struct TitleState {
    pub init_flag: bool,
    pub selected: usize
}

impl TitleState {
    pub fn init(&mut self) {
        self.selected = 0;
    }
    pub fn run(&mut self, fb: &mut Framebuffer, b: Buttons) {
        if self.init_flag {
            self.init_flag = false;
            self.init();
        }

        if b.up_edge() {
            self.selected -= 1;
        } else if b.down_edge() {
            self.selected += 1;
        }
        self.selected = self.selected.rem_euclid(data::LEVEL_COUNT);

        for (i,px) in fb.pixels() {
            *px = 0xFF222222;
        }
        let text = include_bytes!(concat!(env!("OUT_DIR"), "/title.txt"));
        let x = (Framebuffer::size().x - 8 * text.len() as i32)/2;
        crate::utils::draw_text(data::BOLDFACE, text, vec2(x,8), fb);

        crate::utils::draw_text(data::BOLDFACE, b"1-1-1 FRENCH FRY FIELDS", vec2(32, 32), fb);
        crate::utils::draw_text(data::BOLDFACE, b"1-1-2", vec2(32, 40), fb);
        crate::utils::draw_text(data::BOLDFACE, b"1-1-3", vec2(32, 48), fb);
        crate::utils::draw_text(data::BOLDFACE, b"1-1-4", vec2(32, 56), fb);

        crate::utils::draw_text(data::BOLDFACE, b"-", vec2(16, 32 + self.selected as i32 * 8), fb);


        if b.start_edge() || b.right_edge() {
            // NOTE: self destroyed
            unsafe {
                use super::GameStateId;
                let v = self.selected;
                let st = crate::state();
                st.id = GameStateId::Level;
                st.as_level().init_flag = true;
                st.as_level().reset_flag = true;
                st.as_level().room = v;
                return;
            }
        }
    }
}
