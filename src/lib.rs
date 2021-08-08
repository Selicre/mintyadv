#![no_std]

mod lz4;
mod vec2;
mod framebuffer;
mod data;
mod controller;

mod state;
mod utils;
mod entity;
mod foreground;
mod background;

use crate::framebuffer::Framebuffer;
use crate::state::GameState;

#[panic_handler]
unsafe fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    // Where we're going, we don't need safety.
    core::hint::unreachable_unchecked()
}
unsafe fn copy_fwd(src: *const u8, dest: *mut u8, n: usize) {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
}

#[no_mangle]
pub static mut BUF: Framebuffer = Framebuffer::new();

static mut STATE: GameState = GameState::new();

// You assume all responsibility for misusing this function.
pub fn state() -> &'static mut GameState {
    unsafe { &mut STATE }
}

#[no_mangle]
pub unsafe fn drw(b: u32) {
    STATE.run(&mut BUF, b);
}

#[no_mangle]
pub static mut SND: [f32; 1024] = [0.0; 1024];

#[no_mangle]
pub unsafe fn snd() {
    SND.copy_from_slice(&[0.0; 1024]);
    //STATE.fill_buf(&mut SND);
}


