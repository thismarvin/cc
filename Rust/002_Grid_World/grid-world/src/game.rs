use raylib::prelude::*;
use rna::*;

struct State {
    x: usize,
    y: usize,
}

impl State {
    fn new(x: usize, y: usize) -> Self {
        State { x, y }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Action<'a> {
    Move(&'a State),
    Exit(&'a State),
}

struct World {
    width: usize,
    height: usize,
    board: Vec<usize>,
    values: Vec<f32>,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        World {
            width,
            height,
            board: vec![0; width * height],
            values: vec![0.0; width * height],
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<&usize> {
        self.board.get(y * self.width + x)
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut usize> {
        self.board.get_mut(y * self.width + x)
    }

    fn set(&mut self, x: usize, y: usize, value: usize) {
        if let Some(target) = self.get_mut(x, y) {
            *target = value;
        }
    }

    fn is_valid(&self, x: usize, y: usize) -> bool {
        if let Some(target) = self.get(x, y) {
            return *target == 0;
        }
        false
    }

    fn debug(&self) {
        let mut temp = 0;
        for i in 0..self.board.len() {
            print!("{}", self.board[i]);
            temp += 1;
            if temp >= self.width {
                temp = 0;
                println!();
            }
        }
    }
}

pub struct Game {
    camera: Camera2D,
    world: World,
    gamma: f32,
    k: usize,
}

impl Core for Game {
    fn initialize(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {
        // Initialize all values with zero.
        let mut values = vec![0.0; self.world.width * self.world.height];
        // Use a bottom up approach to calculate the values of each state.
        for _ in 0..self.k {
            values = self.value_iteration(values);
        }
        self.world.values = values;
    }
    fn update(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(41, 173, 255, 255));

        let mut max = std::f32::MIN;
        for value in self.world.values.iter() {
            if *value > max {
                max = *value;
            }
        }

        let mut min = std::f32::MAX;
        for value in self.world.values.iter() {
            if *value < min {
                min = *value;
            }
        }

        let size = 100;
        let margin = size as f64 * 0.05;

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let value = self.world.values[y * self.world.width + x];
                let color;
                if value < 0.0 {
                    color = Color::new(
                        rna::remap_range(value as f64, min as f64, 0.0, 255.0, 0.0) as u8,
                        0,
                        0,
                        255,
                    );
                } else {
                    color = Color::new(
                        0,
                        rna::remap_range(value as f64, 0.0, max as f64, 0.0, 255.0) as u8,
                        0,
                        255,
                    );
                }

                d.draw_rectangle(
                    x as i32 * size,
                    y as i32 * size,
                    (size as f64 - margin) as i32,
                    (size as f64 - margin) as i32,
                    color,
                );

                if self.world.is_valid(x, y) {
                    d.draw_text(
                        format!("{:5.2}", value).as_str(),
                        x as i32 * size + 8,
                        y as i32 * size + 8,
                        20,
                        Color::WHITE,
                    );
                }
            }
        }
    }
}

impl Game {
    pub fn new(mut args: std::env::Args) -> Self {
        args.next();

        let k = args
            .next()
            .unwrap_or(String::from("100"))
            .parse::<usize>()
            .unwrap_or(100);

        let gamma = args
            .next()
            .unwrap_or(String::from("0.9"))
            .parse::<f32>()
            .unwrap_or(0.9);

        let mut world = World::new(4, 3);
        world.set(1, 1, 1);

        Game {
            camera: Camera2D {
                zoom: 1.0,
                target: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                offset: Vector2::new(0.0, 0.0),
            },
            world,
            gamma,
            k,
        }
    }

    fn move_to(&self, state: &State, direction: Direction) -> State {
        match direction {
            Direction::Up if state.y > 0 => {
                if self.world.is_valid(state.x, state.y - 1) {
                    return State::new(state.x, state.y - 1);
                }
            }
            Direction::Down if state.y < self.world.height - 1 => {
                if self.world.is_valid(state.x, state.y + 1) {
                    return State::new(state.x, state.y + 1);
                }
            }
            Direction::Left if state.x > 0 => {
                if self.world.is_valid(state.x - 1, state.y) {
                    return State::new(state.x - 1, state.y);
                }
            }
            Direction::Right if state.x < self.world.width - 1 => {
                if self.world.is_valid(state.x + 1, state.y) {
                    return State::new(state.x + 1, state.y);
                }
            }
            _ => {
                return State::new(state.x, state.y);
            }
        }

        State::new(state.x, state.y)
    }

    fn can_exit(&self, state: &State) -> bool {
        match state {
            State { x: 3, y: 0 } => true,
            State { x: 3, y: 1 } => true,
            _ => false,
        }
    }

    fn reward(&self, action: Action) -> f32 {
        match action {
            Action::Move(_) => {
                return 0.0;
            }
            Action::Exit(state) => {
                return match state {
                    State { x: 3, y: 0 } => 1.0,
                    State { x: 3, y: 1 } => -1.0,
                    _ => 0.0,
                };
            }
        }
    }

    // To be honest, I am not sure if I am doing value iteration or policy iteration.
    fn value_iteration(&mut self, values: Vec<f32>) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                let index = y * self.world.width + x;

                // If we happen to be in an invalid position then stop!
                if !self.world.is_valid(state.x, state.y) {
                    result[index] = 0.0;
                    continue;
                }

                // If we can exit then we must exit.
                if self.can_exit(&state) {
                    result[index] = self.reward(Action::Exit(&state));
                    continue;
                }

                // We need to find the optimal policy.
                // In order to do so we must recursively find the expected value for each possible action in the
                // current state. The action with the hightest value is our final target.

                // T(s,a,s') * [R(s,a,s') + gamma * V(s', depth - 1)]

                let mut new_values: [f32; 4] = [0.0; 4];

                let target = self.move_to(&state, Direction::Up);
                let misstep_a = self.move_to(&state, Direction::Left);
                let misstep_b = self.move_to(&state, Direction::Right);
                new_values[0] = 0.8
                    * (self.reward(Action::Move(&target))
                        + self.gamma * values[target.y * self.world.width + target.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_a))
                            + self.gamma * values[misstep_a.y * self.world.width + misstep_a.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_b))
                            + self.gamma * values[misstep_b.y * self.world.width + misstep_b.x]);

                let target = self.move_to(&state, Direction::Right);
                let misstep_a = self.move_to(&state, Direction::Up);
                let misstep_b = self.move_to(&state, Direction::Down);
                new_values[1] = 0.8
                    * (self.reward(Action::Move(&target))
                        + self.gamma * values[target.y * self.world.width + target.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_a))
                            + self.gamma * values[misstep_a.y * self.world.width + misstep_a.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_b))
                            + self.gamma * values[misstep_b.y * self.world.width + misstep_b.x]);

                let target = self.move_to(&state, Direction::Down);
                let misstep_a = self.move_to(&state, Direction::Right);
                let misstep_b = self.move_to(&state, Direction::Left);
                new_values[2] = 0.8
                    * (self.reward(Action::Move(&target))
                        + self.gamma * values[target.y * self.world.width + target.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_a))
                            + self.gamma * values[misstep_a.y * self.world.width + misstep_a.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_b))
                            + self.gamma * values[misstep_b.y * self.world.width + misstep_b.x]);

                let target = self.move_to(&state, Direction::Left);
                let misstep_a = self.move_to(&state, Direction::Down);
                let misstep_b = self.move_to(&state, Direction::Up);
                new_values[3] = 0.8
                    * (self.reward(Action::Move(&target))
                        + self.gamma * values[target.y * self.world.width + target.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_a))
                            + self.gamma * values[misstep_a.y * self.world.width + misstep_a.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_b))
                            + self.gamma * values[misstep_b.y * self.world.width + misstep_b.x]);

                // Find the highest value.
                let mut max = new_values[0];
                for i in 1..new_values.len() {
                    if new_values[i] > max {
                        max = new_values[i]
                    }
                }

                result[index] = max;
            }
        }

        result
    }
}
