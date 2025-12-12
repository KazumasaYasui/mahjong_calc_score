#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Man,
    Pin,
    Sou,
    Honor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Honor {
    East,
    South,
    West,
    North,
    White,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    pub suit: Suit,
    pub num: u8,
    pub honor: Option<Honor>,
    pub red: bool,
}

impl Tile {
    pub fn from_code(code: &str) -> Result<Self, String> {
        match code {
            "E" => return Ok(Self::honor(Honor::East)),
            "S" => return Ok(Self::honor(Honor::South)),
            "W" => return Ok(Self::honor(Honor::West)),
            "N" => return Ok(Self::honor(Honor::North)),
            "P" => return Ok(Self::honor(Honor::White)),
            "F" => return Ok(Self::honor(Honor::Green)),
            "C" => return Ok(Self::honor(Honor::Red)),
            _ => {}
        }

        let bytes = code.as_bytes();
        if bytes.len() != 2 {
            return Err(format!("invalid tile code: {}", code));
        }

        let n = (bytes[0] as char)
            .to_digit(10)
            .ok_or_else(|| format!("invalid number: {}", code))? as u8;

        let suit = match bytes[1] as char {
            'm' => Suit::Man,
            'p' => Suit::Pin,
            's' => Suit::Sou,
            _ => return Err(format!("invalid suit: {}", code)),
        };

        if n == 0 {
            Ok(Tile {
                suit,
                num: 5,
                honor: None,
                red: true,
            })
        } else {
            if !(1..=9).contains(&n) {
                return Err(format!("invalid number: {}", code));
            }
            Ok(Tile {
                suit,
                num: n,
                honor: None,
                red: false,
            })
        }
    }

    pub fn honor(h: Honor) -> Self {
        Tile {
            suit: Suit::Honor,
            num: 0,
            honor: Some(h),
            red: false,
        }
    }

    pub fn is_honor(&self) -> bool {
        self.suit == Suit::Honor
    }

    /// 赤牌を無視した同一性（ドラ判定用）
    pub fn base_id(&self) -> (Suit, u8, Option<Honor>) {
        (self.suit, self.num, self.honor)
    }
}

/// 🔑 分解・比較専用キー（赤牌を無視）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub suit: Suit,
    pub num: u8,
    pub honor: Option<Honor>,
}

impl TileKey {
    pub fn from_tile(t: &Tile) -> Self {
        TileKey {
            suit: t.suit,
            num: t.num,
            honor: t.honor,
        }
    }

    pub fn to_tile(&self) -> Tile {
        Tile {
            suit: self.suit,
            num: self.num,
            honor: self.honor,
            red: false,
        }
    }

    pub fn next(&self) -> Option<Self> {
        if self.suit == Suit::Honor || self.num >= 9 {
            None
        } else {
            Some(TileKey {
                suit: self.suit,
                num: self.num + 1,
                honor: None,
            })
        }
    }

    pub fn next2(&self) -> Option<Self> {
        if self.suit == Suit::Honor || self.num >= 8 {
            None
        } else {
            Some(TileKey {
                suit: self.suit,
                num: self.num + 2,
                honor: None,
            })
        }
    }
}
