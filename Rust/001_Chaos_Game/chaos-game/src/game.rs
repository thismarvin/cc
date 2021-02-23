use rand::prelude::*;
use raylib::prelude::*;
use rna::*;

fn lerp(a: f64, b: f64, step: f64) -> f64 {
    a + (b - a) * step
}

pub struct Game {
    camera: Camera2D,
    rng: ThreadRng,
    vertices: Vec<Vector2>,
    points: Vec<Vector2>,
    last: Vector2,
    n: usize,
    r: f32,
    max: usize,
}

impl Core for Game {
    fn initialize(&mut self, rl: &mut RaylibHandle, _: &RaylibThread) {
        let size = rl.get_screen_width() as f32;
        let increment = std::f32::consts::TAU / self.n as f32;

        for i in 0..self.n {
            self.vertices.push(Vector2::new(
                size * 0.5 + (increment * i as f32).cos() * size * 0.45,
                size * 0.5 + (increment * i as f32).sin() * size * 0.45,
            ));
        }

        let x = self.random_range(0, size as isize) as f32;
        let y = self.random_range(0, size as isize) as f32;
        self.last = Vector2::new(x, y);
    }
    fn update(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {
        if self.points.len() >= self.max {
            return;
        }

        let iterations = 100;

        for _ in 0..iterations {
            let index = self.random_range(0, self.vertices.len() as isize);
            let target = self.vertices[index as usize];

            let x = lerp(self.last.x as f64, target.x as f64, self.r as f64) as f32;
            let y = lerp(self.last.y as f64, target.y as f64, self.r as f64) as f32;
            let result = Vector2::new(x, y);

            self.points.push(result);
            self.last = Vector2::new(result.x, result.y);
        }
    }
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::WHITE);

        for point in self.points.iter() {
            d.draw_circle(
                point.x as i32,
                point.y as i32,
                1.0,
                Color::new(0, 0, 0, 200),
            );
        }
    }
}

impl Game {
    pub fn new() -> Self {
        let n = 6;
        let r = 1.0 / 3.0;
        let maximum = 60000;

        Game {
            camera: Camera2D {
                zoom: 1.0,
                target: Vector2::new(300.0, 300.0),
                rotation: -90.0,
                offset: Vector2::new(300.0, 300.0),
            },
            rng: rand::thread_rng(),
            last: Vector2::new(0.0, 0.0),
            n,
            r: 1.0 - r,
            max: maximum,
            vertices: Vec::with_capacity(maximum),
            points: Vec::with_capacity(n),
        }
    }

    fn random_range(&mut self, low: isize, high: isize) -> isize {
        let r: f32 = self.rng.gen();
        low + (r * (high - low) as f32).trunc() as isize
    }
}
