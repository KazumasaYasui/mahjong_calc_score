use crate::WinType;

#[derive(Debug, Clone)]
pub struct PointBreakdown {
    pub total_points: u32,
    pub payments: Vec<String>, // e.g. ["親ロン: 12000", "本場: +300", ...]
}

/// Japanese Riichi Mahjong point calc (rounded up to 100).
/// This handles:
/// - base points from fu/han with mangan+ caps
/// - dealer/non-dealer
/// - ron/tsumo
/// - honba (300/100 per honba)
/// - kyotaku (1000 per stick added to winner)
pub fn calc_points(
    fu: u32,
    han: u32,
    yakuman: u32, // 0 for non-yakuman; if >0 treat as yakuman multiples
    win_type: WinType,
    dealer: bool,
    honba: u32,
    kyotaku: u32,
) -> PointBreakdown {
    let mut payments = vec![];

    let mut total = 0u32;

    if yakuman > 0 {
        // yakuman base: dealer 48000, non-dealer 32000 total (ron); tsumo split
        let base_total = if dealer { 48000 } else { 32000 };
        let base_total = base_total * yakuman;

        match win_type {
            WinType::RON => {
                total = base_total;
                payments.push(format!("役満{}倍 ロン: {}", yakuman, base_total));
            }
            WinType::TSUMO => {
                if dealer {
                    // each pays 16000 * yakuman
                    let each = 16000 * yakuman;
                    total = each * 3;
                    payments.push(format!("役満{}倍 親ツモ: {}オール", yakuman, each));
                } else {
                    // dealer pays 16000*y, others 8000*y
                    let from_dealer = 16000 * yakuman;
                    let from_other = 8000 * yakuman;
                    total = from_dealer + from_other * 2;
                    payments.push(format!(
                        "役満{}倍 子ツモ: 親{} / 子{}",
                        yakuman, from_dealer, from_other
                    ));
                }
            }
        }
    } else {
        // base points (no cap) = fu * 2^(han+2)
        let base = (fu as u64) * (1u64 << (han + 2));
        // apply caps:
        // mangan: 2000 base
        // haneman: 3000
        // baiman: 4000
        // sanbaiman: 6000
        // kazoe yakuman: 8000 (13+ han)
        let capped_base = if han >= 13 {
            8000
        } else if han >= 11 {
            6000
        } else if han >= 8 {
            4000
        } else if han >= 6 {
            3000
        } else if han == 5 || (han == 4 && fu >= 40) || (han == 3 && fu >= 70) {
            2000
        } else {
            // round base up? base itself is not rounded; payments are rounded
            base as u32
        };

        match win_type {
            WinType::RON => {
                let raw = if dealer {
                    capped_base * 6
                } else {
                    capped_base * 4
                };
                let ron = round_up_100(raw);
                total = ron;
                payments.push(format!("ロン: {}", ron));
            }
            WinType::TSUMO => {
                if dealer {
                    let each = round_up_100(capped_base * 2);
                    total = each * 3;
                    payments.push(format!("親ツモ: {}オール", each));
                } else {
                    let from_dealer = round_up_100(capped_base * 2);
                    let from_other = round_up_100(capped_base * 1);
                    total = from_dealer + from_other * 2;
                    payments.push(format!("子ツモ: 親{} / 子{}", from_dealer, from_other));
                }
            }
        }
    }

    // honba: ron +300/honba, tsumo each +100/honba (total +300/honba)
    if honba > 0 {
        let add = 300 * honba;
        total += add;
        payments.push(format!("本場 +{}", add));
    }

    // kyotaku: +1000 per stick
    if kyotaku > 0 {
        let add = 1000 * kyotaku;
        total += add;
        payments.push(format!("供託 +{}", add));
    }

    PointBreakdown {
        total_points: total,
        payments,
    }
}

fn round_up_100(x: u32) -> u32 {
    ((x + 99) / 100) * 100
}
