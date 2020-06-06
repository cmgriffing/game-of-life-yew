use crate::app::core::game::{Cellule, LifeState};

pub fn seed_pentadecathlon(cellule_width: i32, cellule_height: i32) -> Vec<Cellule> {
  let middle_row_number: i32 = cellule_height / 2;

  let middle_column_number: i32 = cellule_width / 2;

  let mut cellules: Vec<Cellule> = Vec::new();

  for row_index in 0..cellule_height {
    for column_index in 0..cellule_width {
      if (middle_row_number <= row_index + 1 && middle_row_number >= row_index - 1)
        && (column_index < middle_column_number + 4 && column_index > middle_column_number - 5)
      {
        if middle_row_number == row_index
          && (middle_column_number == column_index + 3 || middle_column_number == column_index - 2)
        {
          cellules.push(Cellule {
            life_state: LifeState::Dead,
          })
        } else {
          cellules.push(Cellule {
            life_state: LifeState::Alive,
          })
        }
      } else {
        cellules.push(Cellule {
          life_state: LifeState::Dead,
        })
      }
    }
  }

  return cellules;
}

pub fn seed_middle_line_starter(cellule_width: i32, cellule_height: i32) -> Vec<Cellule> {
  let middle_row_number: i32 = cellule_height / 2;

  let middle_column_number: i32 = cellule_width / 2;

  let mut cellules: Vec<Cellule> = Vec::new();

  for row_index in 0..cellule_height {
    for column_index in 0..cellule_width {
      if (middle_row_number == row_index)
        && (column_index < middle_column_number + 7 && column_index > middle_column_number - 7)
      {
        cellules.push(Cellule {
          life_state: LifeState::Alive,
        })
      } else {
        cellules.push(Cellule {
          life_state: LifeState::Dead,
        })
      }
    }
  }

  return cellules;
}
