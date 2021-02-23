use raylib::prelude::*;
use rna::*;

pub struct Game {
    camera: Camera2D,
}

impl Core for Game {
    fn initialize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {}
    fn update(&mut self, rl: &mut RaylibHandle, _: &RaylibThread) {}
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(41, 173, 255, 255));
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            camera: Camera2D {
                zoom: 1.0,
                target: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                offset: Vector2::new(0.0, 0.0),
            },
        }
    }
}
