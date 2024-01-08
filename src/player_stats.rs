use std::collections::HashMap;

use serde::Serialize;
use crate::{placement_stats::CumulativePlacementStats, replay_response::{ClearType, MinoType}};


#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats{
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



}

impl From<&CumulativePlacementStats> for PlayerStats{
    fn from(stats: &CumulativePlacementStats) -> Self {

        let tspins = 
        stats.clear_types[ClearType::TspinDouble as usize] + 
        stats.clear_types[ClearType::TspinMiniDouble as usize] + 
        stats.clear_types[ClearType::TspinSingle as usize] + 
        stats.clear_types[ClearType::TspinMiniSingle as usize] + 
        stats.clear_types[ClearType::TspinTriple as usize] + 
        stats.clear_types[ClearType::TspinQuad as usize] + 
        stats.clear_types[ClearType::TspinPenta as usize];

        let time_frames = stats.delays.iter().sum::<f64>();
        let frame_average = time_frames/stats.delays.len() as f64;
        let time_secs = time_frames / 60.0;
        let opener_time_secs = stats.opener_frames / 60.0;

        let blocks = stats.delays.len() as f64;

        let true_combo_chains : Vec<_> = stats.combo_segments.iter().filter(|segment|segment.blocks>4).collect();
        let true_combo_chain_blocks = true_combo_chains.iter().map(|seg| seg.blocks).sum::<usize>() as f64;
        let true_combo_chain_attack = true_combo_chains.iter().map(|segment|segment.attack).sum::<usize>() as f64;

        let true_btb_chains : Vec<_> = stats.btb_segments.iter().filter(|segment|segment.btb>=4).collect();
        let wellshifts = true_btb_chains.iter().map(|segment| segment.wellshifts).sum::<usize>();
        let true_btb_chain_blocks = true_btb_chains.iter().map(|seg| seg.blocks).sum::<usize>() as f64;
        let true_btb_chain_attack = true_btb_chains.iter().map(|segment|segment.attack).sum::<usize>() as f64;

        let mut clear_types = HashMap::new();

        for clear_type in 0..16{
            clear_types.insert(ClearType::try_from(clear_type).unwrap(), stats.clear_types[clear_type as usize]);
        }

        Self{
            clear_types,
            t_efficiency: tspins as f64 / stats.shape_types[MinoType::T as usize] as f64,
            i_efficiency: stats.clear_types[ClearType::Quad as usize] as f64 / stats.shape_types[MinoType::I as usize] as f64,
            cheese_apl: stats.attack_with_cheese as f64 / stats.exclusive_cheese_cleared as f64,
            downstack_apl: stats.attack_with_garbage as f64 / stats.exclusive_garbage_cleared as f64,
            upstack_apl: stats.attack_with_stack as f64 / stats.exclusive_stack_cleared as f64,
            apl: stats.attack as f64 / stats.lines_cleared as f64,
            app: stats.attack as f64 / blocks,
            kpp: stats.keypresses as f64 / blocks,
            kps: stats.keypresses as f64 / time_secs,
            stack_height: stats.stack_heights.iter().sum::<usize>() as f64 / stats.stack_heights.len() as f64,
            garbage_height: stats.garbage_heights.iter().sum::<usize>() as f64 / stats.garbage_heights.len() as f64,
            spike_efficiency: stats.combo_segments.iter().filter(|segment|segment.attack>=10).map(|segment|segment.blocks).sum::<usize>() as f64 / blocks,
            apm: stats.attack as f64 * 60.0 / time_secs,
            opener_apm: (stats.opener_attack as f64 / opener_time_secs) * 60.0,
            midgame_apm: ((stats.attack - stats.opener_attack) as f64 / (time_secs-opener_time_secs)) *60.0,
            opener_pps: stats.opener_blocks as f64 / opener_time_secs,
            midgame_pps: (blocks - stats.opener_blocks as f64) / (time_secs-opener_time_secs),
            pps: blocks / time_secs,
            btb_wellshifts: wellshifts,
            btb_chain_wellshifts: wellshifts as f64/ true_btb_chains.len() as f64,
            btb_chain_efficiency: true_btb_chains.iter().map(|segment|segment.btb+1).sum::<usize>() as f64 / (tspins as f64 + blocks),
            btb_chain: true_btb_chains.iter().map(|segment|segment.btb).sum::<usize>() as f64 / true_btb_chains.len() as f64,
            btb_chain_apm: true_btb_chain_attack / true_btb_chains.iter().map(|seg| seg.frames).sum::<f64>() * 3600.0,
            btb_chain_attack: true_btb_chain_attack / true_btb_chains.len() as f64,
            max_btb: stats.btb_segments.iter().map(|segment|segment.btb).max().unwrap_or(0),
            max_btb_attack: stats.btb_segments.iter().map(|segment|segment.attack).max().unwrap_or(0),
            combo_chain_efficiency: true_combo_chain_blocks / blocks,
            combo_chain: true_combo_chains.iter().map(|seg| seg.blocks - 1).sum::<usize>() as f64 / true_combo_chains.len() as f64,
            combo_chain_apm: true_combo_chain_attack / true_combo_chains.iter().map(|seg| seg.frames).sum::<f64>() * 3600.0,
            combo_chain_attack: true_combo_chain_attack / true_combo_chains.len() as f64,
            max_combo: stats.combo_segments.iter().map(|segment|segment.blocks - 1 ).max().unwrap_or(0),
            max_combo_attack: stats.combo_segments.iter().map(|segment|segment.attack).max().unwrap_or(0),
            average_spike_potential: stats.attack_potentials.iter().filter(|x|x>=&&&10).count() as f64 / stats.attack_potentials.len() as f64,
            average_defence_potential: stats.defense_potentials.iter().sum::<usize>() as f64 / blocks,
            btb_chain_app: true_btb_chain_attack/true_btb_chain_blocks,
            combo_chain_app: true_combo_chain_attack/true_combo_chain_blocks,
            pps_variance: (stats.delays.iter().map(|delay|(delay-frame_average).powi(2)).sum::<f64>()/stats.delays.len() as f64).sqrt()/frame_average,
            blockfish_score: stats.blockfish_scores.iter().sum::<usize>() as f64 / stats.blockfish_scores.len() as f64,
        }
    }
}