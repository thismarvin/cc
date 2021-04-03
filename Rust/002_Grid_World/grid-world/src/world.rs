pub struct State {
    pub x: usize,
    pub y: usize,
}

impl State {
    pub fn new(x: usize, y: usize) -> Self {
        State { x, y }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    None,
    Move(Direction),
    Exit,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub board: Vec<usize>,
    pub exits: Vec<Option<f32>>,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        World {
            width,
            height,
            board: vec![0; width * height],
            exits: vec![None; width * height],
        }
    }

    pub fn load(path: &str) -> Result<World, Box<dyn std::error::Error>> {
        let mut width = 0;
        let mut height = 0;
        let mut walls = Vec::new();
        let mut exits = Vec::new();

        let data = std::fs::read_to_string(path)?;

        let lines = data.split("\n");
        for line in lines {
            let line = line.trim();
            if line.starts_with("#") {
                continue;
            }

            if line.starts_with("Dimension") {
                let mut temp = line.split_whitespace();
                temp.next();
                let data = temp.next().ok_or("Expected more than one entry.")?;
                let values: Vec<&str> = data.split(",").collect();

                width = values
                    .get(0)
                    .ok_or("Could not get width.")
                    .and_then(|value| {
                        value
                            .parse::<usize>()
                            .map_err(|_| "Could not parse width as usize.")
                    })?;

                height = values
                    .get(1)
                    .ok_or("Could not get height")
                    .and_then(|value| {
                        value
                            .parse::<usize>()
                            .map_err(|_| "Cound not parse height as usize.")
                    })?;

                continue;
            }

            if line.starts_with("Wall") {
                let mut temp = line.split_whitespace();
                temp.next();
                let data = temp.next().ok_or("Expected more than one entry.")?;
                let values: Vec<&str> = data.split(",").collect();

                let x = values.get(0).ok_or("Could not get x.").and_then(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| "Could not parse x as usize.")
                })?;

                let y = values.get(1).ok_or("Could not get y").and_then(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| "Cound not parse y as usize.")
                })?;

                walls.push((x, y));
                continue;
            }

            if line.starts_with("Exit") {
                let mut temp = line.split_whitespace();
                temp.next();
                let data = temp.next().ok_or("Expected more than one entry.")?;
                let values: Vec<&str> = data.split(",").collect();

                let x = values.get(0).ok_or("Could not get x.").and_then(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| "Could not parse x as usize.")
                })?;

                let y = values.get(1).ok_or("Could not get y").and_then(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| "Cound not parse y as usize.")
                })?;

                let reward = values
                    .get(2)
                    .ok_or("Could not get reward")
                    .and_then(|value| {
                        value
                            .parse::<f32>()
                            .map_err(|_| "Cound not parse reward as f32.")
                    })?;

                exits.push((x, y, reward));
                continue;
            }
        }

        let mut world = World::new(width, height);

        for wall in walls {
            world.add_wall(wall.0, wall.1);
        }
        for exit in exits {
            world.add_exit(exit.0, exit.1, exit.2);
        }

        Ok(world)
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
            return target.is_some();
        }

        false
    }

    pub fn transition(
        &self,
        state: &State,
        action: Action,
        noise: f32,
    ) -> Option<Vec<(f32, Action)>> {
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
                result.push((noise, Action::Move(moves[0])));
                let remainder = (1.0 - noise) / (moves.len() as f32 - 1.0);
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

    fn value(
        &self,
        state: &State,
        action: Action,
        discount: f32,
        noise: f32,
        values: &Vec<f32>,
    ) -> f32 {
        return match action {
            Action::Exit => self.reward(&state, action),
            Action::Move(_) => {
                let actions = self.transition(&state, action, noise);
                let mut accumulation = 0.0;
                if let Some(actions) = actions {
                    for entry in actions {
                        if let Action::Move(direction) = entry.1 {
                            let target = self.move_to(&state, direction);
                            accumulation += entry.0
                                * (self.reward(&state, entry.1)
                                    + discount * values[target.y * self.width + target.x]);
                        }
                    }
                }
                accumulation
            }
            Action::None => 0.0,
        };
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

    pub fn generate_policy(&self, q_values: &Vec<[f32; 4]>) -> Vec<Action> {
        let mut policy = vec![Action::None; self.area()];
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                if !self.valid_position(&state) {
                    policy[index] = Action::None;
                    continue;
                }

                if self.can_exit(&state) {
                    policy[index] = Action::Exit;
                    continue;
                }

                let mut target = 0;
                for i in 1..q_values[index].len() {
                    if q_values[index][i] > q_values[index][target] {
                        target = i;
                    }
                }

                policy[index] = Action::Move(DIRECTIONS[target]);
            }
        }

        policy
    }

    pub fn generate_random_policy(&self) -> Vec<Action> {
        // Create a valid random policy.
        let mut policy = Vec::with_capacity(self.area());
        for y in 0..self.height {
            for x in 0..self.width {
                let state = State::new(x, y);
                if !self.valid_position(&state) {
                    policy.push(Action::None);
                    continue;
                }
                if self.can_exit(&state) {
                    policy.push(Action::Exit);
                    continue;
                }

                // TODO: this works, but what would happen if the policy was truly random?
                policy.push(Action::Move(Direction::Up));
            }
        }

        policy
    }

    pub fn value_bellman_update(
        &self,
        discount: f32,
        noise: f32,
        values: &Vec<f32>,
        q_values: &mut Vec<[f32; 4]>,
    ) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                // If we happen to be in an invalid position then move along!
                if !self.valid_position(&state) {
                    result[index] = 0.0;
                    continue;
                }

                // If we can exit then we must exit.
                if self.can_exit(&state) {
                    result[index] = self.reward(&state, Action::Exit);
                    continue;
                }

                // In order to find the optimal policy we must recursively calculate the expected value for each possible action in the
                // current state. The action with the hightest value is our final target.

                let mut new_values = [0.0; 4];

                for (i, direction) in DIRECTIONS.iter().enumerate() {
                    new_values[i] =
                        self.value(&state, Action::Move(*direction), discount, noise, values)
                }

                // Find the highest value.
                let mut max = new_values[0];
                for i in 1..new_values.len() {
                    if new_values[i] > max {
                        max = new_values[i]
                    }
                }

                q_values[index] = new_values;
                result[index] = max;
            }
        }

        result
    }

    pub fn value_iteration(&mut self, discount: f32, noise: f32, epsilon: f32) -> Vec<Action> {
        let mut values = vec![0.0; self.area()];
        let mut q_values = vec![[0.0; 4]; self.area()];

        let mut iterations = 0;
        loop {
            // Loop until convergence.
            iterations += 1;

            let temp = self.value_bellman_update(discount, noise, &values, &mut q_values);
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

        self.generate_policy(&q_values)
    }

    pub fn policy_bellman_update(
        &mut self,
        discount: f32,
        noise: f32,
        policy: &Vec<Action>,
        values: &Vec<f32>,
    ) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                result[index] = self.value(&state, policy[index], discount, noise, values);
            }
        }

        result
    }

    pub fn policy_evaluation(
        &mut self,
        discount: f32,
        noise: f32,
        epsilon: f32,
        policy: &Vec<Action>,
        values: &Vec<f32>,
    ) -> Vec<f32> {
        let mut result = values.clone();
        let mut iterations = 0;
        // Loop until convergence.
        loop {
            iterations += 1;
            let temp = self.policy_bellman_update(discount, noise, &policy, &result);
            let deltas = temp.iter().enumerate().map(|(i, v)| *v - result[i]);

            let mut max_delta = f32::MIN;
            for delta in deltas {
                if delta > max_delta {
                    max_delta = delta;
                }
            }

            result = temp;

            if max_delta.abs() < epsilon && iterations > 1 {
                return result;
            }
        }
    }

    pub fn policy_improvement(
        &self,
        discount: f32,
        noise: f32,
        policy: &Vec<Action>,
        values: &Vec<f32>,
        q_values: &mut Vec<[f32; 4]>,
    ) -> (Vec<Action>, bool) {
        let mut result = vec![Action::None; policy.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                result[index] = match policy[index] {
                    Action::Exit | Action::None => policy[index],
                    Action::Move(_) => {
                        // TODO: This should be extracted into a function some how! Value iteration basically uses the same exact thing!
                        let mut new_values = [0.0; 4];

                        for (i, direction) in DIRECTIONS.iter().enumerate() {
                            new_values[i] = self.value(
                                &state,
                                Action::Move(*direction),
                                discount,
                                noise,
                                values,
                            );
                        }

                        let mut optimal = 0;
                        for i in 1..new_values.len() {
                            if new_values[i] > new_values[optimal] {
                                optimal = i;
                            }
                        }

                        q_values[index] = new_values;

                        Action::Move(DIRECTIONS[optimal])
                    }
                };
            }
        }

        let mut stable = false;

        for i in 0..policy.len() {
            if !(result[i] == policy[i]) {
                // The policy is not stable; another pass of policy iteration -- using the new
                // policy -- is required.
                break;
            }

            if i == policy.len() - 1 {
                // The policy is stable; policy iteration is complete.
                stable = true;
            }
        }

        (result, stable)
    }

    pub fn policy_iteration(&mut self, discount: f32, noise: f32, epsilon: f32) -> Vec<Action> {
        let mut policy = self.generate_random_policy();
        let mut values = vec![0.0; self.area()];
        let mut q_values = vec![[0.0; 4]; self.area()];

        loop {
            values = self.policy_evaluation(discount, noise, epsilon, &policy, &values);
            let (temp, stable) =
                self.policy_improvement(discount, noise, &policy, &values, &mut q_values);
            policy = temp;

            if stable {
                break;
            }
        }

        policy
    }
}

pub struct Analysis {
    pub policy: Vec<Action>,
    pub values: Vec<f32>,
    pub q_values: Vec<[f32; 4]>,
    pub min_value: f32,
    pub max_value: f32,
}

impl Analysis {
    pub fn min(values: &Vec<f32>) -> f32 {
        let mut min_value = f32::MAX;
        for value in values.iter() {
            if *value < min_value {
                min_value = *value;
            }
        }

        min_value
    }

    pub fn max(values: &Vec<f32>) -> f32 {
        let mut max_value = f32::MIN;
        for value in values.iter() {
            if *value > max_value {
                max_value = *value;
            }
        }

        max_value
    }
}
