use crate::replay_response::{Board, MinoType};

pub fn get_height(board: &Board) -> usize {
    for y in 0..40 {
        for x in 0..10 {
            if board[x + y * 10] != MinoType::Empty {
                return 40 - y;
            }
        }
    }
    0
}

pub fn get_garbage_height(board: &Board) -> usize {
    for y in (0..40).rev() {
        let mut garbage_found = false;
        for x in 0..10 {
            if board[x + y * 10] == MinoType::Garbage {
                garbage_found = true;
                break;
            }
        }
        if !garbage_found {
            return 39 - y;
        }
    }
    0
}

pub fn get_well(board: &Board) -> (usize, usize) {
    let column_heights: Vec<usize> = (0..10_usize)
        .map(|x| {
            for y in 0..40 {
                if board[x + y * 10] != MinoType::Empty {
                    return 40 - y;
                }
            }
            0
        })
        .collect();
    let min_height = column_heights.into_iter().enumerate().min_by(|(_, a), (_, b)| a.cmp(b)).expect("column_heights empty");
    min_height
}
///Checks if the top layer of garbage on the board is cheese or not
pub fn has_cheese(board: &Board) -> bool {
    let mut last_garbage_col = 10;
    let mut count = 0;
    for y in (0..40).rev() {
        let mut garbage_found = false;
        let mut new_col = 10;
        for x in 0..10 {
            if board[x + y * 10] == MinoType::Garbage {
                garbage_found = true;
            } else if board[x + y * 10] == MinoType::Empty {
                new_col = x;
            }
        }
        if !garbage_found {
            break;
        }
        if last_garbage_col == new_col {
            count += 1;
        } else {
            count = 1;
        }
        last_garbage_col = new_col;
    }
    if count > 0 && count < 4 {
        return true;
    }
    false
}
