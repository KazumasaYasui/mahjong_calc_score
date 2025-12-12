use crate::tile::{Honor, Suit, Tile, TileKey};
use crate::{Meld, MeldType};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Block {
    Shuntsu(Tile, Tile, Tile),
    Koutsu(Tile, Tile, Tile),
    Kantsu(Tile, Tile, Tile, Tile),
    Toitsu(Tile, Tile),
}

#[derive(Debug, Clone)]
pub struct OpenInfo {
    open_triplets: HashSet<TileKey>,
    open_kans: HashSet<TileKey>,
}

impl OpenInfo {
    pub fn from_melds(melds: &[Meld]) -> Self {
        let mut open_triplets = HashSet::new();
        let mut open_kans = HashSet::new();

        for m in melds {
            if m.tiles.is_empty() {
                continue;
            }
            let t0 = Tile::from_code(&m.tiles[0]).unwrap();
            let k0 = TileKey::from_tile(&t0);

            match m.meld_type {
                MeldType::CHI => {
                    // 順子は「開き」だが、符(0)なのでここでは保持しない
                }
                MeldType::PON => {
                    open_triplets.insert(k0);
                }
                MeldType::MINKAN => {
                    open_kans.insert(k0);
                }
                MeldType::ANKAN => {
                    // 暗槓は open 扱いではない
                }
            }
        }

        Self {
            open_triplets,
            open_kans,
        }
    }

    pub fn is_open_triplet(&self, k: TileKey) -> bool {
        self.open_triplets.contains(&k)
    }
    pub fn is_open_kan(&self, k: TileKey) -> bool {
        self.open_kans.contains(&k)
    }
}

#[derive(Debug, Clone)]
pub struct HandPattern {
    pub blocks: Vec<Block>, // 副露ブロックも含める（engine側で追加）
    pub pair: Block,        // 雀頭

    pub menzen: bool,
    pub pair_key: Option<TileKey>,
    pub open_info: Option<OpenInfo>,
}

impl HandPattern {
    pub fn extract_pair_key(&self) -> Option<TileKey> {
        match &self.pair {
            Block::Toitsu(t, _) => Some(TileKey::from_tile(t)),
            _ => None,
        }
    }
}

/// 従来互換：副露なし想定（4面子1雀頭）
pub fn decompose_standard_hand(tiles: &[Tile]) -> Vec<HandPattern> {
    decompose_with_blocks_needed(tiles, 4)
}

/// ✅ 副露あり対応：必要な面子数を指定して分解する
/// blocks_needed: 手牌側で分解したい面子数（= 4 - 副露ブロック数）
pub fn decompose_with_blocks_needed(tiles: &[Tile], blocks_needed: usize) -> Vec<HandPattern> {
    let mut counts = count_tiles(tiles);
    let mut results = vec![];

    // pair候補も順序を安定させる（デバッグしやすくするため）
    let mut keys: Vec<TileKey> = counts.keys().cloned().collect();
    keys.sort_by_key(key_rank);

    for pair_key in keys {
        if counts.get(&pair_key).copied().unwrap_or(0) >= 2 {
            *counts.get_mut(&pair_key).unwrap() -= 2;

            let mut blocks = vec![];
            dfs_blocks(
                &mut counts,
                &mut blocks,
                &mut results,
                pair_key,
                blocks_needed,
            );

            *counts.get_mut(&pair_key).unwrap() += 2;
        }
    }

    results
}

fn dfs_blocks(
    counts: &mut HashMap<TileKey, u8>,
    blocks: &mut Vec<Block>,
    results: &mut Vec<HandPattern>,
    pair_key: TileKey,
    blocks_needed: usize,
) {
    // ブロック数が必要数に達したら、残り牌が0かチェックして完了
    if blocks.len() == blocks_needed {
        if counts.values().all(|&c| c == 0) {
            let pair = Block::Toitsu(pair_key.to_tile(), pair_key.to_tile());
            let mut hp = HandPattern {
                blocks: blocks.clone(),
                pair,
                menzen: true,             // engineで上書き
                pair_key: Some(pair_key), // 念のため
                open_info: None,          // engineで上書き
            };
            hp.pair_key = hp.extract_pair_key();
            results.push(hp);
        }
        return;
    }

    // まだブロックを作る必要があるのに、残りがない → 失敗
    if counts.values().all(|&c| c == 0) {
        return;
    }

    // ✅ HashMapの順序に依存すると「9から始めて詰む」等が起きる
    //    常に「残り牌のうち最小の牌」を選ぶ（これが標準的な分解の定石）
    let key = match min_nonzero_key(counts) {
        Some(k) => k,
        None => return,
    };

    // 刻子
    if counts.get(&key).copied().unwrap_or(0) >= 3 {
        *counts.get_mut(&key).unwrap() -= 3;
        blocks.push(Block::Koutsu(key.to_tile(), key.to_tile(), key.to_tile()));
        dfs_blocks(counts, blocks, results, pair_key, blocks_needed);
        blocks.pop();
        *counts.get_mut(&key).unwrap() += 3;
    }

    // 順子（開始牌=key のみ作る。keyが最小なのでこれで十分探索できる）
    if key.suit != Suit::Honor {
        if let (Some(k2), Some(k3)) = (key.next(), key.next2()) {
            if counts.get(&k2).copied().unwrap_or(0) > 0
                && counts.get(&k3).copied().unwrap_or(0) > 0
            {
                *counts.get_mut(&key).unwrap() -= 1;
                *counts.get_mut(&k2).unwrap() -= 1;
                *counts.get_mut(&k3).unwrap() -= 1;

                blocks.push(Block::Shuntsu(key.to_tile(), k2.to_tile(), k3.to_tile()));
                dfs_blocks(counts, blocks, results, pair_key, blocks_needed);
                blocks.pop();

                *counts.get_mut(&key).unwrap() += 1;
                *counts.get_mut(&k2).unwrap() += 1;
                *counts.get_mut(&k3).unwrap() += 1;
            }
        }
    }
}

fn count_tiles(tiles: &[Tile]) -> HashMap<TileKey, u8> {
    let mut map = HashMap::new();
    for t in tiles {
        *map.entry(TileKey::from_tile(t)).or_insert(0) += 1;
    }
    map
}

// ===== 追加：TileKey の順序付け（分解の安定化） =====

fn min_nonzero_key(counts: &HashMap<TileKey, u8>) -> Option<TileKey> {
    counts
        .iter()
        .filter(|(_, &v)| v > 0)
        .map(|(k, _)| *k)
        .min_by_key(|k| key_rank(k))
}

fn key_rank(k: &TileKey) -> (u8, u8, u8) {
    // suit: Man < Pin < Sou < Honor
    let suit_rank = match k.suit {
        Suit::Man => 0,
        Suit::Pin => 1,
        Suit::Sou => 2,
        Suit::Honor => 3,
    };

    // honor: winds < dragons (None は 255)
    let honor_rank = match k.honor {
        Some(Honor::East) => 0,
        Some(Honor::South) => 1,
        Some(Honor::West) => 2,
        Some(Honor::North) => 3,
        Some(Honor::White) => 4,
        Some(Honor::Green) => 5,
        Some(Honor::Red) => 6,
        None => 255,
    };

    (suit_rank, honor_rank, k.num)
}
