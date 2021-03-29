mod game;
use game::Game;
use rna::*;

fn main() {
    let mut config = AppConfig::new();

    config.title = "Grid World";
    config.window_size = (150 * 4, 150 * 3);
    config.vsync_enabled = true;
    config.core = Some(Box::new(Game::new(std::env::args())));

    App::build(config).run();
}
