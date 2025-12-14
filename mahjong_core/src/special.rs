use std::collections::HashMap;

use crate::tile::{Suit, Tile, TileKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialHand {
    Chiitoitsu,
    Kokushi,
    Kokushi13, // 国士無双十三面待ち（ダブル役満扱い）
}

/// tiles14 は「手牌13 + 和了牌1」を想定
/// has_calls = 副露があるかどうか（副露があると七対子・国士は不可として None）
pub fn detect_special(tiles14: &[Tile], win_tile: Tile, has_calls: bool) -> Option<SpecialHand> {
    if has_calls {
        return None;
    }
    if tiles14.len() != 14 {
        return None;
    }

    if is_chiitoitsu(tiles14) {
        return Some(SpecialHand::Chiitoitsu);
    }

    if let Some(sp) = detect_kokushi(tiles14, win_tile) {
        return Some(sp);
    }

    None
}

fn is_chiitoitsu(tiles14: &[Tile]) -> bool {
    let mut counts: HashMap<TileKey, u8> = HashMap::new();
    for t in tiles14 {
        *counts.entry(TileKey::from_tile(t)).or_insert(0) += 1;
    }

    // 7種類がすべて2枚ずつ
    if counts.len() != 7 {
        return false;
    }
    counts.values().all(|&c| c == 2)
}

fn detect_kokushi(tiles14: &[Tile], win_tile: Tile) -> Option<SpecialHand> {
    // 国士の対象：么九（1/9） + 字牌 = 13種
    // tiles14 がその13種すべてを含み、どれか1種が2枚なら国士
    let mut counts: HashMap<TileKey, u8> = HashMap::new();
    for t in tiles14 {
        let k = TileKey::from_tile(t);
        if !is_terminal_or_honor(t) {
            return None;
        }
        *counts.entry(k).or_insert(0) += 1;
    }

    // 13種揃っている必要がある
    if counts.len() != 13 {
        return None;
    }

    // 2枚の種類がちょうど1つ、他は1枚
    let mut pair_key: Option<TileKey> = None;
    for (k, &c) in &counts {
        match c {
            1 => {}
            2 => {
                if pair_key.is_some() {
                    return None;
                }
                pair_key = Some(*k);
            }
            _ => return None,
        }
    }

    let Some(pk) = pair_key else {
        return None;
    };

    let wk = TileKey::from_tile(&win_tile);

    // 十三面待ち：和了牌が「対子になった牌」であること
    // （= 和了前は13種1枚ずつで、どれを引いても国士になる形）
    if pk == wk {
        Some(SpecialHand::Kokushi13)
    } else {
        Some(SpecialHand::Kokushi)
    }
}

fn is_terminal_or_honor(t: &Tile) -> bool {
    if t.suit == Suit::Honor {
        return true;
    }
    t.num == 1 || t.num == 9
}
