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
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        World {
            width,
            height,
            board: vec![0; width * height],
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
        for y in 0..self.world.height {
            for x in 0..self.world.width {
                print!("{:5.2}, ", self.value(State::new(x, y), self.k));
            }
            println!();
        }
    }
    fn update(&mut self, _: &mut RaylibHandle, _: &RaylibThread) {}
    fn draw(&self, d: &mut RaylibDrawHandle, _: &RaylibThread) {
        let mut d = d.begin_mode2D(self.camera);
        d.clear_background(Color::new(41, 173, 255, 255));
    }
}

impl Game {
    pub fn new() -> Self {
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
            gamma: 0.9,
            k: 5,
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

    // Value Iteration
    fn value(&self, state: State, depth: usize) -> f32 {
        // If we happen to be in an invalid position then stop!
        if !self.world.is_valid(state.x, state.y) {
            return 0.0;
        }

        // If the depth is zero or we are at an "end" state then we no longer need to look ahead.
        if depth == 0 {
            return 0.0;
        }

        // If we can exit then we must exit.
        if self.can_exit(&state) {
            return self.reward(Action::Exit(&state));
        }

        // We need to determine the optimal policy.
        // In order to do so we must recursively find the expected value for each possible action in the
        // current state. The action with the hightest value is our final target.

        // T(s,a,s') * [R(s,a,s') + gamma * V(s', depth - 1)]

        let mut values: [f32; 4] = [0.0; 4];

        let target = self.move_to(&state, Direction::Up);
        let misstep_a = self.move_to(&state, Direction::Left);
        let misstep_b = self.move_to(&state, Direction::Right);
        values[0] = 0.8
            * (self.reward(Action::Move(&target)) + self.gamma * self.value(target, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_a))
                    + self.gamma * self.value(misstep_a, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_b))
                    + self.gamma * self.value(misstep_b, depth - 1));

        let target = self.move_to(&state, Direction::Right);
        let misstep_a = self.move_to(&state, Direction::Up);
        let misstep_b = self.move_to(&state, Direction::Down);
        values[1] = 0.8
            * (self.reward(Action::Move(&target)) + self.gamma * self.value(target, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_a))
                    + self.gamma * self.value(misstep_a, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_b))
                    + self.gamma * self.value(misstep_b, depth - 1));

        let target = self.move_to(&state, Direction::Down);
        let misstep_a = self.move_to(&state, Direction::Right);
        let misstep_b = self.move_to(&state, Direction::Left);
        values[2] = 0.8
            * (self.reward(Action::Move(&target)) + self.gamma * self.value(target, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_a))
                    + self.gamma * self.value(misstep_a, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_b))
                    + self.gamma * self.value(misstep_b, depth - 1));

        let target = self.move_to(&state, Direction::Left);
        let misstep_a = self.move_to(&state, Direction::Down);
        let misstep_b = self.move_to(&state, Direction::Up);
        values[3] = 0.8
            * (self.reward(Action::Move(&target)) + self.gamma * self.value(target, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_a))
                    + self.gamma * self.value(misstep_a, depth - 1))
            + 0.1
                * (self.reward(Action::Move(&misstep_b))
                    + self.gamma * self.value(misstep_b, depth - 1));

        // Find the highest value.
        let mut max = &values[0];
        for i in 1..values.len() {
            if &values[i] > max {
                max = &values[i]
            }
        }

        *max
    }
}
