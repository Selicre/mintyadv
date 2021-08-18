use crate::vec2::{Vec2, vec2};
use crate::Framebuffer;
use crate::data::DataDef;
use crate::utils::ArrayVec;

mod player;

pub struct Entity {
    pub data: EntityData,
    pub kind: EntityKind
}

pub struct EntitySet {
    pub inner: [Entity; 32],
}

pub struct EntityEntry {
    pub x: u8,
    pub y: u8,
    pub kind: u8
}

pub struct Sprite {
    active: bool,
    source: DataDef,
    offset: Vec2<i32>,
    frame: i32
}

pub struct EntityData {
    pub state: EntityState,
    pub pos: Vec2<i32>,
    pub vel: Vec2<i32>,
    pub on_ground: bool,
    pub sensors: [*mut u8; 6],
    pub sensor_pos: [Vec2<i32>; 6],
    pub flip: Vec2<bool>,
    pub next_pos: Vec2<i32>,
    pub radius: Vec2<i32>,
    pub sprite: Sprite,
    pub sprites: [Sprite; 4],
    pub anim_timer: i32,
}

#[repr(u8)]
pub enum EntityKind {
    None,
    Player(player::Player),
    Tomato,
    Bee,
    Snail,
    Platform
}

pub enum EntityState {
    Alive,
    Squished,
    Stunned,
    Kicked,
    Dead
}

impl Entity {
    pub fn init(&mut self, id: u8) {
        self.kind.init_kind(id);
        self.kind.init(&mut self.data);
    }
}

impl EntityKind {
    pub fn init_kind(&mut self, id: u8) {
        unsafe { (self as *mut _ as *mut u8).write(id); }
    }
    pub fn init(&mut self, data: &mut EntityData) {
        match self {
            EntityKind::None => {},
            EntityKind::Player(p) => {
                p.init();
                data.vel = vec2(0,0);
                data.radius = vec2(0x400, 0xE00);
                data.sprites[0].active = true;
                data.sprites[0].source = crate::data::TOOTHPASTE;
                data.sprites[0].offset = vec2(-16, -16);
                data.sprites[0].frame = 0x10;
                data.sprites[1].active = true;
                data.sprites[1].source = crate::data::TOOTHPASTE;
                data.sprites[1].offset = vec2(-16, -16);
                data.sprites[1].frame = 0;
            },
            EntityKind::Tomato => {
                data.radius = vec2(0x400, 0x400);
                data.flip.x = true;
                data.sprites[0].active = true;
                data.sprites[0].source = crate::data::ENTITIES;
                data.sprites[0].offset = vec2(-16, -26);
                data.sprites[0].frame = 0;
            },
            _ => {}
        }
    }
    pub fn process(&mut self, data: &mut EntityData) {
        let camera = crate::state().as_level().camera;
        let x_delta = data.pos.x - camera.x * 0x100;
        if x_delta < -0x2000 || x_delta > 0x16000 { return; }
        let fg = &crate::state().as_level().fg;
        let level_h = fg.height() as i32 * 16 * 256;
        if data.pos.y > level_h { *self = EntityKind::None; return; }
        match data.state {
            EntityState::Squished => {
                data.anim_timer -= 1;
                if data.anim_timer == 0 { *self = EntityKind::None; data.pos = vec2(0,0); }
            }
            EntityState::Dead => {
                data.pos += data.vel;
                data.vel.y += 0x30;
                data.flip.y = true;
            }
            EntityState::Alive => match self {
                EntityKind::None => {},
                EntityKind::Player(p) => p.run(data),
                EntityKind::Tomato => {
                    if data.flip.x {
                        data.vel.x = -0x60;
                    } else {
                        data.vel.x = 0x60;
                    }
                    if data.on_ground {
                        data.vel.y = 0;
                    } else {
                        data.vel.y += 0x30;
                    }
                    data.anim_timer += 0x1;
                    data.sprites[0].frame = match data.anim_timer / 10 & 3 {
                        0 => 0,
                        1 => 1,
                        2 => 2,
                        3 => 1,
                        _ => panic!()
                    };
                    data.physics();
                    if data.vel.x == 0 { data.flip.x = !data.flip.x; }
                }
                _ => {}
            }
            _ => {}
        }
    }
}


impl EntitySet {
    pub fn init(&mut self) {
        for i in self.inner.iter_mut() {
            i.kind = EntityKind::None;
            i.data.init();
        }
    }
    pub fn init_with(&mut self, list: &[EntityEntry]) {
        for idx in 0..32 {
            self.inner[idx].data.init();
            let e = &mut self.inner[idx];
            if let Some(c) = list.get(idx) {
                e.init(c.kind);
                e.data.pos = vec2(c.x as i32, c.y as i32) * 0x1000 + vec2(0x800, 0xF00-e.data.radius.y);
            } else {
                e.kind = EntityKind::None;
            }
        }
    }
    pub fn process(&mut self) {
        for i in self.inner.iter_mut() {
            i.kind.process(&mut i.data);
        }
    }
    pub fn render(&self, camera: Vec2<i32>, fb: &mut Framebuffer) {
        for i in self.inner.iter() {
            if !matches!(i.kind, EntityKind::None) {
                i.data.render(camera, fb);
            }
        }
    }
}

impl EntityData {
    pub fn init(&mut self) {
        *self = unsafe { core::mem::zeroed() };
    }
    pub fn visual_pos(&self) -> Vec2<i32> {
        self.pos >> 8
    }
    pub fn intersects(&self, other: &EntityData) -> bool {
        let dist = (self.pos - other.pos).map(|c| c.abs());
        let rad = self.radius + other.radius;
        dist.x < rad.x && dist.y < rad.y
    }
    pub fn render(&self, camera: Vec2<i32>, fb: &mut Framebuffer) {
        for spr in self.sprites.iter().filter(|c| c.active) {
            let data = spr.source.data();
            let pal = spr.source.pal();
            for mut x in 0..32 {
                for mut y in 0..32 {
                    let mut pos = self.visual_pos() + vec2(x as i32, y as i32) - camera + spr.offset;
                    if self.flip.x {
                        //pos.x -= i.offset.x * 2;
                        x = 31 - x;
                    }
                    if self.flip.y {
                        //pos.x -= i.offset.x * 2;
                        y = 31 - y;
                    }
                    let px = data[x+y*32 + spr.frame as usize *32*32];
                    if px != 0 {
                        let px = pal[px as usize];
                        fb.pixel(pos).map(|c| *c = px);
                    }
                }
            }
            /*
            for x in -self.radius.x/256..self.radius.x/256+1 {
                for y in -self.radius.y/256..self.radius.y/256+1 {
                    let mut pos = self.visual_pos() + vec2(x as i32, y as i32) - camera;
                    fb.pixel(pos).map(|c| *c = *c & 0x7FFFFFFF);
                }
            }
            let pos = self.visual_pos() - camera;
            fb.pixel(pos).map(|c| *c = 0xFFFF2020);
            for i in self.sensor_pos.iter() {
                let pos = *i / 256 - camera;
                fb.pixel(pos).map(|c| *c = 0xFF0000FF);
            }*/
        }
    }
    pub fn physics(&mut self) {
        let fg = &mut crate::state().as_level().fg;
        let mut next_pos = self.pos;

        let last_on_ground = self.on_ground;
        self.on_ground = false;
        for axis in 0..2 {
            next_pos[axis] += self.vel[axis];
            let direction = self.vel[axis] >= 0;
            let mut sensor_pos = if direction {
                self.radius[axis]
            } else {
                -self.radius[axis]
            };
            if axis == 1 && last_on_ground { sensor_pos += 256; }
            for i in -1..=1 {
                let p = axis * 3 + (i+1) as usize;
                let mut offset = self.radius * i;
                offset[axis] = sensor_pos;
                self.sensor_pos[p] = ((next_pos + offset) & !0xFFF) + vec2(0x800, 0x800);
                let sensor = (next_pos + offset) / 16 / 256;
                if !fg.in_bounds(sensor) { continue; }
                let block = unsafe { &mut *fg.mut_ptr_at(sensor) };
                self.sensors[p] = block;
                let coll = crate::foreground::collision(*block);
                if coll.is_semisolid() {
                    if direction && axis == 1 && (next_pos.y + sensor_pos) & 0xFFF < 0x400 {
                        self.on_ground = true;
                        next_pos.y = ((next_pos.y + sensor_pos) & !0xFFF) - self.radius.y - 0x100;
                    }
                } else if coll.is_solid() {
                    if direction && axis == 1 {
                        self.on_ground = true;
                        next_pos.y = ((next_pos.y + sensor_pos) & !0xFFF) - self.radius.y - 0x100;
                    } else {
                        next_pos[axis] = self.pos[axis]; self.vel[axis] = 0;
                    }
                }
            }
        }
        self.pos = next_pos;
    }
}
