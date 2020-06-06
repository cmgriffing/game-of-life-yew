use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum LifeState {
    Alive,
    Dead,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cellule {
    pub life_state: LifeState,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub active: bool,
    pub cellules: Vec<Cellule>,
    pub cellules_width: usize,
    pub cellules_height: usize,
}

impl Cellule {
    pub fn set_alive(&mut self) {
        self.life_state = LifeState::Alive;
    }

    pub fn set_dead(&mut self) {
        self.life_state = LifeState::Dead;
    }

    pub fn alive(self) -> bool {
        self.life_state == LifeState::Alive
    }

    pub fn count_alive_neighbors(neighbors: &[Cellule]) -> usize {
        neighbors.iter().filter(|n| n.alive()).count()
    }

    pub fn alone(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) < 2
    }

    pub fn overpopulated(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) > 3
    }

    pub fn can_be_revived(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) == 3
    }
}

fn wrap(coord: isize, range: isize) -> usize {
    let result = if coord < 0 {
        coord + range
    } else if coord >= range {
        coord - range
    } else {
        coord
    };
    result as usize
}

impl GameState {
    fn reset(&mut self) {
        for cellule in self.cellules.iter_mut() {
            cellule.set_dead();
        }
    }

    pub fn set_cellules(&mut self, cellules: Vec<Cellule>) {
        self.cellules = cellules;
    }

    pub fn step(&mut self) {
        let mut to_dead = Vec::new();
        let mut to_live = Vec::new();
        for row in 0..self.cellules_height {
            for col in 0..self.cellules_width {
                let neighbors = self.neighbors(row as isize, col as isize);

                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                if self.cellules[current_idx].alive() {
                    if Cellule::alone(&neighbors) || Cellule::overpopulated(&neighbors) {
                        to_dead.push(current_idx);
                    }
                } else if Cellule::can_be_revived(&neighbors) {
                    to_live.push(current_idx);
                }
            }
        }
        to_dead
            .iter()
            .for_each(|idx| self.cellules[*idx].set_dead());
        to_live
            .iter()
            .for_each(|idx| self.cellules[*idx].set_alive());
    }

    fn neighbors(&self, row: isize, col: isize) -> [Cellule; 8] {
        [
            self.cellules[self.row_col_as_idx(row + 1, col)],
            self.cellules[self.row_col_as_idx(row + 1, col + 1)],
            self.cellules[self.row_col_as_idx(row + 1, col - 1)],
            self.cellules[self.row_col_as_idx(row - 1, col)],
            self.cellules[self.row_col_as_idx(row - 1, col + 1)],
            self.cellules[self.row_col_as_idx(row - 1, col - 1)],
            self.cellules[self.row_col_as_idx(row, col - 1)],
            self.cellules[self.row_col_as_idx(row, col + 1)],
        ]
    }

    fn row_col_as_idx(&self, row: isize, col: isize) -> usize {
        let row = wrap(row, self.cellules_height as isize);
        let col = wrap(col, self.cellules_width as isize);

        row * self.cellules_width + col
    }

    pub fn toggle_cellule(&mut self, idx: usize) {
        let cellule = self.cellules.get_mut(idx).unwrap();
        if cellule.life_state == LifeState::Alive {
            cellule.life_state = LifeState::Dead
        } else {
            cellule.life_state = LifeState::Alive
        };
    }
}
