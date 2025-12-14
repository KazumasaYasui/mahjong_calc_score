use crate::decompose::{decompose_with_blocks_needed, Block, OpenInfo};
use crate::dora::{count_aka, count_dora_from_indicators};
use crate::fu::calc_fu;
use crate::points::calc_points;
use crate::special::{detect_special, SpecialHand};
use crate::tile::{Honor, Suit, Tile, TileKey};
use crate::yaku::{eval_special_yaku, eval_yaku_standard};
use crate::{MeldType, Riichi, ScoreRequest, ScoreResult, WinType};
use std::collections::HashMap;

pub fn score_best(req: &ScoreRequest) -> ScoreResult {
    // concealed tiles (hand + win)
    let hand: Vec<Tile> = req
        .hand_tiles
        .iter()
        .map(|s| Tile::from_code(s).unwrap())
        .collect();
    let win_tile = Tile::from_code(&req.win_tile).unwrap();

    let mut tiles14 = vec![];
    tiles14.extend(hand.iter().copied());
    tiles14.push(win_tile);

    // ✅ 分解の安定のため、必ずソートする
    sort_tiles(&mut tiles14);

    // all tiles (for dora/aka count)
    let mut all_tiles = tiles14.clone();
    for m in &req.melds {
        for t in &m.tiles {
            all_tiles.push(Tile::from_code(t).unwrap());
        }
    }
    sort_tiles(&mut all_tiles);

    let has_any_melds = !req.melds.is_empty();

    // indicators
    let dora_inds: Vec<Tile> = req
        .dora_indicators
        .iter()
        .map(|s| Tile::from_code(s).unwrap())
        .collect();
    let kan_dora_inds: Vec<Tile> = req
        .kan_dora_indicators
        .iter()
        .map(|s| Tile::from_code(s).unwrap())
        .collect();
    let ura_inds: Vec<Tile> = req
        .ura_indicators
        .iter()
        .map(|s| Tile::from_code(s).unwrap())
        .collect();
    let kan_ura_inds: Vec<Tile> = req
        .kan_ura_indicators
        .iter()
        .map(|s| Tile::from_code(s).unwrap())
        .collect();

    // melds -> blocks
    let meld_blocks = melds_to_blocks(req);
    let meld_block_count = meld_blocks.len();

    // menzen strict: CHI/PON/MINKAN breaks menzen; ANKAN doesn't
    let menzen = is_menzen_strict(req);

    // open info for fu/yaku
    let open_info = OpenInfo::from_melds(&req.melds);

    let mut best: Option<ScoreResult> = None;

    // A: special hands
    let sp_opt = detect_special(&tiles14, win_tile, has_any_melds)
        .or_else(|| detect_special_fallback(&tiles14, win_tile, has_any_melds));

    if let Some(sp) = sp_opt {
        let yr = eval_special_yaku(sp);

        let aka = count_aka(&all_tiles);
        let dora = count_dora_from_indicators(&all_tiles, &dora_inds)
            + count_dora_from_indicators(&all_tiles, &kan_dora_inds);
        let ura = if req.flags.riichi != Riichi::NONE {
            count_dora_from_indicators(&all_tiles, &ura_inds)
                + count_dora_from_indicators(&all_tiles, &kan_ura_inds)
        } else {
            0
        };

        let han = yr.han + dora + ura + aka;
        let yakuman = yr.yakuman;

        let mut yaku = yr.yaku;
        if dora > 0 {
            yaku.push(format!("ドラ{}", dora));
        }
        if ura > 0 {
            yaku.push(format!("裏ドラ{}", ura));
        }
        if aka > 0 {
            yaku.push(format!("赤ドラ{}", aka));
        }

        let fu = match sp {
            SpecialHand::Chiitoitsu => 25,
            SpecialHand::Kokushi | SpecialHand::Kokushi13 => 0,
        };

        let bd = calc_points(
            fu,
            han,
            yakuman,
            req.win_type,
            req.dealer,
            req.honba,
            req.kyotaku,
        );

        best = Some(ScoreResult {
            total_points: bd.total_points,
            yakuman,
            han,
            fu,
            yaku,
            dora_han: dora,
            ura_dora_han: ura,
            aka_dora_han: aka,
        });
    }

    // Standard hand patterns:
    // ✅ decompose.rs の仕様：pair は HandPattern.pair に別で保持される
    //    よって blocks_needed は「面子数」= 4 - 副露面子数
    if meld_block_count > 4 {
        return ScoreResult {
            total_points: 0,
            yakuman: 0,
            han: 0,
            fu: 0,
            yaku: vec!["副露が多すぎます（面子数が4を超えています）".into()],
            dora_han: 0,
            ura_dora_han: 0,
            aka_dora_han: 0,
        };
    }

    let blocks_needed = 4 - meld_block_count;
    let mut patterns = decompose_with_blocks_needed(&tiles14, blocks_needed);

    // 切り分け用：分解不能と役なしを区別
    if patterns.is_empty() {
        // ✅ 七対子/国士など special で best が埋まっているなら、それを返す
        if let Some(b) = best {
            return b;
        }

        // ✅ special でも拾えず、標準形でも分解できない場合だけエラー
        return ScoreResult {
            total_points: 0,
            yakuman: 0,
            han: 0,
            fu: 0,
            yaku: vec![format!(
                "分解できませんでした: tiles14={:?}, blocks_needed={}, meld_blocks={}",
                tiles14, blocks_needed, meld_block_count
            )],
            dora_han: 0,
            ura_dora_han: 0,
            aka_dora_han: 0,
        };
    }

    for mut p in patterns.drain(..) {
        // attach meld blocks
        let mut blocks = p.blocks;
        blocks.extend(meld_blocks.clone());
        p.blocks = blocks;

        // attach metadata
        p.menzen = menzen;
        p.pair_key = p.extract_pair_key();
        p.open_info = Some(open_info.clone());

        let yr = eval_yaku_standard(
            &p,
            &tiles14,
            win_tile,
            req.win_type,
            req.round_wind,
            req.seat_wind,
            req.flags.riichi,
            req.flags.ippatsu,
            req.flags.rinshan,
            req.flags.chankan,
            req.flags.haitei,
            req.flags.houtei,
        );

        // yakuなしは無効
        if yr.yakuman == 0 && yr.han == 0 {
            continue;
        }

        let (mut fu, _wt) = calc_fu(
            &p,
            win_tile,
            req.win_type,
            p.menzen,
            req.round_wind,
            req.seat_wind,
        );

        // 平和ツモは 20符
        if yr.yaku.iter().any(|y| y == "平和") && req.win_type == WinType::TSUMO {
            fu = 20;
        }

        let aka = count_aka(&all_tiles);
        let dora = count_dora_from_indicators(&all_tiles, &dora_inds)
            + count_dora_from_indicators(&all_tiles, &kan_dora_inds);
        let ura = if req.flags.riichi != Riichi::NONE {
            count_dora_from_indicators(&all_tiles, &ura_inds)
                + count_dora_from_indicators(&all_tiles, &kan_ura_inds)
        } else {
            0
        };

        let han = yr.han + dora + ura + aka;
        let yakuman = yr.yakuman;

        let mut yaku = yr.yaku;
        if dora > 0 {
            yaku.push(format!("ドラ{}", dora));
        }
        if ura > 0 {
            yaku.push(format!("裏ドラ{}", ura));
        }
        if aka > 0 {
            yaku.push(format!("赤ドラ{}", aka));
        }

        let bd = calc_points(
            fu,
            han,
            yakuman,
            req.win_type,
            req.dealer,
            req.honba,
            req.kyotaku,
        );

        let cand = ScoreResult {
            total_points: bd.total_points,
            yakuman,
            han,
            fu,
            yaku,
            dora_han: dora,
            ura_dora_han: ura,
            aka_dora_han: aka,
        };

        best = match best {
            None => Some(cand),
            Some(b) => Some(if cand.total_points > b.total_points {
                cand
            } else {
                b
            }),
        };
    }

    best.unwrap_or(ScoreResult {
        total_points: 0,
        yakuman: 0,
        han: 0,
        fu: 0,
        yaku: vec!["役なし（和了不可）".into()],
        dora_han: 0,
        ura_dora_han: 0,
        aka_dora_han: 0,
    })
}

fn is_menzen_strict(req: &ScoreRequest) -> bool {
    // CHI/PON/MINKAN があれば門前ではない。ANKANは門前扱いのまま。
    for m in &req.melds {
        match m.meld_type {
            MeldType::CHI | MeldType::PON | MeldType::MINKAN => return false,
            MeldType::ANKAN => {}
        }
    }
    true
}

fn melds_to_blocks(req: &ScoreRequest) -> Vec<Block> {
    let mut v = vec![];

    for m in &req.melds {
        let tiles: Vec<Tile> = m
            .tiles
            .iter()
            .map(|s| Tile::from_code(s).unwrap())
            .collect();

        match m.meld_type {
            MeldType::CHI => {
                if tiles.len() == 3 {
                    // 前提：tilesが順子として並んでいる（UI側で保証）
                    v.push(Block::Shuntsu(tiles[0], tiles[1], tiles[2]));
                }
            }
            MeldType::PON => {
                if tiles.len() == 3 {
                    v.push(Block::Koutsu(tiles[0], tiles[1], tiles[2]));
                }
            }
            MeldType::MINKAN | MeldType::ANKAN => {
                if tiles.len() == 4 {
                    v.push(Block::Kantsu(tiles[0], tiles[1], tiles[2], tiles[3]));
                }
            }
        }
    }

    v
}

fn detect_special_fallback(
    tiles14: &[Tile],
    win_tile: Tile,
    has_calls: bool,
) -> Option<SpecialHand> {
    if has_calls {
        return None;
    }
    if tiles14.len() != 14 {
        return None;
    }

    // counts
    let mut counts: HashMap<TileKey, u8> = HashMap::new();
    for t in tiles14 {
        *counts.entry(TileKey::from_tile(t)).or_insert(0) += 1;
    }

    // 七対子：7種類がすべて2枚
    if counts.len() == 7 && counts.values().all(|&c| c == 2) {
        return Some(SpecialHand::Chiitoitsu);
    }

    // 国士：13種（么九字牌）すべてが存在し、どれか1種が2枚、他は1枚
    // さらに全牌が么九字牌であること
    for t in tiles14 {
        if !(t.suit == Suit::Honor || t.num == 1 || t.num == 9) {
            return None; // 么九字牌以外が混ざったら国士ではない
        }
    }

    if counts.len() != 13 {
        return None;
    }

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

    let pk = pair_key?;
    let wk = TileKey::from_tile(&win_tile);

    if pk == wk {
        Some(SpecialHand::Kokushi13)
    } else {
        Some(SpecialHand::Kokushi)
    }
}

// ===== ソート用のローカルヘルパー =====

fn sort_tiles(v: &mut Vec<Tile>) {
    v.sort_by_key(|t| (rank_suit(t.suit), rank_honor(t.honor), t.num, t.red));
}

fn rank_suit(s: Suit) -> u8 {
    match s {
        Suit::Man => 0,
        Suit::Pin => 1,
        Suit::Sou => 2,
        Suit::Honor => 3,
    }
}

fn rank_honor(h: Option<Honor>) -> u8 {
    match h {
        Some(Honor::East) => 0,
        Some(Honor::South) => 1,
        Some(Honor::West) => 2,
        Some(Honor::North) => 3,
        Some(Honor::White) => 4,
        Some(Honor::Green) => 5,
        Some(Honor::Red) => 6,
        None => 255,
    }
}
