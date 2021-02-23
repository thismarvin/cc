mod game;
use game::Game;
use rna::*;

fn main() {
    let mut config = AppConfig::new();

    config.title = "Chaos Game";
    config.window_size = (600, 600);
    config.vsync_enabled = true;
    config.core = Some(Box::new(Game::new()));

    App::build(config).run();
}
