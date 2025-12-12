use crate::decompose::{Block, HandPattern};
use crate::tile::{Honor, Suit, Tile, TileKey};
use crate::{WinType, Wind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitType {
    Ryanmen, // 両面
    Kanchan, // 嵌張
    Penchan, // 辺張
    Tanki,   // 単騎
    Shanpon, // 双碰
}

#[derive(Debug, Clone, Copy)]
pub struct WaitInfo {
    pub wait_type: WaitType,
    pub ron_completed_triplet: Option<TileKey>,
}

/// yaku判定から使う待ち形判定（互換API）
pub fn detect_wait_type(pattern: &HandPattern, win_tile: Tile) -> WaitType {
    detect_wait_info(pattern, win_tile, WinType::TSUMO).wait_type
}

pub fn calc_fu(
    pattern: &HandPattern,
    win_tile: Tile,
    win_type: WinType,
    menzen: bool,
    round_wind: Wind,
    seat_wind: Wind,
) -> (u32, WaitType) {
    // 待ち形（+ シャンポンロンで完成した刻子キー）
    let wi = detect_wait_info(pattern, win_tile, win_type);

    // ---- 基本符 ----
    // 基本は 20符
    let mut fu: u32 = 20;

    // ツモ 2符
    if win_type == WinType::TSUMO {
        fu += 2;
    }

    // 門前ロン 10符
    if win_type == WinType::RON && menzen {
        fu += 10;
    }

    // 雀頭役牌 2符（場風/自風/三元）
    if let Some(pair_key) = pattern.pair_key {
        fu += pair_fu(pair_key, round_wind, seat_wind);
    }

    // 面子符
    for b in &pattern.blocks {
        fu += block_fu(b, pattern.open_info.as_ref(), wi.ron_completed_triplet);
    }

    // 待ち符（嵌張・辺張・単騎 = +2）
    if matches!(
        wi.wait_type,
        WaitType::Kanchan | WaitType::Penchan | WaitType::Tanki
    ) {
        fu += 2;
    }

    // ---- 例外と最低符 ----
    // 20符が成立するのは「平和ツモ（20符固定）」のみ。
    // それ以外で符が20になってしまった場合は 30符にする（一般的ルール）
    //
    // ここでは yaku.rs に依存せず、形から「平和形」を推定して判定する。
    // 条件:
    // 1) 全面子が順子
    // 2) 雀頭が役牌でない
    // 3) 待ちが両面
    // 4) 門前
    // 5) ツモ
    //
    // ※この条件を満たす場合だけ 20符を許可。そうでなければ最低30符。
    if fu == 20 {
        let pinfu_like = is_pinfu_shape(pattern, round_wind, seat_wind, wi.wait_type);
        let allow_20 = menzen && win_type == WinType::TSUMO && pinfu_like;
        if !allow_20 {
            fu = 30;
        }
    }

    // 10符単位切り上げ（25符固定の七対子は engine 側で別扱いなのでここでは不要）
    fu = round_up_10(fu);

    (fu, wi.wait_type)
}

fn round_up_10(x: u32) -> u32 {
    ((x + 9) / 10) * 10
}

fn pair_fu(pair: TileKey, round_wind: Wind, seat_wind: Wind) -> u32 {
    if pair.suit != Suit::Honor {
        return 0;
    }
    let h = pair.honor.unwrap();

    let mut fu = 0;

    // 三元牌
    if matches!(h, Honor::White | Honor::Green | Honor::Red) {
        fu += 2;
    }

    // 場風 / 自風（連風牌なら両方入り 4符）
    if h == wind_to_honor(round_wind) {
        fu += 2;
    }
    if h == wind_to_honor(seat_wind) {
        fu += 2;
    }

    fu
}

fn wind_to_honor(w: Wind) -> Honor {
    match w {
        Wind::E => Honor::East,
        Wind::S => Honor::South,
        Wind::W => Honor::West,
        Wind::N => Honor::North,
    }
}

fn block_fu(
    block: &Block,
    open_info: Option<&crate::decompose::OpenInfo>,
    ron_completed_triplet: Option<TileKey>,
) -> u32 {
    match block {
        Block::Shuntsu(_, _, _) => 0,
        Block::Toitsu(_, _) => 0,

        Block::Koutsu(t, _, _) => {
            let k = TileKey::from_tile(t);

            // 開いている（副露ポン）なら明刻
            let mut open = open_info.map(|o| o.is_open_triplet(k)).unwrap_or(false);

            // ✅ シャンポン待ちのロンで完成した刻子は「明刻扱い」
            if ron_completed_triplet == Some(k) {
                open = true;
            }

            triplet_fu(k, open)
        }

        Block::Kantsu(t, _, _, _) => {
            let k = TileKey::from_tile(t);

            // 明槓なら open、暗槓なら open=false のまま（open_info が持つのは明槓のみ）
            let open = open_info.map(|o| o.is_open_kan(k)).unwrap_or(false);

            kan_fu(k, open)
        }
    }
}

fn is_terminal_or_honor(k: TileKey) -> bool {
    if k.suit == Suit::Honor {
        return true;
    }
    k.num == 1 || k.num == 9
}

fn triplet_fu(k: TileKey, open: bool) -> u32 {
    let th = is_terminal_or_honor(k);
    match (open, th) {
        (true, false) => 2,  // 明刻 中張
        (true, true) => 4,   // 明刻 么九字
        (false, false) => 4, // 暗刻 中張
        (false, true) => 8,  // 暗刻 么九字
    }
}

fn kan_fu(k: TileKey, open: bool) -> u32 {
    let th = is_terminal_or_honor(k);
    match (open, th) {
        (true, false) => 8,   // 明槓 中張
        (true, true) => 16,   // 明槓 么九字
        (false, false) => 16, // 暗槓 中張
        (false, true) => 32,  // 暗槓 么九字
    }
}

pub fn detect_wait_info(pattern: &HandPattern, win_tile: Tile, win_type: WinType) -> WaitInfo {
    let wk = TileKey::from_tile(&win_tile);

    // 和了牌が雀頭なら単騎
    if let Some(pk) = pattern.pair_key {
        if pk == wk {
            return WaitInfo {
                wait_type: WaitType::Tanki,
                ron_completed_triplet: None,
            };
        }
    }

    // まず順子待ち（両面/嵌張/辺張）を見つける
    for b in &pattern.blocks {
        if let Block::Shuntsu(a, b2, c) = b {
            let ka = TileKey::from_tile(a);
            let kb = TileKey::from_tile(b2);
            let kc = TileKey::from_tile(c);

            if wk == ka || wk == kb || wk == kc {
                if wk == kb {
                    return WaitInfo {
                        wait_type: WaitType::Kanchan,
                        ron_completed_triplet: None,
                    };
                }

                // 辺張: 1-2-3 の 3待ち / 7-8-9 の 7待ち
                if ka.suit != Suit::Honor {
                    if ka.num == 1 && wk == kc {
                        return WaitInfo {
                            wait_type: WaitType::Penchan,
                            ron_completed_triplet: None,
                        };
                    }
                    if ka.num == 7 && wk == ka {
                        return WaitInfo {
                            wait_type: WaitType::Penchan,
                            ron_completed_triplet: None,
                        };
                    }
                }

                return WaitInfo {
                    wait_type: WaitType::Ryanmen,
                    ron_completed_triplet: None,
                };
            }
        }
    }

    // 次に双碰（刻子待ち）を確認
    for b in &pattern.blocks {
        match b {
            Block::Koutsu(t, _, _) => {
                let k = TileKey::from_tile(t);
                if k == wk {
                    // ✅ シャンポン待ちでロンなら、この刻子は「ロンで完成」した可能性が高いので明刻扱いにする
                    let ron_completed = if win_type == WinType::RON {
                        Some(k)
                    } else {
                        None
                    };
                    return WaitInfo {
                        wait_type: WaitType::Shanpon,
                        ron_completed_triplet: ron_completed,
                    };
                }
            }
            Block::Kantsu(t, _, _, _) => {
                let k = TileKey::from_tile(t);
                if k == wk {
                    // 槓子で待つケースは通常ここには来ないが、双碰扱いでOK
                    return WaitInfo {
                        wait_type: WaitType::Shanpon,
                        ron_completed_triplet: None,
                    };
                }
            }
            _ => {}
        }
    }

    // fallback
    WaitInfo {
        wait_type: WaitType::Ryanmen,
        ron_completed_triplet: None,
    }
}

/// yaku.rs の is_pinfu と同等の「形チェック」を符側で行う（循環依存を避ける）
fn is_pinfu_shape(pattern: &HandPattern, round_wind: Wind, seat_wind: Wind, wt: WaitType) -> bool {
    // 1) 全部順子
    if !pattern
        .blocks
        .iter()
        .all(|b| matches!(b, Block::Shuntsu(_, _, _)))
    {
        return false;
    }

    // 2) 雀頭が役牌でない
    let Some(pk) = pattern.pair_key else {
        return false;
    };
    if pk.suit == Suit::Honor {
        let h = pk.honor.unwrap();
        let rw = wind_to_honor(round_wind);
        let sw = wind_to_honor(seat_wind);

        if matches!(h, Honor::White | Honor::Green | Honor::Red) {
            return false;
        }
        if h == rw || h == sw {
            return false;
        }
    }

    // 3) 両面待ち
    wt == WaitType::Ryanmen
}
