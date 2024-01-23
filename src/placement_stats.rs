use serde::Serialize;

use crate::board_analyzer::{get_garbage_height, get_height, get_well, has_cheese};
use crate::replay_response::{ClearType, MinoType, PlacementStats};
use crate::solver::solve_state;
///stats that represents the sum total of the data from several sequences of placements
#[derive(Debug, Default, Clone, Serialize)]
pub struct CumulativePlacementStats {
    pub clear_types: [usize; 16],
    pub shape_types: [usize; 9],
    pub garbage_cleared: usize,
    pub lines_cleared: usize,
    pub attack: usize,
    pub attack_with_garbage: usize,
    pub exclusive_garbage_cleared: usize,
    pub attack_with_stack: usize,
    pub exclusive_stack_cleared: usize,
    pub attack_with_cheese: usize,
    pub exclusive_cheese_cleared: usize,
    pub delays: Vec<f64>,
    pub stack_heights: Vec<usize>,
    pub garbage_heights: Vec<usize>,
    pub btb_segments: Vec<BTBSegment>,
    pub combo_segments: Vec<ComboSegment>,
    pub keypresses: usize,
    pub opener_attack: usize,
    pub opener_frames: f64,
    pub opener_blocks: usize,
    pub defense_potentials: Vec<usize>,
    pub blockfish_scores: Vec<usize>,
    pub spikable_boards: usize,
    pub pre_spike_boards: usize,
}

impl CumulativePlacementStats {
    fn add_stats(&mut self, stats: &CumulativePlacementStats) {
        self.clear_types
            .iter_mut()
            .zip(stats.clear_types.iter())
            .for_each(|(c, s)| *c += s);
        self.shape_types
            .iter_mut()
            .zip(stats.shape_types.iter())
            .for_each(|(c, s)| *c += s);
        self.garbage_cleared += stats.garbage_cleared;
        self.lines_cleared += stats.lines_cleared;
        self.attack += stats.attack;
        self.attack_with_garbage += stats.attack_with_garbage;
        self.exclusive_garbage_cleared += stats.exclusive_garbage_cleared;
        self.attack_with_stack += stats.attack_with_stack;
        self.exclusive_stack_cleared += stats.exclusive_stack_cleared;
        self.attack_with_cheese += stats.attack_with_cheese;
        self.exclusive_cheese_cleared += stats.exclusive_cheese_cleared;

        self.keypresses += stats.keypresses;
        self.opener_attack += stats.opener_attack;
        self.opener_frames += stats.opener_frames;
        self.opener_blocks += stats.opener_blocks;

        self.spikable_boards += stats.spikable_boards;
        self.pre_spike_boards += stats.pre_spike_boards;
    }
    ///combine stats while consuming the other
    pub fn absorb(&mut self, stats: CumulativePlacementStats) {
        self.add_stats(&stats);

        self.delays.extend(stats.delays);
        self.stack_heights.extend(stats.stack_heights);
        self.garbage_heights.extend(stats.garbage_heights);
        self.btb_segments.extend(stats.btb_segments);
        self.combo_segments.extend(stats.combo_segments);

        self.spikable_boards += stats.spikable_boards;
        self.pre_spike_boards += stats.pre_spike_boards;
    }
    ///combine stats with a reference and cloning
    #[allow(dead_code)]
    pub fn absorb_ref(&mut self, stats: &CumulativePlacementStats) {
        self.add_stats(&stats);

        self.delays.extend(stats.delays.clone());
        self.stack_heights.extend(stats.stack_heights.clone());
        self.garbage_heights.extend(stats.garbage_heights.clone());
        self.btb_segments.extend(stats.btb_segments.clone());
        self.combo_segments.extend(stats.combo_segments.clone());

        self.defense_potentials
            .extend(stats.defense_potentials.clone());
        self.blockfish_scores.extend(stats.blockfish_scores.clone());
    }
}

impl From<&[PlacementStats]> for CumulativePlacementStats {
    fn from(game: &[PlacementStats]) -> Self {
        let blockfish_config = blockfish::Config {
            search_limit: 100,
            parameters: blockfish::Parameters::default(),
        };
        let mut blockfish = blockfish::ai::AI::new(blockfish_config);

        let mut stats = CumulativePlacementStats::default();
        let mut opener_over = false;

        let mut current_combo = None;
        let mut current_btb = None;

        let mut spike_grace_period = 0;

        for (i, placement) in game.iter().enumerate() {
            if !opener_over
                && placement.garbage_cleared > 0
                && ((placement.shape == MinoType::T && !placement.btb_clear)
                    || (placement.shape != MinoType::T && placement.lines_cleared < 4))
            {
                opener_over = true;
                //log opener over if we skim clear a garbage line
            }

            stats.shape_types[placement.shape as usize] += 1;
            let height = get_height(&placement.board);

            if height == 0 {
                stats.clear_types[ClearType::PerfectClear as usize] += 1;
            } else {
                stats.clear_types[placement.clear_type as usize] += 1;
            }
            //override clear type if it is a PC

            stats.garbage_cleared += placement.garbage_cleared;
            stats.lines_cleared += placement.lines_cleared;

            let attack = placement.attack.iter().sum::<usize>();
            stats.attack += attack;

            if !opener_over {
                stats.opener_blocks += 1;
                stats.opener_attack += attack;
                stats.opener_frames += placement.frame_delay;
            }

            if placement.garbage_cleared > 0 {
                stats.attack_with_garbage += attack;
                stats.exclusive_garbage_cleared += placement.lines_cleared;
            } else if placement.lines_cleared > 0 {
                stats.attack_with_stack += attack;
                stats.exclusive_stack_cleared += placement.lines_cleared
            }

            let just_ate_cheese =
                i != 0 && placement.garbage_cleared > 0 && has_cheese(&game[i - 1].board);
            if just_ate_cheese {
                stats.attack_with_cheese += attack;
                stats.exclusive_cheese_cleared += placement.lines_cleared;
            }

            stats.delays.push(placement.frame_delay);
            stats.keypresses += placement.keypresses;

            let garbage_height = get_garbage_height(&placement.board);

            stats.stack_heights.push(height - garbage_height);
            stats.garbage_heights.push(garbage_height);

            if placement.lines_cleared > 0 {
                current_combo = match current_combo {
                    None => Some(ComboSegment::new(
                        attack,
                        placement.clear_type.is_multipliable(),
                        placement.frame_delay,
                        if i > 0 {
                            game.get(i - 1).and_then(|p| Some(p.frame_delay))
                        } else {
                            None
                        },
                    )),
                    Some(mut current_combo) => {
                        current_combo.frames += placement.frame_delay;
                        current_combo.attack += attack;
                        current_combo.blocks += 1;
                        Some(current_combo)
                    }
                }
            } else {
                if let Some(combo) = current_combo {
                    if combo.attack >= 9 {
                        stats.pre_spike_boards =
                            stats.pre_spike_boards.saturating_sub(combo.blocks);
                        spike_grace_period += 14; //if we just did a spike, we have 14 blocks of a grace period before we start penalizing not having a spike
                    }
                    stats.combo_segments.push(combo);
                    current_combo = None;
                }
            }

            if placement.lines_cleared > 0 && !placement.clear_type.is_btb_clear() {
                if let Some(btb) = current_btb {
                    stats.btb_segments.push(btb);
                    current_btb = None;
                }
            } else {
                current_btb = match current_btb {
                    None => {
                        if placement.lines_cleared > 0 {
                            let mut well = None;
                            if height > 0 {
                                well = Some(get_well(&placement.board));
                            }
                            Some(BTBSegment::new(attack, placement.shape, well))
                        } else {
                            None
                        }
                    }
                    Some(mut current_btb) => {
                        current_btb.frames += placement.frame_delay;
                        current_btb.attack += attack;

                        if placement.clear_type.is_btb_clear() {
                            current_btb.btb += 1;
                        } else {
                            if placement.shape == MinoType::I {
                                current_btb.wasted_i += 1;
                            } else if placement.shape == MinoType::T {
                                current_btb.wasted_t += 1;
                            }
                        }

                        if placement.shape == MinoType::I {
                            current_btb.i_placed += 1;
                        } else if placement.shape == MinoType::T {
                            current_btb.t_placed += 1;
                        }

                        current_btb.blocks += 1;

                        let mut well = None;
                        if height > 0 {
                            well = Some(get_well(&placement.board));
                        }
                        if current_btb.well != well {
                            current_btb.wellshifts += 1;
                        }
                        current_btb.well = well;
                        Some(current_btb)
                    }
                }
            }

            let (atk, def) = solve_state(
                &placement.board,
                placement.btb_chain,
                placement.combo,
                &placement.queue,
            );

            if atk >= 9 {
                //spikable board limit is around 2btb clears
                stats.spikable_boards += 1;
            } else {
                let mut bf_queue: Vec<_> = placement
                    .queue
                    .iter()
                    .filter_map(|&mino| mino_to_color(mino))
                    .take(5)
                    .collect();
                let bf_hold = bf_queue.remove(0);
                let mut bf_matrix = blockfish::BasicMatrix::with_cols(10);
                for y in 0..(40 - garbage_height) {
                    for x in 0..10 {
                        if placement.board[x + y * 10] != MinoType::Empty {
                            bf_matrix.set(((39 - garbage_height - y) as u16, x as u16));
                        }
                    }
                }

                let analysis = blockfish.analyze_raw(blockfish::ai::Snapshot {
                    hold: Some(bf_hold),
                    queue: bf_queue,
                    matrix: bf_matrix,
                });
                if analysis > 0 {
                    stats.blockfish_scores.push(analysis as usize);
                }
            }

            stats.defense_potentials.push(def);

            if spike_grace_period > 0 {
                spike_grace_period -= 1;
            } else {
                stats.pre_spike_boards += 1;
            }
        }
        if let Some(current_combo) = current_combo {
            stats.combo_segments.push(current_combo);
        }
        if let Some(current_btb) = current_btb {
            stats.btb_segments.push(current_btb);
        }

        stats
    }
}
#[derive(Debug, Default, Clone, Serialize)]

pub struct BTBSegment {
    pub frames: f64,
    pub attack: usize,
    pub btb: usize,
    pub blocks: usize,
    pub wellshifts: usize,

    pub wasted_i: usize,
    pub wasted_t: usize,

    pub i_placed: usize,
    pub t_placed: usize,

    pub well: Option<usize>,
}

impl BTBSegment {
    fn new(starting_attack: usize, shape: MinoType, well: Option<usize>) -> Self {
        Self {
            frames: 0.0,
            attack: starting_attack,
            btb: 0,
            blocks: 1,
            wellshifts: 0,

            wasted_i: 0,
            wasted_t: 0,

            i_placed: (shape == MinoType::I) as usize,
            t_placed: (shape == MinoType::T) as usize,

            well,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ComboSegment {
    pub frames: f64,
    pub attack: usize,
    pub blocks: usize,
    pub multipliers: Vec<usize>,
    pub initial_delay: f64,
    pub prev_delay: Option<f64>,
}

impl ComboSegment {
    fn new(
        starting_attack: usize,
        is_multiplier: bool,
        initial_delay: f64,
        prev_delay: Option<f64>,
    ) -> Self {
        let mut multipliers = Vec::new();
        if is_multiplier {
            multipliers.push(0);
        }
        Self {
            frames: 0.0,
            attack: starting_attack,
            blocks: 1,
            multipliers,
            initial_delay,
            prev_delay,
        }
    }
}

fn mino_to_color(mino: MinoType) -> Option<blockfish::Color> {
    match mino {
        MinoType::Z => blockfish::Color::try_from('Z').ok(),
        MinoType::L => blockfish::Color::try_from('L').ok(),
        MinoType::O => blockfish::Color::try_from('O').ok(),
        MinoType::S => blockfish::Color::try_from('S').ok(),
        MinoType::I => blockfish::Color::try_from('I').ok(),
        MinoType::J => blockfish::Color::try_from('J').ok(),
        MinoType::T => blockfish::Color::try_from('T').ok(),
        _ => None,
    }
}
