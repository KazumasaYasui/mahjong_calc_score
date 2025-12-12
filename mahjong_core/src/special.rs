use crate::tile::Tile;
use crate::tile::{Honor, Suit, TileKey};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum SpecialHand {
    Chiitoitsu, // 七対子
    Kokushi,    // 国士無双
    Kokushi13,  // 国士無双十三面待ち（ダブル役満扱い）
}

pub fn detect_special(tiles14: &[Tile], win_tile: Tile, has_calls: bool) -> Option<SpecialHand> {
    if has_calls {
        return None; // 七対子/国士は副露不可
    }

    let counts = count_tiles(tiles14);

    if is_chiitoitsu(&counts) {
        return Some(SpecialHand::Chiitoitsu);
    }

    if is_kokushi(&counts) {
        // 13面待ち：対子になっている牌が和了牌である（＝13種が揃っていて、最後の1枚がどれでもOKだった形）
        let wk = TileKey::from_tile(&win_tile);
        if counts.get(&wk).copied().unwrap_or(0) == 2 {
            return Some(SpecialHand::Kokushi13);
        }
        return Some(SpecialHand::Kokushi);
    }

    None
}

fn count_tiles(tiles: &[Tile]) -> HashMap<TileKey, u8> {
    let mut map = HashMap::new();
    for t in tiles {
        *map.entry(TileKey::from_tile(t)).or_insert(0) += 1;
    }
    map
}

fn is_chiitoitsu(counts: &HashMap<TileKey, u8>) -> bool {
    if counts.len() != 7 {
        return false;
    }
    counts.values().all(|&c| c == 2)
}

fn is_kokushi(counts: &HashMap<TileKey, u8>) -> bool {
    let orphans = kokushi_orphans();
    let mut pair_found = false;

    for o in &orphans {
        match counts.get(o).copied().unwrap_or(0) {
            1 => {}
            2 => {
                if pair_found {
                    return false;
                }
                pair_found = true;
            }
            _ => return false,
        }
    }

    // 余計な牌がない（13種のみ）
    if counts.len() != 13 {
        return false;
    }

    pair_found
}

fn kokushi_orphans() -> Vec<TileKey> {
    let mut v = vec![];

    for &s in &[Suit::Man, Suit::Pin, Suit::Sou] {
        v.push(TileKey {
            suit: s,
            num: 1,
            honor: None,
        });
        v.push(TileKey {
            suit: s,
            num: 9,
            honor: None,
        });
    }

    for &h in &[
        Honor::East,
        Honor::South,
        Honor::West,
        Honor::North,
        Honor::White,
        Honor::Green,
        Honor::Red,
    ] {
        v.push(TileKey {
            suit: Suit::Honor,
            num: 0,
            honor: Some(h),
        });
    }

    v
}
