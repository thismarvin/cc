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

#[derive(Clone, Copy, PartialEq)]
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
    q_values: Vec<[f32; 4]>,
    min_value: f32,
    max_value: f32,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        World {
            width,
            height,
            board: vec![0; width * height],
            values: vec![0.0; width * height],
            q_values: vec![[0.0; 4]; width * height],
            min_value: std::f32::MAX,
            max_value: std::f32::MIN,
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
            values = self.value_iteration(&values);
        }

        // let mut policy = vec![Direction::Up; self.world.width * self.world.height];

        // let epsilon = 0.0001;
        // let mut iterations = 0;
        // let mut difference;

        // // Value iteration until convergence.
        // loop {
        //     let b = self.value_iteration(&values);
        //     difference = b
        //         .iter()
        //         .enumerate()
        //         .map(|(i, v)| *v - values[i])
        //         .fold(0.0, |a, v| a + v)
        //         / values.len() as f32;
        //     values = b;
        //     iterations += 1;

        //     if difference.abs() < epsilon && iterations > 1 {
        //         println!("{}", iterations);
        //         break;
        //     }
        // }

        // // Policy Iteration until convergence.
        // loop {
        //     let (a, b) = self.policy_iteration(policy, &values);
        //     difference = b
        //         .iter()
        //         .enumerate()
        //         .map(|(i, v)| *v - values[i])
        //         .fold(0.0, |a, v| a + v)
        //         / values.len() as f32;
        //     policy = a;
        //     values = b;
        //     iterations += 1;

        //     if difference.abs() < epsilon && iterations > 1 {
        //         println!("{}", iterations);
        //         break;
        //     }
        // }

        // // Policy Iteration until policy convergence.
        // 'outer: loop {
        //     let (a, b) = self.policy_iteration(&policy, &values);
        //     values = b;
        //     iterations += 1;

        //     if iterations > 1 {
        //         for i in 0..policy.len() {
        //             if !(a[i] == policy[i]) {
        //                 policy = a;
        //                 break;
        //             }
        //             if i == policy.len() - 1 {
        //                 policy = a;
        //                 println!("{}", iterations);
        //                 break 'outer;
        //             }
        //         }
        //     }
        // }

        self.world.values = values;

        for value in self.world.values.iter() {
            if *value > self.world.max_value {
                self.world.max_value = *value;
            }
        }

        for value in self.world.values.iter() {
            if *value < self.world.min_value {
                self.world.min_value = *value;
            }
        }
    }
    fn update(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(0, 0, 0, 255));

        let size: usize = 150;

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                if self.world.is_valid(x, y) {
                    self.draw_cell(
                        &mut d,
                        y * self.world.width + x,
                        x as f32 * size as f32,
                        y as f32 * size as f32,
                        size,
                    );

                    if self.can_exit(&State::new(x, y)) {
                        let value = self.world.values[y * self.world.width + x];
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
                    }
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

    fn get_moves(&self, direction: Direction) -> [Direction; 3] {
        match direction {
            Direction::Up | Direction::Down => [direction, Direction::Left, Direction::Right],
            Direction::Right | Direction::Left => [direction, Direction::Up, Direction::Down],
        }
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

    fn value_iteration(&mut self, values: &Vec<f32>) -> Vec<f32> {
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

                self.world.q_values[index] = new_values;

                result[index] = max;
            }
        }

        result
    }

    fn policy_evaluation(&mut self, policy: &Vec<Direction>, values: &Vec<f32>) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                let index = y * self.world.width + x;

                // If we happen to be in an invalid position then stop!
                if !self.world.is_valid(state.x, state.y) {
                    continue;
                }

                // If we can exit then we must exit.
                if self.can_exit(&state) {
                    result[index] = self.reward(Action::Exit(&state));
                    continue;
                }

                let moves = self.get_moves(policy[index]);
                let target = self.move_to(&state, moves[0]);
                let misstep_a = self.move_to(&state, moves[1]);
                let misstep_b = self.move_to(&state, moves[2]);
                let value = 0.8
                    * (self.reward(Action::Move(&target))
                        + self.gamma * values[target.y * self.world.width + target.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_a))
                            + self.gamma * values[misstep_a.y * self.world.width + misstep_a.x])
                    + 0.1
                        * (self.reward(Action::Move(&misstep_b))
                            + self.gamma * values[misstep_b.y * self.world.width + misstep_b.x]);

                result[index] = value;

                match policy[index] {
                    Direction::Up => self.world.q_values[index][0] = value,
                    Direction::Right => self.world.q_values[index][1] = value,
                    Direction::Down => self.world.q_values[index][2] = value,
                    Direction::Left => self.world.q_values[index][3] = value,
                }
            }
        }

        result
    }

    fn policy_improvement(&self, policy: &Vec<Direction>, values: &Vec<f32>) -> Vec<Direction> {
        let mut new_policy = policy.clone();

        let directions = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                let index = y * self.world.width + x;

                let mut action = policy[index];
                let temp = self.move_to(&state, action);
                let mut max = values[temp.y * self.world.width + temp.x];

                for direction in directions.iter() {
                    let temp = self.move_to(&state, *direction);
                    let value = values[temp.y * self.world.width + temp.x];
                    if value > max {
                        max = value;
                        action = *direction;
                    }
                }

                new_policy[index] = action;
            }
        }

        new_policy
    }

    fn policy_iteration(
        &mut self,
        policy: &Vec<Direction>,
        values: &Vec<f32>,
    ) -> (Vec<Direction>, Vec<f32>) {
        let new_values = self.policy_evaluation(&policy, &values);
        let new_policy = self.policy_improvement(&policy, &values);

        (new_policy, new_values)
    }

    fn calculate_color(&self, value: f32) -> Color {
        let color;
        if value < 0.0 {
            color = Color::new(
                rna::remap_range(value as f64, self.world.min_value as f64, 0.0, 255.0, 0.0) as u8,
                0,
                0,
                255,
            );
        } else {
            color = Color::new(
                0,
                rna::remap_range(value as f64, 0.0, self.world.max_value as f64, 0.0, 255.0) as u8,
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
        let q_values = self.world.q_values[index];

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
            x as i32 + (size as f32 * 0.58) as i32,
            y as i32 + (size as f32 * 0.45) as i32,
            20,
            Color::WHITE,
        );
        d.draw_text(
            format!("{:5.2}", q_values[2]).as_str(),
            x as i32 + (size as f32 * 0.3) as i32,
            y as i32 + (size as f32 * 0.78) as i32,
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
