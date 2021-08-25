use raylib::prelude::*;

use super::app_config::AppConfig;
use super::core::Core;

pub struct App {
    rl: RaylibHandle,
    thread: RaylibThread,
    core: Box<dyn Core>,
}

impl App {
    pub fn build(config: AppConfig) -> Self {
        if config.core.is_none() {
            panic!("An RNA Core was not present; could not create application.");
        }

        let mut builder = raylib::init();

        builder.title(config.title);
        builder.size(config.window_size.0, config.window_size.1);

        if config.vsync_enabled {
            builder.vsync();
        }

        if config.resizable {
            builder.resizable();
        }

        if config.borderless {
            builder.undecorated();
        }

        let (rl, thread) = builder.build();

        App {
            rl,
            thread,
            core: config.core.unwrap(),
        }
    }

    pub fn run(&mut self) -> &mut Self {
        self.core.initialize(&mut self.rl, &self.thread);
        while !self.rl.window_should_close() {
            self.core.update(&mut self.rl, &self.thread);
            let mut d = self.rl.begin_drawing(&self.thread);
            self.core.draw(&mut d, &self.thread);
        }

        self
    }
}
