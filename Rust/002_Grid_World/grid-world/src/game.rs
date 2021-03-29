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

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

#[derive(Clone, Copy, PartialEq)]
enum Action {
    None,
    Move(Direction),
    Exit,
}

// TODO:
// - ability to load a world from a file?
struct World {
    width: usize,
    height: usize,
    board: Vec<usize>,
    exits: Vec<Option<f32>>,
    noise: f32,

    values: Vec<f32>,
    q_values: Vec<[f32; 4]>,
    min_value: f32,
    max_value: f32,
}

impl World {
    pub fn new(width: usize, height: usize, noise: f32) -> Self {
        World {
            width,
            height,
            board: vec![0; width * height],
            exits: vec![None; width * height],
            noise,
            values: vec![0.0; width * height],
            q_values: vec![[0.0; 4]; width * height],
            min_value: std::f32::MAX,
            max_value: std::f32::MIN,
        }
    }

    pub fn area(&self) -> usize {
        self.width * self.height
    }

    pub fn add_wall(&mut self, x: usize, y: usize) {
        if let Some(target) = self.board.get_mut(y * self.width + x) {
            *target = 1;
        }
    }

    pub fn add_exit(&mut self, x: usize, y: usize, reward: f32) {
        if let Some(target) = self.exits.get_mut(y * self.width + x) {
            *target = Some(reward);
        }
    }

    pub fn valid_position(&self, state: &State) -> bool {
        if let Some(target) = self.board.get(state.y * self.width + state.x) {
            return *target == 0;
        }
        false
    }

    pub fn can_exit(&self, state: &State) -> bool {
        if let Some(target) = self.exits.get(state.y * self.width + state.x) {
            if target.is_some() {
                return true;
            }
        }

        false
    }

    pub fn transition(&self, state: &State, action: Action) -> Option<Vec<(f32, Action)>> {
        match action {
            Action::Exit => {
                if let Some(target) = self.exits.get(state.y * self.width + state.x) {
                    if target.is_some() {
                        return Some(vec![(1.0, Action::Exit)]);
                    }
                }
                None
            }
            Action::Move(direction) => {
                let moves = self.get_moves(direction);
                let mut result = Vec::with_capacity(moves.len());
                result.push((self.noise, Action::Move(moves[0])));
                let remainder = (1.0 - self.noise) / (moves.len() as f32 - 1.0);
                for i in 1..moves.len() {
                    result.push((remainder, Action::Move(moves[i])));
                }
                return Some(result);
            }
            Action::None => None,
        }
    }

    pub fn reward(&self, state: &State, action: Action) -> f32 {
        match action {
            Action::Exit => {
                if let Some(target) = self.exits.get(state.y * self.width + state.x) {
                    if let Some(reward) = target {
                        return *reward;
                    }
                }
                0.0
            }
            Action::Move(_) => 0.0,
            Action::None => 0.0,
        }
    }

    pub fn move_to(&self, state: &State, direction: Direction) -> State {
        match direction {
            Direction::Up if state.y > 0 => {
                if self.valid_position(&State::new(state.x, state.y - 1)) {
                    return State::new(state.x, state.y - 1);
                }
            }
            Direction::Down if state.y < self.height - 1 => {
                if self.valid_position(&State::new(state.x, state.y + 1)) {
                    return State::new(state.x, state.y + 1);
                }
            }
            Direction::Left if state.x > 0 => {
                if self.valid_position(&State::new(state.x - 1, state.y)) {
                    return State::new(state.x - 1, state.y);
                }
            }
            Direction::Right if state.x < self.width - 1 => {
                if self.valid_position(&State::new(state.x + 1, state.y)) {
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
}

pub struct Game {
    camera: Camera2D,
    world: World,
    gamma: f32,
}

impl Core for Game {
    fn initialize(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {
        let epsilon = 0.0001;
        let (_, values) = self.value_iteration(epsilon);

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

        let gamma = args
            .next()
            .unwrap_or(String::from("0.9"))
            .parse::<f32>()
            .unwrap_or(0.9);

        let mut world = World::new(4, 3, 0.8);
        world.add_wall(1, 1);
        world.add_exit(3, 0, 1.0);
        world.add_exit(3, 1, -1.0);

        Game {
            camera: Camera2D {
                zoom: 1.0,
                target: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                offset: Vector2::new(0.0, 0.0),
            },
            world,
            gamma,
        }
    }

    fn value_bellman(&mut self, values: &Vec<f32>) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let index = y * self.world.width + x;
                let state = State::new(x, y);

                // If we happen to be in an invalid position then move along!
                if !self.world.valid_position(&state) {
                    result[index] = 0.0;
                    continue;
                }

                // If we can exit then we must exit.
                if self.world.can_exit(&state) {
                    result[index] = self.world.reward(&state, Action::Exit);
                    continue;
                }

                // In order to find the optimal policy we must recursively calculate the expected value for each possible action in the
                // current state. The action with the hightest value is our final target.

                let mut new_values = [0.0; 4];

                for (i, direction) in DIRECTIONS.iter().enumerate() {
                    let actions = self.world.transition(&state, Action::Move(*direction));

                    if let Some(actions) = actions {
                        for entry in actions {
                            if let Action::Move(direction) = entry.1 {
                                let target = self.world.move_to(&state, direction);
                                new_values[i] += entry.0
                                    * (self.world.reward(&state, entry.1)
                                        + self.gamma
                                            * values[target.y * self.world.width + target.x]);
                            }
                        }
                    }
                }

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

    fn value_iteration(&mut self, epsilon: f32) -> (Vec<Action>, Vec<f32>) {
        let mut values = vec![0.0; self.world.area()];

        let mut iterations = 0;
        loop {
            // Loop until convergence.
            iterations += 1;

            let temp = self.value_bellman(&values);
            let deltas = temp.iter().enumerate().map(|(i, v)| *v - values[i]);

            let mut max_delta = f32::MIN;
            for delta in deltas {
                if delta > max_delta {
                    max_delta = delta;
                }
            }

            values = temp;

            if max_delta.abs() < epsilon && iterations > 1 {
                break;
            }
        }

        let mut policy = vec![Action::None; self.world.area()];
        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let index = y * self.world.width + x;
                let state = State::new(x, y);

                if !self.world.valid_position(&state) {
                    policy[index] = Action::None;
                    continue;
                }

                if self.world.can_exit(&state) {
                    policy[index] = Action::Exit;
                    continue;
                }

                let mut target = 0;
                for i in 1..self.world.q_values[index].len() {
                    if self.world.q_values[index][i] > self.world.q_values[index][target] {
                        target = i;
                    }
                }

                policy[index] = Action::Move(DIRECTIONS[target]);
            }
        }

        (policy, values)
    }

    fn policy_bellman(&mut self, policy: &Vec<Action>, values: &Vec<f32>) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let index = y * self.world.width + x;
                let state = State::new(x, y);

                result[index] = match policy[index] {
                    Action::Exit => self.world.reward(&state, policy[index]),
                    Action::Move(direction) => {
                        let actions = self.world.transition(&state, Action::Move(direction));

                        let mut value = 0.0;
                        if let Some(actions) = actions {
                            for entry in actions {
                                if let Action::Move(direction) = entry.1 {
                                    let target = self.world.move_to(&state, direction);
                                    value += entry.0
                                        * (self.world.reward(&state, entry.1)
                                            + self.gamma
                                                * values[target.y * self.world.width + target.x]);
                                }
                            }
                        }

                        match direction {
                            Direction::Up => self.world.q_values[index][0] = value,
                            Direction::Right => self.world.q_values[index][1] = value,
                            Direction::Down => self.world.q_values[index][2] = value,
                            Direction::Left => self.world.q_values[index][3] = value,
                        }

                        value
                    }
                    Action::None => 0.0,
                };
            }
        }

        result
    }

    fn policy_improvement(&self, policy: &Vec<Action>, values: &Vec<f32>) -> Vec<Action> {
        let mut new_policy = vec![Action::None; self.world.area()];

        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let index = y * self.world.width + x;
                let state = State::new(x, y);

                new_policy[index] = match policy[index] {
                    Action::Exit => policy[index],
                    Action::Move(direction) => {
                        let mut optimal = direction;
                        let mut max = f32::MIN;

                        for direction in DIRECTIONS.iter() {
                            let temp = self.world.move_to(&state, *direction);
                            let value = values[temp.y * self.world.width + temp.x];
                            if value > max {
                                max = value;
                                optimal = *direction;
                            }
                        }

                        Action::Move(optimal)
                    }
                    Action::None => policy[index],
                };
            }
        }

        new_policy
    }

    fn policy_iteration(&mut self, epsilon: f32) -> (Vec<Action>, Vec<f32>) {
        // Create a valid random policy.
        let mut policy = Vec::with_capacity(self.world.area());
        for y in 0..self.world.height {
            for x in 0..self.world.width {
                let state = State::new(x, y);
                if !self.world.valid_position(&state) {
                    policy.push(Action::None);
                    continue;
                }
                if self.world.can_exit(&state) {
                    policy.push(Action::Exit);
                    continue;
                }

                // TODO: this works, but what would happen if the policy was truly random?
                policy.push(Action::Move(Direction::Up));
            }
        }

        let mut values = vec![0.0; self.world.area()];

        let mut iterations = 0;
        'outer: loop {
            // Loop until convergence.
            loop {
                // Policy Evaluation:
                iterations += 1;
                let temp = self.policy_bellman(&policy, &values);
                let deltas = temp.iter().enumerate().map(|(i, v)| *v - values[i]);

                let mut max_delta = f32::MIN;
                for delta in deltas {
                    if delta > max_delta {
                        max_delta = delta;
                    }
                }

                values = temp;

                if max_delta.abs() < epsilon && iterations > 1 {
                    break;
                }
            }

            // Policy Improvement.
            let temp = self.policy_improvement(&policy, &values);
            for i in 0..policy.len() {
                if !(temp[i] == policy[i]) {
                    // The policy is not stable; another pass of policy iteration -- using the new
                    // policy -- is required.
                    policy = temp;
                    break;
                }
                if i == policy.len() - 1 {
                    // The policy is stable; policy iteration is complete.
                    break 'outer;
                }
            }
        }

        (policy, values)
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
