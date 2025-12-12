use crate::tile::{Honor, Tile};

/// Convert dora indicator tile -> actual dora tile (red flag is always false here)
pub fn indicator_to_dora(ind: Tile) -> Tile {
    if ind.is_honor() {
        let h = ind.honor.expect("honor must have honor enum");
        let next = match h {
            Honor::East => Honor::South,
            Honor::South => Honor::West,
            Honor::West => Honor::North,
            Honor::North => Honor::East,
            Honor::White => Honor::Green,
            Honor::Green => Honor::Red,
            Honor::Red => Honor::White,
        };
        Tile::honor(next)
    } else {
        let suit = ind.suit;
        let n = ind.num;
        let next = if n == 9 { 1 } else { n + 1 };
        Tile {
            suit,
            num: next,
            honor: None,
            red: false,
        }
    }
}

/// Count dora in given tiles (hand + win tile + meld tiles) based on indicator list.
/// - red fives counted separately by caller
pub fn count_dora_from_indicators(all_tiles: &[Tile], indicators: &[Tile]) -> u32 {
    let doras: Vec<_> = indicators.iter().map(|&t| indicator_to_dora(t)).collect();

    let mut count = 0u32;
    for tile in all_tiles {
        for d in &doras {
            if tile.base_id() == d.base_id() {
                count += 1;
            }
        }
    }
    count
}

/// Count aka dora (red fives) in given tiles
pub fn count_aka(all_tiles: &[Tile]) -> u32 {
    all_tiles
        .iter()
        .filter(|t| t.red && !t.is_honor() && t.num == 5)
        .count() as u32
}
