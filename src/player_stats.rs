use std::collections::HashMap;

use crate::{
    placement_stats::CumulativePlacementStats,
    replay_response::{ClearType, MinoType},
};
use serde::Serialize;

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats {
    pub well_columns: [usize; 10],
    pub clear_types: HashMap<ClearType, usize>,

    pub t_efficiency: f64,
    pub i_efficiency: f64,

    pub cheese_apl: f64,
    pub downstack_apl: f64,
    pub upstack_apl: f64,

    pub apl: f64,
    pub app: f64,

    pub kpp: f64,
    pub kps: f64,

    pub stack_height: f64,
    pub garbage_height: f64,

    pub spike_efficiency: f64,

    pub apm: f64,
    pub opener_apm: f64,
    pub midgame_apm: f64,

    pub pps: f64,
    pub opener_pps: f64,
    pub midgame_pps: f64,
    pub btb_wellshifts: usize,

    pub btb_chain_efficiency: f64,
    pub btb_chain: f64,
    pub btb_chain_apm: f64,
    pub btb_chain_attack: f64,
    pub btb_chain_wellshifts: f64,
    pub btb_chain_app: f64,

    pub max_btb: usize,
    pub max_btb_attack: usize,

    pub combo_chain_efficiency: f64,
    pub combo_chain: f64,
    pub combo_chain_apm: f64,
    pub combo_chain_attack: f64,
    pub combo_chain_app: f64,

    pub max_combo: usize,
    pub max_combo_attack: usize,

    pub average_spike_potential: f64,
    pub average_defence_potential: f64,

    pub pps_variance: f64,

    pub blockfish_score: f64,

    pub burst_pps: f64,
    pub attack_delay_rate: f64,
    pub pre_attack_delay_rate: f64,
}
#[derive(Debug, Clone, Copy)]
struct Burst {
    blocks: usize,
    delay: f64,
}

impl From<&CumulativePlacementStats> for PlayerStats {
    fn from(stats: &CumulativePlacementStats) -> Self {
        let tspins = stats.clear_types[ClearType::TspinDouble as usize]
            + stats.clear_types[ClearType::TspinMiniDouble as usize]
            + stats.clear_types[ClearType::TspinSingle as usize]
            + stats.clear_types[ClearType::TspinMiniSingle as usize]
            + stats.clear_types[ClearType::TspinTriple as usize]
            + stats.clear_types[ClearType::TspinQuad as usize]
            + stats.clear_types[ClearType::TspinPenta as usize];

        let time_frames = stats.delays.iter().sum::<f64>();
        let frame_average = time_frames / stats.delays.len() as f64;
        let time_secs = time_frames / 60.0;
        let opener_time_secs = stats.opener_frames / 60.0;

        let frame_sd = (stats
            .delays
            .iter()
            .map(|delay: &f64| (delay - frame_average).powi(2))
            .sum::<f64>()
            / stats.delays.len() as f64)
            .sqrt();
        let frame_sd = (stats
            .delays
            .iter()
            .filter(|&&delay| (delay - frame_average).abs() < frame_sd * 5.0)
            .map(|delay: &f64| (delay - frame_average).powi(2))
            .sum::<f64>()
            / stats.delays.len() as f64)
            .sqrt();

        let blocks = stats.delays.len() as f64;

        let attack_chains: Vec<_> = stats
            .combo_segments
            .iter()
            .filter(|segment| segment.attack >= 4)
            .collect();

        let true_combo_chains: Vec<_> = stats
            .combo_segments
            .iter()
            .filter(|segment| segment.blocks > 4)
            .collect();
        let true_combo_chain_blocks = true_combo_chains
            .iter()
            .map(|seg| seg.blocks)
            .sum::<usize>() as f64;
        let true_combo_chain_attack = true_combo_chains
            .iter()
            .map(|segment| segment.attack)
            .sum::<usize>() as f64;

        let true_btb_chains: Vec<_> = stats
            .btb_segments
            .iter()
            .filter(|segment| segment.btb >= 4)
            .collect();

        let wellshifts = true_btb_chains
            .iter()
            .map(|segment| segment.wellshifts)
            .sum::<usize>();
        let true_btb_chain_blocks =
            true_btb_chains.iter().map(|seg| seg.blocks).sum::<usize>() as f64;
        let true_btb_chain_attack = true_btb_chains
            .iter()
            .map(|segment| segment.attack)
            .sum::<usize>() as f64;

        let mut clear_types = HashMap::new();

        for clear_type in 0..16 {
            clear_types.insert(
                ClearType::try_from(clear_type).unwrap(),
                stats.clear_types[clear_type as usize],
            );
        }
        let segment_times: Vec<_> = (0..stats.delays.len().saturating_sub(6))
            .map(|start| {
                let seg: Vec<_> = stats.delays.iter().skip(start).take(7).cloned().collect();
                let last = *seg.last().unwrap_or(&0.0);
                (last, seg.iter().sum::<f64>())
            })
            .collect();

        let segment_average =
            segment_times.iter().map(|s| s.1).sum::<f64>() / segment_times.len() as f64;

        let segment_sd = (segment_times
            .iter()
            .map(|(_, delay)| (delay - segment_average).powi(2))
            .sum::<f64>()
            / segment_times.len() as f64)
            .sqrt();

        let mut bursts: Vec<Burst> = Vec::new();
        let mut current_burst = None;
        for &(last, delay) in segment_times.iter() {
            if segment_average - delay > segment_sd {
                match current_burst {
                    None => {
                        current_burst = Some(Burst { blocks: 7, delay });
                    }
                    Some(mut burst) => {
                        burst.blocks += 1;
                        burst.delay += last;
                    }
                }
            } else {
                if let Some(burst) = current_burst {
                    bursts.push(burst);
                }
                current_burst = None;
            }
        }
        if let Some(burst) = current_burst {
            bursts.push(burst);
        }
        //bursts defined as segments that are 1 sd below the average

        let prev_attack_chains: Vec<_> =
            attack_chains.iter().filter_map(|c| c.prev_delay).collect();

        Self {
            well_columns: stats.well_cols,
            clear_types,
            t_efficiency: tspins as f64 / stats.shape_types[MinoType::T as usize] as f64,
            i_efficiency: stats.clear_types[ClearType::Quad as usize] as f64
                / stats.shape_types[MinoType::I as usize] as f64,
            cheese_apl: stats.attack_with_cheese as f64 / stats.exclusive_cheese_cleared as f64,
            downstack_apl: stats.attack_with_garbage as f64
                / stats.exclusive_garbage_cleared as f64,
            upstack_apl: stats.attack_with_stack as f64 / stats.exclusive_stack_cleared as f64,
            apl: stats.attack as f64 / stats.lines_cleared as f64,
            app: stats.attack as f64 / blocks,
            kpp: stats.keypresses as f64 / blocks,
            kps: stats.keypresses as f64 / time_secs,
            stack_height: stats.stack_heights.iter().sum::<usize>() as f64
                / stats.stack_heights.len() as f64,
            garbage_height: stats.garbage_heights.iter().sum::<usize>() as f64
                / stats.garbage_heights.len() as f64,
            spike_efficiency: stats
                .combo_segments
                .iter()
                .filter(|segment| segment.attack >= 10)
                .map(|segment| segment.blocks)
                .sum::<usize>() as f64
                / blocks,
            apm: stats.attack as f64 * 60.0 / time_secs,
            opener_apm: (stats.opener_attack as f64 / opener_time_secs) * 60.0,
            midgame_apm: ((stats.attack - stats.opener_attack) as f64
                / (time_secs - opener_time_secs))
                * 60.0,
            opener_pps: stats.opener_blocks as f64 / opener_time_secs,
            midgame_pps: (blocks - stats.opener_blocks as f64) / (time_secs - opener_time_secs),
            pps: blocks / time_secs,
            btb_wellshifts: wellshifts,
            btb_chain_wellshifts: wellshifts as f64 / true_btb_chains.len() as f64,
            btb_chain_efficiency: true_btb_chains.len() as f64 / stats.btb_segments.len() as f64,
            btb_chain: true_btb_chains
                .iter()
                .map(|segment| segment.btb)
                .sum::<usize>() as f64
                / true_btb_chains.len() as f64,
            btb_chain_apm: true_btb_chain_attack
                / true_btb_chains.iter().map(|seg| seg.frames).sum::<f64>()
                * 3600.0,
            btb_chain_attack: true_btb_chain_attack / true_btb_chains.len() as f64,
            max_btb: stats
                .btb_segments
                .iter()
                .map(|segment| segment.btb)
                .max()
                .unwrap_or(0),
            max_btb_attack: stats
                .btb_segments
                .iter()
                .map(|segment| segment.attack)
                .max()
                .unwrap_or(0),
            combo_chain_efficiency: true_combo_chains.len() as f64
                / stats.combo_segments.len() as f64,
            combo_chain: true_combo_chains
                .iter()
                .map(|seg| seg.blocks - 1)
                .sum::<usize>() as f64
                / true_combo_chains.len() as f64,
            combo_chain_apm: true_combo_chain_attack
                / true_combo_chains.iter().map(|seg| seg.frames).sum::<f64>()
                * 3600.0,
            combo_chain_attack: true_combo_chain_attack / true_combo_chains.len() as f64,
            max_combo: stats
                .combo_segments
                .iter()
                .map(|segment| segment.blocks.saturating_sub(1))
                .max()
                .unwrap_or(0),
            max_combo_attack: stats
                .combo_segments
                .iter()
                .map(|segment| segment.attack)
                .max()
                .unwrap_or(0),
            average_spike_potential: stats.spikable_boards as f64 / stats.pre_spike_boards as f64,
            average_defence_potential: stats.defense_potentials.iter().sum::<usize>() as f64
                / blocks,
            btb_chain_app: true_btb_chain_attack / true_btb_chain_blocks,
            combo_chain_app: true_combo_chain_attack / true_combo_chain_blocks,
            pps_variance: frame_sd / frame_average,
            blockfish_score: stats.blockfish_scores.iter().sum::<usize>() as f64
                / stats.blockfish_scores.len() as f64,
            attack_delay_rate: attack_chains
                .iter()
                .filter(|segment| segment.initial_delay - frame_average > frame_sd)
                .count() as f64
                / (attack_chains.len() as f64),
            pre_attack_delay_rate: prev_attack_chains
                .iter()
                .filter(|&&segment| segment - frame_average > frame_sd)
                .count() as f64
                / (prev_attack_chains.len() as f64),
            burst_pps: bursts.iter().map(|burst| burst.blocks).sum::<usize>() as f64
                / (bursts.iter().map(|burst| burst.delay).sum::<f64>() / 60.0),
        }
    }
}
