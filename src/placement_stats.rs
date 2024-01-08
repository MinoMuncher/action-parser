use crate::replay_response::{PlayerPlacements, ClearType, MinoType};
use crate::board_analyzer::{get_garbage_height, get_height, get_well, has_cheese};
use crate::solver::solve_state;

#[derive(Debug, Default)]
pub struct CumulativePlacementStats{
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
    pub defense_potentials: Vec<usize>,
    pub attack_potentials: Vec<usize>,
    pub blockfish_scores: Vec<usize>
}
fn mino_to_color(mino: MinoType)->blockfish::Color{
    match mino{
        MinoType::Z => blockfish::Color::try_from('Z').unwrap(),
        MinoType::L => blockfish::Color::try_from('L').unwrap(),
        MinoType::O => blockfish::Color::try_from('O').unwrap(),
        MinoType::S => blockfish::Color::try_from('S').unwrap(),
        MinoType::I => blockfish::Color::try_from('I').unwrap(),
        MinoType::J => blockfish::Color::try_from('J').unwrap(),
        MinoType::T => blockfish::Color::try_from('T').unwrap(),
        MinoType::Garbage => blockfish::Color::try_from('G').unwrap(),
        MinoType::Empty => blockfish::Color::try_from(' ').unwrap(),
    }
}

impl From<&PlayerPlacements> for CumulativePlacementStats{

    fn from(value: &PlayerPlacements) -> Self {
        let blockfish_config = blockfish::Config{
            search_limit: 1_000,
            parameters: blockfish::Parameters::default(),
        };
        let mut blockfish = blockfish::ai::AI::new(blockfish_config);


        let mut stats = CumulativePlacementStats::default();

        for game in value{
            let mut opener_over = false;

            let mut current_combo = ComboSegment::default();
            let mut current_btb = BTBSegment::default();

            for (i, placement) in game.iter().enumerate(){
                if !opener_over && placement.garbage_cleared > 0 && !placement.btb_clear{
                    opener_over = true;
                }

                stats.shape_types[placement.shape as usize] += 1;
                let height = get_height(&placement.board);
                
                if height == 0{
                    stats.clear_types[ClearType::PerfectClear as usize] += 1;
                }else{
                    stats.clear_types[placement.clear_type as usize] += 1;

                }

                stats.garbage_cleared += placement.garbage_cleared;
                stats.lines_cleared += placement.lines_cleared;

                let attack = placement.attack.iter().sum::<usize>();
                stats.attack += attack;

                if !opener_over{
                    stats.opener_attack += attack;
                    stats.opener_frames += placement.frame_delay;
                }

                if placement.garbage_cleared > 0{
                    stats.attack_with_garbage += attack;
                    stats.exclusive_garbage_cleared += placement.garbage_cleared;
                }else if placement.lines_cleared > 0{
                    stats.attack_with_stack += attack;
                    stats.exclusive_stack_cleared += placement.lines_cleared
                }

                let just_ate_cheese = i != 0 && placement.garbage_cleared > 0 && has_cheese(&game[i-1].board);
                if just_ate_cheese{
                    stats.attack_with_cheese += attack;
                    stats.exclusive_cheese_cleared += stats.garbage_cleared;
                }

                stats.delays.push(placement.frame_delay);
                stats.keypresses+=placement.keypresses;

                let garbage_height = get_garbage_height(&placement.board);

                stats.stack_heights.push(height-garbage_height);
                stats.garbage_heights.push(garbage_height);


                if placement.clear_type != ClearType::None{
                    current_combo.frames += placement.frame_delay;
                    current_combo.attack += attack;
                    current_combo.blocks += 1;
                }else{
                    if current_combo.blocks > 1{
                        stats.combo_segments.push(current_combo);
                    }
                    current_combo = ComboSegment::new(attack, placement.clear_type.is_multipliable());
                }

                if placement.clear_type!=ClearType::None && !placement.btb_clear{
                    if current_btb.btb > 0{
                        stats.btb_segments.push(current_btb);
                    }
                    let mut well = None;
                    if height > 0{
                        well = Some(get_well(&placement.board));
                    }
                    current_btb = BTBSegment::new(attack, placement.shape, well);
                }else{
                    current_btb.frames += placement.frame_delay;
                    current_btb.attack += attack;

                    if placement.clear_type.is_multipliable(){
                        current_btb.btb += 1;
                    }else{
                        if placement.shape == MinoType::I{
                            current_btb.wasted_i +=1;
                        }else if placement.shape == MinoType::T{
                            current_btb.wasted_t += 1;
                        }
                    }

                    if placement.shape == MinoType::I{
                        current_btb.i_placed +=1;
                    }else if placement.shape == MinoType::T{
                        current_btb.t_placed +=1;
                    }

                    current_btb.blocks += 1;


                    let mut well = None;
                    if height > 0{
                        well = Some(get_well(&placement.board));
                    }
                    if current_btb.well != well{
                        current_btb.wellshifts +=1;
                    }
                    current_btb.well = well;
                }

                let (atk, def) = solve_state(&placement.board, placement.btb_chain, placement.combo, &placement.queue);

                stats.defense_potentials.push(def);
                stats.attack_potentials.push(atk);

                let mut bf_queue: Vec<_> = placement.queue.iter().map(|&mino|mino_to_color(mino)).collect();
                let bf_hold = bf_queue.remove(0);
                let mut bf_matrix = blockfish::BasicMatrix::with_cols(10);
                for y in 0..40{
                    for x in 0..10{
                        if placement.board[x + y * 10] != MinoType::Empty{
                            bf_matrix.set((x as u16, (39 - y) as u16));
                        }
                    }
                }
                let mut analysis = blockfish.analyze(blockfish::ai::Snapshot { hold: Some(bf_hold), queue: bf_queue, matrix: bf_matrix });
                analysis.wait();
                let move_id = analysis
                .all_moves()
                .min_by(|&m, &n| analysis.cmp(m, n))
                .expect("no suggestions");
                let rating = analysis.suggestion(move_id, 1).rating;
                if rating > 0{
                    stats.blockfish_scores.push(rating as usize);
                }
            }
        }


        stats
    }
}
#[derive(Debug, Default)]

pub struct BTBSegment{
    pub frames:f64,
    pub attack:usize,
    pub btb: usize,
    pub blocks: usize,
    pub wellshifts: usize,

    pub wasted_i: usize,
    pub wasted_t: usize,

    pub i_placed: usize,
    pub t_placed: usize,

    pub well: Option<usize>
}

impl BTBSegment{
    fn new(starting_attack: usize, shape: MinoType, well: Option<usize>)->Self{
        Self{
            frames: 0.0,
            attack: starting_attack,
            btb: 0,
            blocks: 1,
            wellshifts: 0,

            wasted_i: 0,
            wasted_t: 0,

            i_placed: (shape==MinoType::I) as usize,
            t_placed: (shape==MinoType::T) as usize,

            well
        }
    }
}

#[derive(Debug, Default)]

pub struct ComboSegment{
    pub frames:f64,
    pub attack:usize,
    pub blocks: usize,
    pub multipliers: Vec<usize>
}

impl ComboSegment{
    fn new(starting_attack: usize, is_multiplier: bool)->Self{
        let mut multipliers = Vec::new();
        if is_multiplier{
            multipliers.push(0);
        }
        Self { frames: 0.0, attack: starting_attack, blocks: 1, multipliers }
    }
}