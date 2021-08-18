use crate::foreground::Foreground;
use crate::background::Background;
use crate::vec2::{vec2, Vec2};
use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;
use crate::data::{self, DataDef};
use crate::entity::{EntityKind, EntityEntry, EntitySet};
//use crate::particle::ParticleSet;
use crate::state::GameStateId;

pub struct LevelState {
    pub fg: Foreground,
    pub bg: Background,
    pub entities: EntitySet,
    //pub particles: ParticleSet,
    pub camera: Vec2<i32>,
    pub init_flag: bool,
    pub reset_flag: bool,
    pub room: usize,
    pub coins: i32,
    pub health: i32,
}



impl LevelState {
    pub fn init(&mut self) {
        if self.room >= data::LEVEL_COUNT {
            return;
        }
        let map = &data::MAPS[self.room];
        unsafe {
            self.fg.init(map.width as _, map.height as _);
            crate::copy_fwd(map.data().as_ptr(), self.fg.blocks.as_mut_ptr(), self.fg.blocks.len());
        }
        let (l,r) = (data::ENTITY_OFFSET[self.room], data::ENTITY_OFFSET[self.room+1]);
        self.entities.init_with(&data::ENTITY_LIST[l..r]);
        //self.particles.init();
        let e = &mut self.entities.inner;
        e[31].init(1);
        e[31].data.pos = map.start_pos;
        //e[2].init(2);
        //e[2].data.pos = vec2(0x4000, 0x4000);

        if self.reset_flag {
            self.coins = 0;
            self.health = 3;
            self.reset_flag = false;
        }
    }
    pub fn run(&mut self, fb: &mut Framebuffer, b: Buttons) {
        if self.room >= data::LEVEL_COUNT {
            let s = crate::state();
            s.id = GameStateId::Title;
            s.as_title().init_flag = true;
            return;
        }
        if self.init_flag {
            self.init_flag = false;
            self.init();
        }

        //if b.left() { self.camera.x -= 4; }
        //if b.right() { self.camera.x += 4; }
        //if b.up() { self.camera.y -= 4; }
        //if b.down() { self.camera.y += 4; }
        //self.particles.process();
        self.entities.process();

        let followed_slot = 31;

        let e = &self.entities.inner[followed_slot];
        let pivot = e.data.visual_pos() - Framebuffer::size()/2;


        self.camera += (pivot - self.camera).zip(
            e.data.vel,
            |c,v| c.signum() * (c.abs() - 4).max(0)
        );
        self.camera.x = self.camera.x.max(0).min(self.fg.width()  as i32 * 16 - Framebuffer::size().x);
        self.camera.y = self.camera.y.max(6).min(self.fg.height() as i32 * 16 - Framebuffer::size().y - 6);


        //self.particles.render(self.camera, fb);

        self.bg.render(self.camera, fb);
        self.fg.render(self.camera, fb);

        self.entities.render(self.camera, fb);

        static mut COINS_TEXT: [u8; 7] = *b"GEMS 00";
        let s = unsafe { &mut COINS_TEXT };
        s[5] = (self.coins / 10) as u8 + b'0';
        s[6] = (self.coins % 10) as u8 + b'0';
        crate::utils::draw_text(data::BOLDFACE, s, vec2(8,8), fb);

        let h = b"HEALTH @@@@@";
        crate::utils::draw_text(data::BOLDFACE, &h[..self.health as usize + 7], vec2(8,16), fb);
    }
}
