use super::*;

#[derive(PartialEq)]
pub enum PlayerPose {
    Normal,
    Crouching,
    Sliding,
    Hurt,
}

pub struct Player {
    p_meter: i32,
    p_speed: bool,
    slide_timer: i32,
    idle_timer: i32,
    pose: PlayerPose,
    invuln_timer: i32,
}
impl Player {
    pub fn init(&mut self) {
        self.p_meter = 0;
        self.p_speed = false;
        self.pose = PlayerPose::Normal;
        self.invuln_timer = 0;
    }
    pub fn hurt(&mut self, data: &mut EntityData) {
        let l = crate::state().as_level();
        l.health -= 1;
        if l.health == 0 { l.init_flag = true; }
        self.pose = PlayerPose::Hurt;
        data.vel = if data.flip.x {
            vec2(0x100, -0x500)
        } else {
            vec2(-0x100, -0x500)
        };
        data.on_ground = false;
    }
    pub fn invincible(&self) -> bool {
        self.pose == PlayerPose::Sliding
    }
    pub fn can_interact(&self) -> bool {
        self.pose != PlayerPose::Hurt
    }
    pub fn invulnerable(&self) -> bool {
        self.pose == PlayerPose::Hurt || self.invuln_timer > 0
    }
    pub fn run(&mut self, data: &mut EntityData) {
        let buttons = crate::state().buttons;
        if self.slide_timer > 0 { self.slide_timer -= 1; }
        if !data.on_ground {
            if buttons.a() && self.pose != PlayerPose::Hurt {
                data.vel.y += 0x30;
            } else {
                data.vel.y += 0x60;
            }
            data.vel.y = data.vel.y.min(1024);
        } else {
            if self.pose == PlayerPose::Hurt {
                self.invuln_timer = 60;
            }
            if buttons.down() {
                if data.vel.x.abs() > 0 {
                    if self.pose != PlayerPose::Sliding {
                        self.slide_timer = 30;
                    }
                    self.pose = PlayerPose::Sliding;
                } else {
                    self.slide_timer = 0;
                    self.pose = PlayerPose::Crouching;
                }
            } else {
                self.slide_timer = 0;
                self.pose = PlayerPose::Normal;
            }
            if buttons.a_edge() {
                let lift = 0x500 + data.vel.x.abs() * 5 / 16;
                data.vel.y = -lift + 0x30;
                data.on_ground = false;
                if matches!(self.pose, PlayerPose::Sliding) {
                    self.pose = PlayerPose::Normal;
                }
            } else {
                if data.vel.y >= 0 {
                    data.vel.y = 0;
                } else {
                    data.on_ground = false;
                }
            }
        }
        let max_speed = if self.p_meter == 0x70 { 0x300 } else { 0x240 };
        let mut neutral = true;
        let can_move = match self.pose {
            PlayerPose::Normal => true,
            PlayerPose::Hurt => false,
            _ => !data.on_ground,
            //PlayerPose::Sliding => !data.on_ground,
            //PlayerPose::Crouching => !data.on_ground
        };
        if can_move {
            neutral = !buttons.left() && !buttons.right();
            let mut button = buttons.left() && !buttons.right();
            for i in 0..2 {
                data.vel.x = -data.vel.x;
                if button {
                    data.flip.x = i == 0;
                    if data.vel.x > 0 {
                        if data.vel.x < max_speed { data.vel.x += 0x18; }
                        if data.vel.x >= 0x240 && (data.on_ground || self.p_speed) { self.p_meter += 3; }
                    } else {
                        data.vel.x += 0x50;
                    }
                }
                button = buttons.right();
            }
        }
        if neutral && data.on_ground && self.slide_timer == 0 {
            self.p_speed = false;
            if data.vel.x > 0 {
                data.vel.x -= 0x10;
                if data.vel.x < 0 { data.vel.x = 0; }
            } else {
                data.vel.x += 0x10;
                if data.vel.x > 0 { data.vel.x = 0; }
            }
        }
        self.p_meter -= 1;
        if self.p_meter > 0x70 { self.p_speed = true; self.p_meter = 0x70; }
        if self.p_meter < 0 { self.p_meter = 0; }
        match self.pose {
            PlayerPose::Normal => {
                if data.on_ground {
                    if data.vel.x == 0 {
                        self.idle_timer += 1;
                        let b = self.idle_timer / 4 % 0x40;
                        data.sprites[1].frame = match b {
                            0x3F => 1,
                            0x3E => 2,
                            0x3D => 1,
                            _ => 0
                        };
                    } else {
                        self.idle_timer = 0;
                        data.anim_timer += data.vel.x.abs();
                        if data.anim_timer > 0xA00 {
                            data.anim_timer -= 0xA00;
                            data.sprites[1].frame += 1;
                        }
                        if data.vel.x.abs() > 0x260 {
                            if data.sprites[1].frame >= 16 || data.sprites[1].frame < 12 {
                                data.sprites[1].frame = 12;
                            }
                        } else {
                            if data.sprites[1].frame >= 12 || data.sprites[1].frame < 8 {
                                data.sprites[1].frame = 8;
                            }
                        }
                    }
                } else {
                    if data.vel.y > 0 {
                    } else {
                        data.sprites[1].frame = 3;
                    }
                }
            }
            PlayerPose::Sliding => {
                //data.sprites[1].frame = 7;
            }
            PlayerPose::Crouching => {
                //data.sprites[1].frame = 7;
            }
            PlayerPose::Hurt => {
                //data.sprites[1].frame = 7;
            }
        }
        if self.invuln_timer > 0 {
            //data.sprites[1].len = (self.invuln_timer / 2 & 1) as _;
            self.invuln_timer -= 1;
        } else {
            //data.sprites[1].len = 1;
        }

        for i in crate::state().as_level().entities.inner.iter_mut() {
            // do not interact with self
            if &mut i.data as *mut _ == data as *mut _ { continue; }
            if data.intersects(&i.data) {
                if !matches!(i.data.state, EntityState::Alive) { continue; }
                if matches!(i.kind, EntityKind::Tomato) {
                    if data.intersects(&i.data) {
                        if data.vel.y > 0 && i.data.pos.y - data.pos.y > 0x400 && self.can_interact() {
                            data.vel.y = -0x500;
                            i.data.state = EntityState::Squished;
                            i.data.anim_timer = 30;
                            i.data.sprites[0].frame = 3;
                        } else if self.invincible() {
                            /*crate::state().as_level().particles.slot()
                                .init_kick(i.data.pos);*/
                            i.data.state = EntityState::Dead;
                            i.data.vel = vec2(-0x40, -0x200);
                        } else if !self.invulnerable() {
                            self.hurt(data);
                        }
                    }
                }
            }
        }


        let old_x = data.vel.x;


        data.physics();
        unsafe {
            if self.pose == PlayerPose::Sliding && old_x.abs() > 0x180 {
                for i in 0..3 {
                    if *data.sensors[i] == 3 {
                        *data.sensors[i] = 0;
                        data.vel.x = old_x;
                        for v in 0..4 {
                            //crate::state().as_level().particles.slot()
                            //    .init_crumble(v, data.sensor_pos[i]);
                        }
                    }
                }
            }
            if !self.invulnerable() {
                for i in 3..6 {
                    if *data.sensors[i] == 6 {
                        self.hurt(data);
                    }
                }
            }
            for (idx,i) in data.sensors.iter_mut().enumerate() {
                match **i {
                    0x10 => {
                        **i = 0;
                        crate::state().as_level().coins += 1;
                        //crate::state().as_level().particles.slot()
                        //    .init_sparkle(data.sensor_pos[idx]);
                    },
                    0x11 => {
                        **i = 0;
                    }
                    0x12 => {
                        **i = 0;
                        crate::state().as_level().health += 1;
                    }
                    _ => {}
                }
            }
        }
        let fg = &mut crate::state().as_level().fg;

        if unsafe { *fg.mut_ptr_at(data.pos / 256 / 16) == 0x0F } && buttons.up_edge() && data.on_ground {
            crate::state().as_level().room += 1;
            crate::state().as_level().init_flag = true;
        }
        let level_w = fg.width() as i32 * 16 * 256 - 0x1000;
        let level_h = fg.height() as i32 * 16 * 256;
        if data.pos.x < 0x1000 { data.pos.x = 0x1000; data.vel.x = 0; }
        if data.pos.x > level_w { data.pos.x = level_w; data.vel.x = 0; }
        if data.pos.y > level_h { crate::state().as_level().init_flag = true; }
    }
    /*
    if data.blocked[1] || data.blocked[2] {
        self.p_speed = false;
        self.p_meter = 0;
    }*/
}
