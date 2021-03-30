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

    fn value_bellman(
        &mut self,
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
                    let actions = self.transition(&state, Action::Move(*direction), noise);

                    if let Some(actions) = actions {
                        for entry in actions {
                            if let Action::Move(direction) = entry.1 {
                                let target = self.move_to(&state, direction);
                                new_values[i] += entry.0
                                    * (self.reward(&state, entry.1)
                                        + discount * values[target.y * self.width + target.x]);
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

                q_values[index] = new_values;
                result[index] = max;
            }
        }

        result
    }

    pub fn value_iteration(&mut self, discount: f32, noise: f32, epsilon: f32) -> Analysis {
        let mut values = vec![0.0; self.area()];
        let mut q_values = vec![[0.0; 4]; self.area()];

        let mut iterations = 0;
        loop {
            // Loop until convergence.
            iterations += 1;

            let temp = self.value_bellman(discount, noise, &values, &mut q_values);
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

        // Now that the values have converged, we can use said values to find the optimal policy.
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

        Analysis::new(policy, values, q_values)
    }

    fn policy_bellman(
        &mut self,
        discount: f32,
        noise: f32,
        policy: &Vec<Action>,
        values: &Vec<f32>,
        q_values: &mut Vec<[f32; 4]>,
    ) -> Vec<f32> {
        let mut result = vec![0.0; values.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                result[index] = match policy[index] {
                    Action::Exit => self.reward(&state, policy[index]),
                    Action::Move(direction) => {
                        let actions = self.transition(&state, Action::Move(direction), noise);

                        let mut value = 0.0;
                        if let Some(actions) = actions {
                            for entry in actions {
                                if let Action::Move(direction) = entry.1 {
                                    let target = self.move_to(&state, direction);
                                    value += entry.0
                                        * (self.reward(&state, entry.1)
                                            + discount * values[target.y * self.width + target.x]);
                                }
                            }
                        }

                        match direction {
                            Direction::Up => q_values[index][0] = value,
                            Direction::Right => q_values[index][1] = value,
                            Direction::Down => q_values[index][2] = value,
                            Direction::Left => q_values[index][3] = value,
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
        let mut new_policy = vec![Action::None; self.area()];

        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let state = State::new(x, y);

                new_policy[index] = match policy[index] {
                    Action::Exit => policy[index],
                    Action::Move(direction) => {
                        let mut optimal = direction;
                        let mut max = f32::MIN;

                        for direction in DIRECTIONS.iter() {
                            let temp = self.move_to(&state, *direction);
                            let value = values[temp.y * self.width + temp.x];
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

    pub fn policy_iteration(
        &mut self,
        discount: f32,
        noise: f32,
        epsilon: f32,
    ) -> Analysis {
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

        let mut values = vec![0.0; self.area()];
        let mut q_values = vec![[0.0; 4]; self.area()];

        let mut iterations = 0;
        'outer: loop {
            // Loop until convergence.
            loop {
                // Policy Evaluation:
                iterations += 1;
                let temp = self.policy_bellman(discount, noise, &policy, &values, &mut q_values);
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

        Analysis::new(policy, values, q_values)
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
    fn new(policy: Vec<Action>, values: Vec<f32>, q_values: Vec<[f32; 4]>) -> Self {
        let mut min_value = f32::MAX;
        for value in values.iter() {
            if *value < min_value {
                min_value = *value;
            }
        }

        let mut max_value = f32::MIN;
        for value in values.iter() {
            if *value > max_value {
                max_value = *value;
            }
        }

        Analysis {
            policy,
            values,
            q_values,
            min_value,
            max_value,
        }
    }
}
