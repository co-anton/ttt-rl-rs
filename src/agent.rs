use rand::prelude::*;
use std::{collections::HashMap, usize};

use crate::logic::{Board, CellState};

struct Reward;

impl Reward {
    pub const WIN: i32 = 2;
    pub const LOSS: i32 = -1;
    pub const DRAW: i32 = 1;
    pub const INTERMEDIATE: i32 = 0; // Repeated values are fine in this context
}

#[derive(Hash, Eq, Clone, PartialEq, Copy)]
struct Action {
    x_axis: usize,
    y_axis: usize,
}

type State = Vec<Vec<CellState>>;

#[derive(Hash, Eq, PartialEq, Clone)]
struct StateAction {
    state: State,
    action: Action,
}

pub struct QTable {
    table: HashMap<StateAction, f64>,
    alpha: f64,
    gamma: f64,
    epsilon: f64,
}

impl QTable {
    fn new(alpha: f64, gamma: f64, epsilon: f64) -> Self {
        Self {
            table: HashMap::new(),
            alpha,
            gamma,
            epsilon,
        }
    }

    fn get_q(&self, state: &State, action: Action) -> f64 {
        *self
            .table
            .get(&StateAction {
                state: state.clone(),
                action: action.clone(),
            })
            .unwrap_or(&0.0)
    }

    fn update_table(
        &mut self,
        state: &State,
        action: Action,
        state_after_action: &State,
        possible_actions: &Vec<Action>,
        reward: i32,
    ) {
        let current_q = self.get_q(state, action.clone());
        let max_q = possible_actions
            .iter()
            .map(|next_action| self.get_q(state_after_action, next_action.clone()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let new_q = current_q + self.alpha * (reward as f64 + self.gamma * max_q - current_q);
        self.table.insert(
            StateAction {
                state: state.clone(),
                action: action.clone(),
            },
            new_q,
        );
    }

    fn epsilon_greedy_search(&self, state: &State, possible_actions: &Vec<Action>) -> Action {
        if rand::thread_rng().gen::<f64>() < self.epsilon {
            *possible_actions.choose(&mut rand::thread_rng()).unwrap()
        } else {
            *possible_actions
                .iter()
                .max_by(|&a, &b| {
                    self.get_q(state, a.clone())
                        .partial_cmp(&self.get_q(state, b.clone()))
                        .unwrap()
                })
                .unwrap()
        }
    }
}

struct Environment {
    board: Board,
    reward: i32,
    player: CellState,
    actions: Vec<Action>,
}

impl Environment {
    fn new(size: usize, win_condition: usize) -> Self {
        let mut rng_thread = rand::thread_rng();
        let player = if rng_thread.gen::<f64>() > 0.5 {
            CellState::X
        } else {
            CellState::O
        };
        let actions: Vec<Action> = Vec::new();

        Self {
            board: Board::new(size, win_condition),
            reward: Reward::INTERMEDIATE,
            player,
            actions,
        }
    }

    fn reset(&mut self) {
        self.board.reset();
        self.reward = Reward::INTERMEDIATE;
        self.actions = Vec::new();
    }

    fn step(&mut self, action: Action) -> (State, i32) {
        self.board.play_move(action.x_axis, action.y_axis);
        self.actions.push(action);

        if let Some(winner) = self.board.is_winner() {
            self.reward = if winner == self.player {
                Reward::WIN
            } else {
                Reward::LOSS
            }
        };

        if self.board.is_board_full() && self.reward == Reward::INTERMEDIATE {
            self.reward = Reward::DRAW;
        }
        (self.get_grid(), self.reward)
    }

    fn get_possibe_moves(&self) -> Vec<Action> {
        let grid = self.board.get_possible_actions();
        grid.iter()
            .map(|&(x_axis, y_axis)| Action { x_axis, y_axis })
            .collect::<Vec<Action>>()
    }

    fn get_grid(&self) -> State {
        self.board.get_grid()
    }
}

fn get_hyperparameters(epoch: usize, n_epoch: usize) -> (f64, f64, f64) {
    let grid_alpha = [0.9, 0.6, 0.3, 0.2, 0.1];
    let grid_epsilon = [0.5, 0.3, 0.1, 0.01];
    let grid_gamma = [0.9, 0.95, 0.99];
    let progress = epoch as f64 / n_epoch as f64;

    let interpolate = |grid: &[f64]| {
        let idx = ((grid.len() - 1) as f64 * progress).floor() as usize;
        let frac = (grid.len() - 1) as f64 * progress - idx as f64;
        grid[idx] * (1.0 - frac) + grid[idx + 1] * frac
    };

    let alpha = interpolate(&grid_alpha);
    let epsilon = interpolate(&grid_epsilon);
    let gamma = interpolate(&grid_gamma);

    (alpha, gamma, epsilon)
}

pub fn train(n_games: usize, n_epoch: usize) -> QTable {
    let (mut alpha, mut gamma, mut epsilon) = get_hyperparameters(0, n_epoch);
    let mut agent = QTable::new(alpha, gamma, epsilon);
    let mut env = Environment::new(3, 3);
    for epoch in 0..n_epoch {
        let mut n_wins = 0;
        let mut n_draws = 0;
        for _game in 0..n_games {
            env.reset();
            let mut state = env.get_grid();
            let mut possible_actions = env.get_possibe_moves();

            loop {
                let action = agent.epsilon_greedy_search(&state, &possible_actions);
                let (next_state, reward) = env.step(action);
                possible_actions = env.get_possibe_moves();
                agent.update_table(&state, action, &next_state, &possible_actions, reward);
                state = next_state;
                if reward != Reward::INTERMEDIATE {
                    if reward == Reward::WIN {
                        n_wins += 1
                    } else if reward == Reward::DRAW {
                        n_draws += 1
                    };
                    break;
                }
            }
        }
        (alpha, gamma, epsilon) = get_hyperparameters(epoch, n_epoch);
        agent.alpha = alpha;
        agent.gamma = gamma;
        agent.epsilon = epsilon;
        println!(
            "Epoch: {}, win rate: {}, draw rate {}, loss rate {}, hyper params {:?}",
            epoch,
            n_wins as f64 / n_games as f64,
            n_draws as f64 / n_games as f64,
            (n_games - n_wins - n_draws) as f64 / n_games as f64,
            (alpha, gamma, epsilon)
        );
    }
    agent
}
