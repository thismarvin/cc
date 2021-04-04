use crate::world::{Action, Analysis, Direction, State, World};
use raylib::prelude::*;
use rna::*;

enum Mode {
    Value,
    Policy,
}

pub struct Game {
    camera: Camera2D,
    world: World,
    analysis: Analysis,
    mode: Mode,
    discount: f32,
    noise: f32,
    show_policy: bool,
    accumulator: f32,
}

impl Game {
    pub fn new(mut args: std::env::Args) -> Self {
        args.next();

        let mut mode = String::from("value");
        let mut discount = 0.9;
        let mut noise = 0.2;
        let mut epsilon = 0.0001;
        let mut path = String::new();

        let args: Vec<String> = args.collect();
        for i in (0..args.len()).step_by(2) {
            if let (Some(flag), Some(value)) = (args.get(i), args.get(i + 1)) {
                match flag.as_str() {
                    "-M" | "--mode" => {
                        mode = value.to_lowercase();
                    }
                    "-D" | "--discount" => {
                        discount = value.parse::<f32>().unwrap_or(discount);
                    }
                    "-N" | "--noise" => {
                        noise = value.parse::<f32>().unwrap_or(noise);
                    }
                    "-E" | "--epsilon" => {
                        epsilon = value.parse::<f32>().unwrap_or(epsilon);
                    }
                    "-P" | "--path" => {
                        path = String::from(value);
                    }
                    _ => (),
                }
            }
        }

        let mut world = World::new(4, 3);
        world.add_wall(1, 1);
        world.add_exit(3, 0, 1.0);
        world.add_exit(3, 1, -1.0);

        let world = World::load(path.as_str()).unwrap_or(world);

        let mode = match mode.as_str() {
            "policy" => Mode::Policy,
            _ => Mode::Value,
        };

        let policy = match mode {
            Mode::Policy => world.generate_random_policy(),
            Mode::Value => vec![Action::None; world.area()],
        };

        let analysis = Analysis {
            policy,
            values: vec![0.0; world.area()],
            q_values: vec![[0.0; 4]; world.area()],
            min_value: 0.0,
            max_value: 0.0,
        };

        Game {
            camera: Camera2D {
                zoom: 1.0,
                target: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                offset: Vector2::new(0.0, 0.0),
            },
            world,
            analysis,
            mode,
            discount,
            noise,
            show_policy: false,
            accumulator: 0.0,
        }
    }

    fn calculate_color(&self, value: f32) -> Color {
        let color;
        if value < 0.0 {
            color = Color::new(
                rna::remap_range(
                    value as f64,
                    self.analysis.min_value as f64,
                    0.0,
                    255.0,
                    0.0,
                ) as u8,
                0,
                0,
                255,
            );
        } else {
            color = Color::new(
                0,
                rna::remap_range(
                    value as f64,
                    0.0,
                    self.analysis.max_value as f64,
                    0.0,
                    255.0,
                ) as u8,
                0,
                255,
            );
        }

        color
    }

    fn draw_cell(
        &self,
        d: &mut RaylibMode2D<RaylibDrawHandle>,
        index: usize,
        x: f32,
        y: f32,
        size: usize,
    ) {
        let q_values = self.analysis.q_values[index];

        d.draw_triangle(
            Vector2::new(x as f32, y as f32),
            Vector2::new(x as f32 + size as f32 * 0.5, y as f32 + size as f32 * 0.5),
            Vector2::new(x as f32 + size as f32, y as f32),
            self.calculate_color(q_values[0]),
        );
        d.draw_triangle(
            Vector2::new(x as f32 + size as f32, y as f32),
            Vector2::new(x as f32 + size as f32 * 0.5, y as f32 + size as f32 * 0.5),
            Vector2::new(x as f32 + size as f32, y as f32 + size as f32),
            self.calculate_color(q_values[1]),
        );
        d.draw_triangle(
            Vector2::new(x as f32 + size as f32, y as f32 + size as f32),
            Vector2::new(x as f32 + size as f32 * 0.5, y as f32 + size as f32 * 0.5),
            Vector2::new(x as f32, y as f32 + size as f32),
            self.calculate_color(q_values[2]),
        );
        d.draw_triangle(
            Vector2::new(x as f32, y as f32 + size as f32),
            Vector2::new(x as f32 + size as f32 * 0.5, y as f32 + size as f32 * 0.5),
            Vector2::new(x as f32, y as f32),
            self.calculate_color(q_values[3]),
        );
    }

    fn draw_policy(
        &self,
        d: &mut RaylibMode2D<RaylibDrawHandle>,
        x: usize,
        y: usize,
        x_offset: usize,
        y_offset: usize,
        size: usize,
    ) {
        if !self.show_policy {
            return;
        }

        match self.analysis.policy[y * self.world.width + x] {
            Action::Exit => {
                let padding = (size as f32 * 0.07) as i32;
                let thickness = (size as f32 * 0.05) as i32;
                let thickness = 1.max(thickness);
                d.draw_rectangle_lines_ex(
                    Rectangle::new(
                        x as f32 * size as f32 + x_offset as f32 + padding as f32,
                        y as f32 * size as f32 + y_offset as f32 + padding as f32,
                        size as f32 - padding as f32 * 2.0,
                        size as f32 - padding as f32 * 2.0,
                    ),
                    thickness,
                    Color::new(255, 255, 255, 155),
                )
            }
            Action::Move(direction) => match direction {
                Direction::Up => {
                    d.draw_triangle(
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.2 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.4 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.8 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.4 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.5 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.1 + y_offset as f32,
                        ),
                        Color::new(255, 255, 255, 155),
                    );
                    d.draw_rectangle(
                        (x as f32 * size as f32 + size as f32 * 0.35 + x_offset as f32).round()
                            as i32,
                        (y as f32 * size as f32 + size as f32 * 0.4 + y_offset as f32).round()
                            as i32,
                        (size as f32 * 0.3) as i32,
                        (size as f32 * 0.5) as i32,
                        Color::new(255, 255, 255, 155),
                    )
                }
                Direction::Right => {
                    d.draw_triangle(
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.6 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.2 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.6 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.8 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.9 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.5 + y_offset as f32,
                        ),
                        Color::new(255, 255, 255, 155),
                    );
                    d.draw_rectangle(
                        (x as f32 * size as f32 + size as f32 * 0.1 + x_offset as f32).round()
                            as i32,
                        (y as f32 * size as f32 + size as f32 * 0.35 + y_offset as f32).round()
                            as i32,
                        (size as f32 * 0.5) as i32,
                        (size as f32 * 0.3) as i32,
                        Color::new(255, 255, 255, 155),
                    )
                }
                Direction::Down => {
                    d.draw_triangle(
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.2 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.6 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.5 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.9 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.8 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.6 + y_offset as f32,
                        ),
                        Color::new(255, 255, 255, 155),
                    );
                    d.draw_rectangle(
                        (x as f32 * size as f32 + size as f32 * 0.35 + x_offset as f32).round()
                            as i32,
                        (y as f32 * size as f32 + size as f32 * 0.1 + y_offset as f32).round()
                            as i32,
                        (size as f32 * 0.3) as i32,
                        (size as f32 * 0.5) as i32,
                        Color::new(255, 255, 255, 155),
                    )
                }
                Direction::Left => {
                    d.draw_triangle(
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.4 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.2 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.1 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.5 + y_offset as f32,
                        ),
                        Vector2::new(
                            x as f32 * size as f32 + size as f32 * 0.4 + x_offset as f32,
                            y as f32 * size as f32 + size as f32 * 0.8 + y_offset as f32,
                        ),
                        Color::new(255, 255, 255, 155),
                    );
                    d.draw_rectangle(
                        (x as f32 * size as f32 + size as f32 * 0.4 + x_offset as f32).round()
                            as i32,
                        (y as f32 * size as f32 + size as f32 * 0.35 + y_offset as f32).round()
                            as i32,
                        (size as f32 * 0.5) as i32,
                        (size as f32 * 0.3) as i32,
                        Color::new(255, 255, 255, 155),
                    )
                }
            },
            Action::None => (),
        }
    }
}

impl Core for Game {
    fn initialize(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn update(&mut self, r: &mut RaylibHandle, _: &RaylibThread) {
        if r.is_key_pressed(KeyboardKey::KEY_SPACE) {
            self.show_policy = !self.show_policy;
        }

        self.accumulator += r.get_frame_time();

        if self.accumulator > 0.2 {
            self.accumulator = 0.0;

            // Look so the following isn't technically correct; however, doing it like this makes the visualization cooler!
            match self.mode {
                Mode::Value => {
                    self.analysis.values = self.world.value_bellman_update(
                        self.discount,
                        self.noise,
                        &self.analysis.values,
                        &mut self.analysis.q_values,
                    );
                    self.analysis.policy = self.world.generate_policy(&self.analysis.q_values);
                }
                Mode::Policy => {
                    self.analysis.values = self.world.policy_bellman_update(
                        self.discount,
                        self.noise,
                        &self.analysis.policy,
                        &self.analysis.values,
                    );
                    let (temp, _) = self.world.policy_improvement(
                        self.discount,
                        self.noise,
                        &self.analysis.policy,
                        &self.analysis.values,
                        &mut self.analysis.q_values,
                    );
                    self.analysis.policy = temp;
                }
            }

            self.analysis.min_value = Analysis::min(&self.analysis.values);
            self.analysis.max_value = Analysis::max(&self.analysis.values);
        }
    }
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(0, 0, 0, 255));

        let size = d.get_screen_width() as usize / self.world.width;
        let size = size.min(d.get_screen_height() as usize / self.world.height);
        let x_offset = (d.get_screen_width() as usize - self.world.width * size) / 2;
        let y_offset = (d.get_screen_height() as usize - self.world.height * size) / 2;

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                if self.world.valid_position(&state) {
                    if self.world.can_exit(&state) {
                        let value = self.world.reward(&state, Action::Exit);
                        d.draw_rectangle(
                            x as i32 * size as i32 + x_offset as i32,
                            y as i32 * size as i32 + y_offset as i32,
                            size as i32,
                            size as i32,
                            self.calculate_color(value),
                        );
                    } else {
                        self.draw_cell(
                            &mut d,
                            y * self.world.width + x,
                            x as f32 * size as f32 + x_offset as f32,
                            y as f32 * size as f32 + y_offset as f32,
                            size,
                        );
                    }
                    self.draw_policy(&mut d, x, y, x_offset, y_offset, size)
                }
            }
        }
    }
}
