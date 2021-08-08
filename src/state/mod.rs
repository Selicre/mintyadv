mod title;
mod level;

use core::mem::ManuallyDrop;
use crate::framebuffer::Framebuffer;
use crate::controller::Buttons;

pub struct GameState {
    pub buttons: Buttons,
    pub id: GameStateId,
    pub data: GameStateData
}

pub enum GameStateId {
    Init,
    Title,
    Level
}

pub union GameStateData {
    init: (),
    title: ManuallyDrop<title::TitleState>,
    level: ManuallyDrop<level::LevelState>
}

impl GameState {
    pub const fn new() -> Self {
        Self {
            buttons: Buttons::new(),
            id: GameStateId::Init,
            data: GameStateData { init: () }
        }
    }
    pub fn as_level(&mut self) -> &mut level::LevelState {
        unsafe { &mut self.data.level }
    }
    pub fn as_title(&mut self) -> &mut title::TitleState {
        unsafe { &mut self.data.title }
    }
    pub fn run(&mut self, fb: &mut Framebuffer, b: u32) {
        self.buttons.update(b);
        let b = self.buttons;
        match self.id {
            GameStateId::Init => {
                crate::data::init();
                self.id = GameStateId::Title;
                self.as_title().init_flag = true;
            }
            GameStateId::Title => {
                self.as_title().run(fb,b);
            },
            GameStateId::Level => {
                self.as_level().run(fb,b);
            },
        }
    }
}
