use raylib::prelude::*;

pub trait Core {
    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread);
    fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread);
    fn draw(&self, d: &mut RaylibDrawHandle, thread: &RaylibThread);
}
