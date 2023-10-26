/// Representation of the state of a cell
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Empty,
    X,
    O,
}

/// Representation of a board of arbitrary size  
#[derive(Debug)]
pub struct Board {
    grid: Vec<Vec<CellState>>,
    turn: CellState,
    size: usize,
    win_condition_length: usize,
    /// first vector is for main diagonal and anti diagonal, then it's a vector of DiagonalCoord
    diagonals_coords: Vec<Vec<DiagonalCoord>>,
}

type DiagonalCoord = Vec<(usize, usize)>;

fn generate_coords(
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: fn() -> (isize, isize),
) -> DiagonalCoord {
    (0..length)
        .filter_map(|i| {
            let (x_delta, y_delta) = direction();
            let coords_isize = (
                start_row as isize + i as isize * x_delta,
                start_col as isize + i as isize * y_delta,
            );
            if coords_isize.0 >= 0 && coords_isize.1 >= 0 {
                Some((coords_isize.0 as usize, coords_isize.1 as usize))
            } else {
                None
            }
        })
        .collect()
}

fn calculate_diagonals_coords(size: usize, minimal_length: usize) -> Vec<Vec<DiagonalCoord>> {
    let mut all_diagonals = Vec::new();
    let mut main_diagonals = Vec::new();
    let mut anti_diagonals = Vec::new();

    let direction_main = || (1, 1);
    let direction_anti = || (1, -1);

    let valid_indices = |start, end| (start..end).filter(move |&i| end - i >= minimal_length);

    for start in valid_indices(0, size) {
        for length in (minimal_length..=size - start).rev() {
            main_diagonals.push(generate_coords(start, 0, length, direction_main));
            anti_diagonals.push(generate_coords(start, size - 1, length, direction_anti));
            if start > 0 {
                main_diagonals.push(generate_coords(0, start, length, direction_main));
                anti_diagonals.push(generate_coords(0, start, length, direction_anti));
            }
        }
    }

    all_diagonals.push(main_diagonals);
    all_diagonals.push(anti_diagonals);

    all_diagonals
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO parametrize the tests
    #[test]
    fn test_diagonal_2_2() {
        let diagonals = calculate_diagonals_coords(2, 2);
        assert_eq!(diagonals[0].len(), 1);
        assert_eq!(diagonals[1].len(), 1);
        println!("{:?}", diagonals);
    }

    #[test]
    fn test_diagonal_3_3() {
        let diagonals = calculate_diagonals_coords(3, 3);
        assert_eq!(diagonals[0].len(), 1);
        assert_eq!(diagonals[1].len(), 1);
        println!("{:?}", diagonals);
    }

    #[test]
    fn test_diagonal_3_2() {
        let diagonals = calculate_diagonals_coords(3, 2);
        assert_eq!(diagonals[0].len(), 4);
        assert_eq!(diagonals[1].len(), 4);
        println!("{:?}", diagonals);
    }

    #[test]
    fn test_diagonal_10_2() {
        let diagonals = calculate_diagonals_coords(10, 2);
        assert_eq!(diagonals[0].len(), 81);
        assert_eq!(diagonals[1].len(), 81);
        // Note: the theoretical number here should be 45 for each, the reason it's 45 is because
        // we are double counting some diagonals. For example, the diagonal that goes along the i_i
        // line is counted several times because you would have [(0,0), (1,1)], [(0, 0), (1, 1),
        // (2, 2)], etc. TODO fix that
        println!("{:?}", diagonals);
    }
}

impl Board {
    /// Creates a new board
    pub fn new(size: usize, win_condition_length: usize) -> Self {
        Self {
            grid: vec![vec![CellState::Empty; size]; size],
            turn: CellState::X,
            size,
            win_condition_length,
            diagonals_coords: calculate_diagonals_coords(size, win_condition_length),
        }
    }

    /// Reset player turn and grid
    pub fn reset(&mut self) {
        self.grid = vec![vec![CellState::Empty; self.size]; self.size];
        self.turn = CellState::X;
    }

    /// Play move at position x_axis, y_axis
    pub fn play_move(&mut self, x_axis: usize, y_axis: usize) {
        println!(
            "Player {:?} attempting to play at position ({},{})",
            self.turn, x_axis, y_axis
        );
        if self.is_valid_move(x_axis, y_axis) {
            self.grid[x_axis][y_axis] = self.turn;
            println!(
                "New value at [{}, {}]: {:?}",
                x_axis, y_axis, self.grid[x_axis][y_axis]
            );
            self.next_turn();
        };
    }

    /// Checks if move is valid
    pub fn is_valid_move(&self, x_axis: usize, y_axis: usize) -> bool {
        (x_axis < self.size && y_axis < self.size) && self.grid[x_axis][y_axis] == CellState::Empty
    }

    /// Returns current player
    pub fn get_current_player(&self) -> CellState {
        self.turn
    }

    /// Update player turn
    fn next_turn(&mut self) {
        self.turn = if self.turn == CellState::X {
            CellState::O
        } else {
            CellState::X
        };
    }

    /// Returns whether the board is full, an alternative approach could be to count the number of
    /// plays rather than iterate over the board every time (TODO compare)
    pub fn is_board_full(&self) -> bool {
        for row in self.grid.iter() {
            for cell in row.iter() {
                if *cell == CellState::Empty {
                    return false;
                }
            }
        }
        true
    }

    /// Returns the winner of the game if there's any
    pub fn is_winner(&self) -> Option<CellState> {
        println!("Checking for winner");
        println!("Grid: {:?}", self.grid);

        for sequence in self.grid.iter() {
            println!("sequence: {:?}", sequence);
            if let Some(winner) = self.find_winner(sequence) {
                return Some(winner);
            }
        }

        for index in 0..self.size {
            let sequence: Vec<CellState> = self.grid.iter().map(|seq| seq[index]).collect();
            println!("sequence: {:?}", sequence);
            if let Some(winner) = self.find_winner(&sequence) {
                return Some(winner);
            }
        }

        for diagonals_coords in self.diagonals_coords.iter() {
            for diagonal_coords in diagonals_coords.iter() {
                let diagonal: Vec<CellState> = diagonal_coords
                    .iter()
                    .map(|&(x_axis, y_axis)| self.grid[x_axis][y_axis])
                    .collect();
                println!("Diagonal: {:?}", diagonal);
                if let Some(winner) = self.find_winner(&diagonal) {
                    return Some(winner);
                }
            }
        }

        // No winners found
        None
    }

    /// Given a sequence, returns the winner if there's any
    fn find_winner(&self, sequence: &[CellState]) -> Option<CellState> {
        let mut count_consecutive = 0;
        let mut previous_cell = CellState::Empty;
        for cell in sequence.iter() {
            if *cell == previous_cell && *cell != CellState::Empty {
                count_consecutive += 1;
                println!("{}", count_consecutive);
                if count_consecutive == self.win_condition_length - 1 {
                    return Some(*cell);
                }
            } else {
                count_consecutive = 0;
                previous_cell = *cell;
            }
        }
        None
    }
}
