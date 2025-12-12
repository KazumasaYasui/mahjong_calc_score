use crate::decompose::{Block, HandPattern};
use crate::fu::WaitType;
use crate::special::SpecialHand;
use crate::tile::{Honor, Suit, Tile, TileKey};
use crate::{Riichi, WinType, Wind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct YakuResult {
    pub yakuman: u32,
    pub han: u32,
    pub yaku: Vec<String>,
}

pub fn eval_special_yaku(sp: SpecialHand) -> YakuResult {
    match sp {
        SpecialHand::Chiitoitsu => YakuResult {
            yakuman: 0,
            han: 2,
            yaku: vec!["七対子".into()],
        },
        SpecialHand::Kokushi => YakuResult {
            yakuman: 1,
            han: 0,
            yaku: vec!["国士無双".into()],
        },
        SpecialHand::Kokushi13 => YakuResult {
            yakuman: 2,
            han: 0,
            yaku: vec!["国士無双十三面待ち".into()],
        },
    }
}

pub fn eval_yaku_standard(
    pattern: &HandPattern,
    tiles14: &[Tile],
    win_tile: Tile,
    win_type: WinType,
    round_wind: Wind,
    seat_wind: Wind,
    riichi: Riichi,
    ippatsu: bool,
    rinshan: bool,
    chankan: bool,
    haitei: bool,
    houtei: bool,
) -> YakuResult {
    let mut yakuman = 0;
    let mut han = 0;
    let mut yaku = vec![];

    let menzen = pattern.menzen;

    // ===== ルール差分（今は固定・後で ScoreRequest に寄せやすい） =====
    let allow_kuitan = true;

    // ===== 状況役（基本） =====
    if riichi != Riichi::NONE {
        if riichi == Riichi::DOUBLE {
            yaku.push("ダブル立直".into());
            han += 2;
        } else {
            yaku.push("立直".into());
            han += 1;
        }
    }
    if ippatsu {
        yaku.push("一発".into());
        han += 1;
    }
    if rinshan {
        yaku.push("嶺上開花".into());
        han += 1;
    }
    if chankan {
        yaku.push("搶槓".into());
        han += 1;
    }
    if haitei {
        yaku.push("海底摸月".into());
        han += 1;
    }
    if houtei {
        yaku.push("河底撈魚".into());
        han += 1;
    }

    // ===== 門前清自摸和 =====
    if menzen && win_type == WinType::TSUMO {
        yaku.push("門前清自摸和".into());
        han += 1;
    }

    // ===== 断么九（喰いタン許可なら副露でもOK） =====
    if is_tanyao(tiles14, menzen, allow_kuitan) {
        yaku.push("断么九".into());
        han += 1;
    }

    // ===== 役牌（刻子/槓子） =====
    add_yakuhai(&mut yaku, &mut han, pattern, round_wind, seat_wind);

    // ===== 対々和 =====
    if is_toitoi(pattern) {
        yaku.push("対々和".into());
        han += 2;
    }

    // ===== 三暗刻（ロン補正込み） =====
    if is_sanankou(pattern, win_tile, win_type) {
        yaku.push("三暗刻".into());
        han += 2;
    }

    // ===== 混一色 / 清一色（喰い下がり） =====
    if let Some((name, h)) = honitsu_chinitsu(tiles14, menzen) {
        yaku.push(name);
        han += h;
    }

    // ===== 一気通貫 / 三色同順（喰い下がり） =====
    if let Some(h) = ittsuu(pattern, menzen) {
        yaku.push("一気通貫".into());
        han += h;
    }
    if let Some(h) = sanshoku_doujun(pattern, menzen) {
        yaku.push("三色同順".into());
        han += h;
    }

    // ===== 混全帯么九 / 純全帯么九（喰い下がり） =====
    if let Some((name, h)) = chanta_junchan(pattern, menzen) {
        yaku.push(name);
        han += h;
    }

    // ===== 三色同刻 =====
    if is_sanshoku_doukou(pattern) {
        yaku.push("三色同刻".into());
        han += 2;
    }

    // ===== 小三元 =====
    if is_shousangen(pattern) {
        yaku.push("小三元".into());
        han += 2;
    }

    // ===== 混老頭 =====
    if is_honroutou(pattern) {
        yaku.push("混老頭".into());
        han += 2;
    }

    // ===== 三槓子 =====
    if is_sankantsu(pattern) {
        yaku.push("三槓子".into());
        han += 2;
    }

    // ===== 平和（門前限定） =====
    // 1) 全面子が順子 2) 雀頭が役牌でない 3) 待ちが両面
    let wt = crate::fu::detect_wait_type(pattern, win_tile);
    if menzen && is_pinfu(pattern, round_wind, seat_wind, wt) {
        yaku.push("平和".into());
        han += 1;
    }

    // ===== 一盃口 / 二盃口（門前限定） =====
    if menzen {
        if let Some((name, h)) = iipeikou_ryanpeikou(pattern) {
            yaku.push(name);
            han += h;
        }
    }

    // ===== 役満（標準形側） =====
    // ※ yakuman > 0 の場合、通常役は基本的に無視する運用（一般的）
    if let Some((ym, names)) =
        eval_yakuman_standard(pattern, win_tile, win_type, round_wind, seat_wind)
    {
        yakuman += ym;
        for n in names {
            yaku.push(n);
        }
        // 通常役を無視
        han = 0;
    }

    YakuResult { yakuman, han, yaku }
}

// =====================
// 通常役：ヘルパー群
// =====================

fn is_tanyao(tiles14: &[Tile], menzen: bool, allow_kuitan: bool) -> bool {
    if !allow_kuitan && !menzen {
        return false;
    }
    tiles14.iter().all(|t| {
        if t.suit == Suit::Honor {
            return false;
        }
        t.num != 1 && t.num != 9
    })
}

fn add_yakuhai(
    yaku: &mut Vec<String>,
    han: &mut u32,
    pattern: &HandPattern,
    round_wind: Wind,
    seat_wind: Wind,
) {
    let rw = wind_to_honor(round_wind);
    let sw = wind_to_honor(seat_wind);

    for b in &pattern.blocks {
        let key_opt = match b {
            Block::Koutsu(t, _, _) => Some(TileKey::from_tile(t)),
            Block::Kantsu(t, _, _, _) => Some(TileKey::from_tile(t)),
            _ => None,
        };
        let Some(k) = key_opt else {
            continue;
        };

        if k.suit != Suit::Honor {
            continue;
        }
        let h = k.honor.unwrap();

        // 三元牌
        match h {
            Honor::White => {
                yaku.push("役牌 白".into());
                *han += 1;
            }
            Honor::Green => {
                yaku.push("役牌 發".into());
                *han += 1;
            }
            Honor::Red => {
                yaku.push("役牌 中".into());
                *han += 1;
            }
            _ => {}
        }

        // 場風 / 自風（連風牌なら両方乗る）
        if h == rw {
            yaku.push("役牌 場風".into());
            *han += 1;
        }
        if h == sw {
            yaku.push("役牌 自風".into());
            *han += 1;
        }
    }
}

fn wind_to_honor(w: Wind) -> Honor {
    match w {
        Wind::E => Honor::East,
        Wind::S => Honor::South,
        Wind::W => Honor::West,
        Wind::N => Honor::North,
    }
}

fn is_toitoi(pattern: &HandPattern) -> bool {
    pattern
        .blocks
        .iter()
        .all(|b| matches!(b, Block::Koutsu(_, _, _) | Block::Kantsu(_, _, _, _)))
}

fn is_sanankou(pattern: &HandPattern, win_tile: Tile, win_type: WinType) -> bool {
    let wi = crate::fu::detect_wait_info(pattern, win_tile, win_type);

    let mut concealed = 0;

    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };

        // 開いている刻子/槓子は暗刻扱いしない
        let open = pattern
            .open_info
            .as_ref()
            .map(|oi| oi.is_open_triplet(k) || oi.is_open_kan(k))
            .unwrap_or(false);
        if open {
            continue;
        }

        // ✅ シャンポンロンで完成した刻子は暗刻扱いしない
        if wi.ron_completed_triplet == Some(k) {
            continue;
        }

        concealed += 1;
    }

    concealed >= 3
}

fn honitsu_chinitsu(tiles14: &[Tile], menzen: bool) -> Option<(String, u32)> {
    let mut suit_seen = None;
    let mut has_honor = false;

    for t in tiles14 {
        if t.suit == Suit::Honor {
            has_honor = true;
            continue;
        }
        suit_seen = match suit_seen {
            None => Some(t.suit),
            Some(s) if s == t.suit => Some(s),
            Some(_) => return None,
        };
    }

    let Some(_) = suit_seen else {
        return None;
    };

    if has_honor {
        // 混一色（門前3 / 鳴き2）
        Some(("混一色".into(), if menzen { 3 } else { 2 }))
    } else {
        // 清一色（門前6 / 鳴き5）
        Some(("清一色".into(), if menzen { 6 } else { 5 }))
    }
}

fn ittsuu(pattern: &HandPattern, menzen: bool) -> Option<u32> {
    // 同一色で 123 456 789 の順子が揃う
    for &s in &[Suit::Man, Suit::Pin, Suit::Sou] {
        let mut has123 = false;
        let mut has456 = false;
        let mut has789 = false;

        for b in &pattern.blocks {
            if let Block::Shuntsu(a, _, _) = b {
                if a.suit != s {
                    continue;
                }
                match a.num {
                    1 => has123 = true,
                    4 => has456 = true,
                    7 => has789 = true,
                    _ => {}
                }
            }
        }
        if has123 && has456 && has789 {
            return Some(if menzen { 2 } else { 1 });
        }
    }
    None
}

fn sanshoku_doujun(pattern: &HandPattern, menzen: bool) -> Option<u32> {
    // 同じ数字開始の順子が3色揃う
    for start in 1..=7 {
        let mut man = false;
        let mut pin = false;
        let mut sou = false;

        for b in &pattern.blocks {
            if let Block::Shuntsu(a, _, _) = b {
                if a.num != start {
                    continue;
                }
                match a.suit {
                    Suit::Man => man = true,
                    Suit::Pin => pin = true,
                    Suit::Sou => sou = true,
                    _ => {}
                }
            }
        }

        if man && pin && sou {
            return Some(if menzen { 2 } else { 1 });
        }
    }
    None
}

fn is_pinfu(pattern: &HandPattern, round_wind: Wind, seat_wind: Wind, wt: WaitType) -> bool {
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

// ===== 一盃口/二盃口（門前限定） =====
fn iipeikou_ryanpeikou(pattern: &HandPattern) -> Option<(String, u32)> {
    // 順子の (suit, start) を数える
    let mut m = HashMap::<(Suit, u8), u8>::new();
    for b in &pattern.blocks {
        if let Block::Shuntsu(a, _, _) = b {
            if a.suit == Suit::Honor {
                continue;
            }
            *m.entry((a.suit, a.num)).or_insert(0) += 1;
        }
    }

    let pairs = m.values().filter(|&&c| c >= 2).count();
    match pairs {
        2 => Some(("二盃口".into(), 3)),
        1 => Some(("一盃口".into(), 1)),
        _ => None,
    }
}

// ===== 混全帯么九/純全帯么九（喰い下がり） =====
fn is_terminal_or_honor_key(k: TileKey) -> bool {
    if k.suit == Suit::Honor {
        return true;
    }
    k.num == 1 || k.num == 9
}

fn chanta_junchan(pattern: &HandPattern, menzen: bool) -> Option<(String, u32)> {
    // 各面子・雀頭が「么九を含む」(チャンタ) / 「字牌を含まない」なら純チャン
    let mut all_have_terminal_or_honor = true;
    let mut any_honor = false;

    for b in &pattern.blocks {
        match b {
            Block::Shuntsu(a, _, c) => {
                // 順子は 1-2-3 or 7-8-9 でなければチャンタ系不可
                if a.suit == Suit::Honor {
                    return None;
                }
                let start = a.num;
                let end = c.num;
                if !((start == 1 && end == 3) || (start == 7 && end == 9)) {
                    all_have_terminal_or_honor = false;
                }
            }
            Block::Koutsu(t, _, _) | Block::Kantsu(t, _, _, _) => {
                let k = TileKey::from_tile(t);
                if !is_terminal_or_honor_key(k) {
                    all_have_terminal_or_honor = false;
                }
                if k.suit == Suit::Honor {
                    any_honor = true;
                }
            }
            _ => {}
        }
    }

    // 雀頭
    let Some(pk) = pattern.pair_key else {
        return None;
    };
    if !is_terminal_or_honor_key(pk) {
        all_have_terminal_or_honor = false;
    }
    if pk.suit == Suit::Honor {
        any_honor = true;
    }

    if !all_have_terminal_or_honor {
        return None;
    }

    // 純チャン：字牌を含まない（= honor なし）
    if !any_honor {
        return Some(("純全帯么九".into(), if menzen { 3 } else { 2 }));
    }

    Some(("混全帯么九".into(), if menzen { 2 } else { 1 }))
}

// ===== 三色同刻 =====
fn is_sanshoku_doukou(pattern: &HandPattern) -> bool {
    // 同じ数の刻子が 3スート揃う
    let mut nums: HashMap<u8, (bool, bool, bool)> = HashMap::new(); // (man,pin,sou)

    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };
        if k.suit == Suit::Honor {
            continue;
        }

        let entry = nums.entry(k.num).or_insert((false, false, false));
        match k.suit {
            Suit::Man => entry.0 = true,
            Suit::Pin => entry.1 = true,
            Suit::Sou => entry.2 = true,
            _ => {}
        }
    }

    nums.values().any(|(m, p, s)| *m && *p && *s)
}

// ===== 小三元 =====
fn is_shousangen(pattern: &HandPattern) -> bool {
    let mut dragon_triplets = 0;
    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };
        if k.suit == Suit::Honor {
            if matches!(k.honor.unwrap(), Honor::White | Honor::Green | Honor::Red) {
                dragon_triplets += 1;
            }
        }
    }

    let pair_is_dragon = pattern
        .pair_key
        .map(|pk| {
            pk.suit == Suit::Honor
                && matches!(pk.honor.unwrap(), Honor::White | Honor::Green | Honor::Red)
        })
        .unwrap_or(false);

    dragon_triplets == 2 && pair_is_dragon
}

// ===== 混老頭 =====
fn is_honroutou(pattern: &HandPattern) -> bool {
    // 么九字牌のみ + 順子が存在しない
    if pattern
        .blocks
        .iter()
        .any(|b| matches!(b, Block::Shuntsu(_, _, _)))
    {
        return false;
    }

    // ブロック側
    for b in &pattern.blocks {
        match b {
            Block::Koutsu(t, _, _) | Block::Kantsu(t, _, _, _) => {
                let k = TileKey::from_tile(t);
                if !is_terminal_or_honor_key(k) {
                    return false;
                }
            }
            _ => {}
        }
    }

    // 雀頭
    pattern
        .pair_key
        .map(|pk| is_terminal_or_honor_key(pk))
        .unwrap_or(false)
}

// ===== 三槓子 =====
fn is_sankantsu(pattern: &HandPattern) -> bool {
    pattern
        .blocks
        .iter()
        .filter(|b| matches!(b, Block::Kantsu(_, _, _, _)))
        .count()
        == 3
}

// =====================
// 役満（標準形）
// =====================

fn eval_yakuman_standard(
    pattern: &HandPattern,
    win_tile: Tile,
    win_type: WinType,
    _round_wind: Wind,
    _seat_wind: Wind,
) -> Option<(u32, Vec<String>)> {
    let mut ym = 0;
    let mut names: Vec<String> = vec![];

    // 大三元
    if is_daisangen(pattern) {
        ym += 1;
        names.push("大三元".into());
    }

    // 大四喜 / 小四喜
    if let Some(n) = suuushi(pattern) {
        ym += 1;
        names.push(n);
    }

    // 字一色
    if is_tsuuiisou(pattern) {
        ym += 1;
        names.push("字一色".into());
    }

    // 四槓子
    if is_suukantsu(pattern) {
        ym += 1;
        names.push("四槓子".into());
    }

    // 四暗刻 / 四暗刻単騎（ダブル役満扱い）
    if let Some((add, n)) = suuankou(pattern, win_tile, win_type) {
        ym += add;
        names.push(n);
    }

    if ym == 0 {
        None
    } else {
        Some((ym, names))
    }
}

fn is_daisangen(pattern: &HandPattern) -> bool {
    let mut dragons = 0;
    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };
        if k.suit == Suit::Honor {
            if matches!(k.honor.unwrap(), Honor::White | Honor::Green | Honor::Red) {
                dragons += 1;
            }
        }
    }
    dragons == 3
}

fn suuushi(pattern: &HandPattern) -> Option<String> {
    let mut wind_triplets = 0;
    let mut wind_pair = false;

    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };
        if k.suit == Suit::Honor {
            if matches!(
                k.honor.unwrap(),
                Honor::East | Honor::South | Honor::West | Honor::North
            ) {
                wind_triplets += 1;
            }
        }
    }

    if let Some(pk) = pattern.pair_key {
        if pk.suit == Suit::Honor {
            wind_pair = matches!(
                pk.honor.unwrap(),
                Honor::East | Honor::South | Honor::West | Honor::North
            );
        }
    }

    match (wind_triplets, wind_pair) {
        (4, _) => Some("大四喜".into()),
        (3, true) => Some("小四喜".into()),
        _ => None,
    }
}

fn is_tsuuiisou(pattern: &HandPattern) -> bool {
    for b in &pattern.blocks {
        match b {
            Block::Shuntsu(a, _, _) => {
                if a.suit != Suit::Honor {
                    return false;
                }
            }
            Block::Koutsu(t, _, _) => {
                if t.suit != Suit::Honor {
                    return false;
                }
            }
            Block::Kantsu(t, _, _, _) => {
                if t.suit != Suit::Honor {
                    return false;
                }
            }
            _ => {}
        }
    }
    if let Some(pk) = pattern.pair_key {
        return pk.suit == Suit::Honor;
    }
    false
}

fn is_suukantsu(pattern: &HandPattern) -> bool {
    pattern
        .blocks
        .iter()
        .filter(|b| matches!(b, Block::Kantsu(_, _, _, _)))
        .count()
        == 4
}

fn suuankou(pattern: &HandPattern, win_tile: Tile, win_type: WinType) -> Option<(u32, String)> {
    let wk = TileKey::from_tile(&win_tile);

    // ✅ シャンポン待ちロンで「どの刻子がロンで完成したか」を fu 側ロジックで特定
    let wi = crate::fu::detect_wait_info(pattern, win_tile, win_type);

    let mut concealed_triplet_like = 0;
    for b in &pattern.blocks {
        let k = match b {
            Block::Koutsu(t, _, _) => TileKey::from_tile(t),
            Block::Kantsu(t, _, _, _) => TileKey::from_tile(t),
            _ => continue,
        };

        let open = pattern
            .open_info
            .as_ref()
            .map(|oi| oi.is_open_triplet(k) || oi.is_open_kan(k))
            .unwrap_or(false);
        if open {
            continue;
        }

        // ✅ シャンポン待ちロンで完成した刻子は暗刻扱いしない
        if wi.ron_completed_triplet == Some(k) {
            continue;
        }

        concealed_triplet_like += 1;
    }

    if concealed_triplet_like != 4 {
        return None;
    }

    // 単騎待ち＝和了牌が雀頭
    let tanki = pattern.pair_key.map(|pk| pk == wk).unwrap_or(false);

    if tanki {
        Some((2, "四暗刻単騎".into()))
    } else {
        Some((1, "四暗刻".into()))
    }
}
