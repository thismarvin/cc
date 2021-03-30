use crate::world::{Action, Analysis, State, World};
use raylib::prelude::*;
use rna::*;

pub struct Game {
    camera: Camera2D,
    world: World,
    analysis: Analysis,
}

impl Core for Game {
    fn initialize(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn update(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(0, 0, 0, 255));

        let size: usize = 150;

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                if self.world.valid_position(&state) {
                    if self.world.can_exit(&state) {
                        let value = self.world.reward(&state, Action::Exit);
                        d.draw_rectangle(
                            x as i32 * size as i32,
                            y as i32 * size as i32,
                            size as i32,
                            size as i32,
                            self.calculate_color(value),
                        );
                        d.draw_text(
                            format!("{:5.2}", value).as_str(),
                            x as i32 * size as i32 + (size as f32 * 0.18) as i32,
                            y as i32 * size as i32 + (size as f32 * 0.38) as i32,
                            40,
                            Color::WHITE,
                        );
                    } else {
                        self.draw_cell(
                            &mut d,
                            y * self.world.width + x,
                            x as f32 * size as f32,
                            y as f32 * size as f32,
                            size,
                        );
                    }
                }
            }
        }
    }
}

impl Game {
    pub fn new(mut args: std::env::Args) -> Self {
        args.next();

        let mut mode = String::from("value");
        let mut discount = 0.9;
        let mut noise = 0.8;
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

        let mut world = World::load(path.as_str()).unwrap_or(world);

        let analysis = match mode.as_str() {
            "policy" => world.policy_iteration(discount, noise, epsilon),
            _ => world.value_iteration(discount, noise, epsilon),
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
        let font_size = 20;
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

        d.draw_text(
            format!("{:5.2}", q_values[0]).as_str(),
            x as i32 + (size as f32 * 0.3) as i32,
            y as i32 + (size as f32 * 0.05) as i32,
            font_size,
            Color::WHITE,
        );
        d.draw_text(
            format!("{:5.2}", q_values[1]).as_str(),
            x as i32 + (size as f32 * 0.63) as i32,
            y as i32 + (size as f32 * 0.45) as i32,
            20,
            Color::WHITE,
        );
        d.draw_text(
            format!("{:5.2}", q_values[2]).as_str(),
            x as i32 + (size as f32 * 0.3) as i32,
            y as i32 + (size as f32 * 0.83) as i32,
            20,
            Color::WHITE,
        );
        d.draw_text(
            format!("{:5.2}", q_values[3]).as_str(),
            x as i32 + (size as f32 * 0.01) as i32,
            y as i32 + (size as f32 * 0.45) as i32,
            font_size,
            Color::WHITE,
        );
    }
}
