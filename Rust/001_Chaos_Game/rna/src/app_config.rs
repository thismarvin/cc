use super::core::Core;

pub struct AppConfig<'a> {
    pub title: &'a str,
    pub window_size: (i32, i32),
    pub vsync_enabled: bool,
    pub resizable: bool,
    pub borderless: bool,
    pub core: Option<Box<dyn Core>>,
}

impl<'a> AppConfig<'a> {
    pub fn new() -> Self {
        AppConfig {
            title: "Application",
            window_size: (640, 360),
            vsync_enabled: false,
            resizable: false,
            borderless: false,
            core: None,
        }
    }
}
