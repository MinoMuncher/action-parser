use crate::replay_response::{Board, MinoType};
use std::{collections::VecDeque, fmt::Display};

use crate::attack::get_indexed_attack;
use bitris::prelude::*;

///parse replay response types into a bitris node and queue
fn parse_replay_args(
    board: &Board,
    btb: usize,
    combo: usize,
    queue: &[MinoType],
) -> (Node, VecDeque<Shape>) {
    let mut board64 = Board64::blank();
    for y in 0..40 {
        for x in 0..10 {
            if board[(39 - y) * 10 + x] != MinoType::Empty {
                board64.set_at(Location {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    let mut vec_queue = VecDeque::new();
    for &p in queue.iter().take(8) {
        use Shape::*;
        vec_queue.push_back(match p {
            MinoType::Z => Z,
            MinoType::L => L,
            MinoType::O => O,
            MinoType::S => S,
            MinoType::I => I,
            MinoType::J => J,
            MinoType::T => T,
            _ => continue,
        })
    }
    let hold = vec_queue.pop_front().unwrap();
    let node = Node {
        board: board64,
        hold,
        btb,
        combo,
        attack: 0,
    };
    (node, vec_queue)
}

///dfs to get atk and def
pub fn solve_state(board: &Board, btb: usize, combo: usize, queue: &[MinoType]) -> (usize, usize) {
    let (node, mut queue) = parse_replay_args(board, btb, combo, queue);
    dfs(node, &mut queue)
}

//we do tspin check with immobile, hopefully it is sufficient
fn is_immobile(board: &Board64, placement: &BlPlacement) -> bool {
    let north = placement + Offset { dx: 0, dy: 1 };
    if north.is_in_free_space(board) {
        return false;
    }

    let south = placement + Offset { dx: 0, dy: -1 };
    if south.is_in_free_space(board) {
        return false;
    }

    let east = placement + Offset { dx: 1, dy: 0 };
    if east.is_in_free_space(board) {
        return false;
    }

    let west = placement + Offset { dx: -1, dy: 0 };
    if west.is_in_free_space(board) {
        return false;
    }

    true
}

#[derive(Clone)]
struct Node {
    board: Board64,
    hold: Shape,
    btb: usize,
    combo: usize,
    attack: usize,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} hold {} btb {} combo {} attack {}",
            self.board, self.hold, self.btb, self.combo, self.attack
        ))?;
        Ok(())
    }
}

impl Node {
    fn get_fall_height(&self, shape: Shape) -> usize {
        let mut defence = 0;
        let mut spawn = Piece::new(shape, Orientation::North)
            .with(cc(4, 21))
            .to_bl_placement();
        while spawn.is_in_free_space(&self.board) {
            defence += 1;
            spawn += Offset { dx: 0, dy: -1 };
        }
        defence
    }
    fn get_children(&self, shape: Shape, next_hold: Shape) -> Vec<Self> {
        let spawn = Piece::new(shape, Orientation::North)
            .with(cc(4, 21))
            .to_bl_placement();
        if !spawn.is_in_free_space(&self.board) {
            return Vec::new();
        }

        let move_rules = MoveRules::default();

        let minimized_moves = move_rules.generate_minimized_moves(self.board, spawn);

        minimized_moves
            .into_iter()
            .filter_map(|placement| {
                let mut new_node = self.clone();
                let lines_cleared = placement
                    .place_on_and_clear_lines(&mut new_node.board)
                    .unwrap_or(Lines::blank())
                    .count();
                if lines_cleared > 0 {
                    let mut clear_type = lines_cleared as usize;
                    let mut is_btb = false;
                    if shape == Shape::T && is_immobile(&self.board, &placement) {
                        if lines_cleared == 1 {
                            clear_type = 5;
                        } else if lines_cleared == 2 {
                            clear_type = 7;
                        } else if lines_cleared == 3 {
                            clear_type = 8;
                        }
                        is_btb = true;
                    }
                    if lines_cleared == 4 {
                        is_btb = true;
                    }
                    if is_btb {
                        new_node.btb += 1;
                    } else {
                        new_node.btb = 0;
                    }
                    let mut atk = get_indexed_attack(clear_type, self.combo, self.btb);
                    if new_node.board.is_empty() {
                        atk += 10;
                    }
                    new_node.attack += atk;

                    new_node.hold = next_hold;
                    Some(new_node)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn dfs(node: Node, queue: &mut VecDeque<Shape>) -> (usize, usize) {
    if queue.is_empty() {
        return (
            node.attack,
            node.attack + node.get_fall_height(Shape::I) + 1,
        );
    }
    let use_shape = queue.pop_front().unwrap();

    let mut max_attack = 0;
    let mut max_def = 0;

    let children: Vec<_> = node.get_children(use_shape, node.hold);
    if children.is_empty() {
        max_attack = max_attack.max(node.attack);
        let height = node.get_fall_height(*queue.front().unwrap_or(&node.hold));
        max_def = max_def.max(node.attack + height + 1);
    } else {
        for child in children {
            let (atk, def) = dfs(child, queue);
            max_attack = max_attack.max(atk);
            max_def = max_def.max(def)
        }
    }

    if use_shape != node.hold {
        let children: Vec<_> = node.get_children(node.hold, use_shape);
        if children.is_empty() {
            max_attack = max_attack.max(node.attack);
            let height = node.get_fall_height(*queue.front().unwrap_or(&node.hold));
            max_def = max_def.max(node.attack + height + 1);
        } else {
            for child in children {
                let (atk, def) = dfs(child, queue);
                max_attack = max_attack.max(atk);
                max_def = max_def.max(def)
            }
        }
    }
    queue.push_front(use_shape);
    (max_attack, max_def)
}
