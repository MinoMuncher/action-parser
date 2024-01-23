use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Board = Vec<MinoType>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlacementStats {
    pub shape: MinoType,
    #[serde(rename = "linesCleared")]
    pub lines_cleared: usize,
    #[serde(rename = "downstackCleared")]
    pub garbage_cleared: usize,
    pub keypresses: usize,
    pub attack: Vec<usize>,
    #[serde(rename = "type")]
    pub clear_type: ClearType,
    pub combo: usize,
    #[serde(rename = "BTBChain")]
    pub btb_chain: usize,
    #[serde(rename = "BTBClear")]
    pub btb_clear: bool,
    #[serde(rename = "frameDelay")]
    pub frame_delay: f64,
    #[serde(rename = "attackRecieved")]
    pub attack_received: Vec<usize>,
    #[serde(rename = "attackTanked")]
    pub attack_tanked: Vec<usize>,
    pub board: Board,
    pub queue: Vec<MinoType>,
}

#[derive(Hash, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClearType {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "TSPIN_MINI")]
    TspinMini,
    #[serde(rename = "TSPIN")]
    Tspin,
    #[serde(rename = "TSPIN_MINI_SINGLE")]
    TspinMiniSingle,
    #[serde(rename = "TSPIN_SINGLE")]
    TspinSingle,
    #[serde(rename = "SINGLE")]
    Single,
    #[serde(rename = "TSPIN_MINI_DOUBLE")]
    TspinMiniDouble,
    #[serde(rename = "TSPIN_DOUBLE")]
    TspinDouble,
    #[serde(rename = "DOUBLE")]
    Double,
    #[serde(rename = "TSPIN_TRIPLE")]
    TspinTriple,
    #[serde(rename = "TRIPLE")]
    Triple,
    #[serde(rename = "TSPIN_QUAD")]
    TspinQuad,
    #[serde(rename = "QUAD")]
    Quad,
    #[serde(rename = "TSPIN_PENTA")]
    TspinPenta,
    #[serde(rename = "PENTA")]
    Penta,
    #[serde(rename = "PERFECT_CLEAR")]
    PerfectClear,
}

impl ClearType {
    pub fn is_multipliable(&self) -> bool {
        self == &Self::TspinDouble || self == &Self::TspinTriple || self == &Self::Quad
    }
    pub fn is_btb_clear(&self) -> bool {
        self == &Self::TspinDouble
            || self == &Self::TspinTriple
            || self == &Self::Quad
            || self == &Self::TspinSingle
            || self == &Self::TspinMiniSingle
            || self == &Self::TspinMiniDouble
    }
}

#[derive(Debug)]
pub struct OutOfBoundsError(u8);
impl std::fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error converting u8 to ClearType: Value {} out of bounds",
            self.0
        ))?;
        Ok(())
    }
}
impl std::error::Error for OutOfBoundsError {}

impl std::convert::TryFrom<u8> for ClearType {
    type Error = OutOfBoundsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ClearType::None),
            1 => Ok(ClearType::TspinMini),
            2 => Ok(ClearType::Tspin),
            3 => Ok(ClearType::TspinMiniSingle),
            4 => Ok(ClearType::TspinSingle),
            5 => Ok(ClearType::Single),
            6 => Ok(ClearType::TspinMiniDouble),
            7 => Ok(ClearType::TspinDouble),
            8 => Ok(ClearType::Double),
            9 => Ok(ClearType::TspinTriple),
            10 => Ok(ClearType::Triple),
            11 => Ok(ClearType::TspinQuad),
            12 => Ok(ClearType::Quad),
            13 => Ok(ClearType::TspinPenta),
            14 => Ok(ClearType::Penta),
            15 => Ok(ClearType::PerfectClear),
            _ => Err(OutOfBoundsError(value)),
        }
    }
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MinoType {
    Z,
    L,
    O,
    S,
    I,
    J,
    T,
    Garbage,
    Empty,
}

impl std::convert::TryFrom<u8> for MinoType {
    type Error = OutOfBoundsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MinoType::Z),
            1 => Ok(MinoType::L),
            2 => Ok(MinoType::O),
            3 => Ok(MinoType::S),
            4 => Ok(MinoType::I),
            5 => Ok(MinoType::J),
            6 => Ok(MinoType::T),
            7 => Ok(MinoType::Garbage),
            8 => Ok(MinoType::Empty),
            _ => Err(OutOfBoundsError(value)),
        }
    }
}
