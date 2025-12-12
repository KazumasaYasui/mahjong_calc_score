use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Wind {
    E,
    S,
    W,
    N,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WinType {
    RON,
    TSUMO,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Riichi {
    NONE,
    RIICHI,
    DOUBLE,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MeldType {
    CHI,
    PON,
    MINKAN,
    ANKAN,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meld {
    #[serde(rename = "type")]
    pub meld_type: MeldType,
    pub tiles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flags {
    pub riichi: Riichi,
    pub ippatsu: bool,
    pub rinshan: bool,
    pub chankan: bool,
    pub haitei: bool,
    pub houtei: bool,
    pub tenhou: bool,
    pub chihou: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreRequest {
    pub round_wind: Wind,
    pub seat_wind: Wind,
    pub kyotaku: u32,
    pub honba: u32,

    pub win_type: WinType,
    pub dealer: bool,

    pub hand_tiles: Vec<String>,
    pub win_tile: String,

    pub melds: Vec<Meld>,

    pub dora_indicators: Vec<String>,
    pub kan_dora_indicators: Vec<String>,
    pub ura_indicators: Vec<String>,
    pub kan_ura_indicators: Vec<String>,

    pub flags: Flags,
}

#[derive(Debug, Serialize)]
pub struct ScoreResult {
    pub total_points: u32,
    pub yakuman: u32,
    pub han: u32,
    pub fu: u32,
    pub yaku: Vec<String>,
    pub dora_han: u32,
    pub ura_dora_han: u32,
    pub aka_dora_han: u32,
}

mod decompose;
mod dora;
mod engine;
mod fu;
mod points;
mod score;
mod special;
mod tile;
mod yaku;

pub use score::score;
