//! 最下段に出す評価値・最善手順・深さ。

use shakmaty::san::San;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, Position};

use crate::model::{Ply, Score};

/// 状態行の中身を組み立てる。注釈が無ければ空になる。
pub fn line(ply: Option<&Ply>) -> String {
    let Some(ply) = ply else {
        return String::new();
    };
    let Some(eval) = ply.eval else {
        return String::new();
    };
    let mut out = format!("{:>8}", render(eval.score));
    if let Some(pv) = pv_in_san(&ply.position, &ply.pv) {
        out.push_str(&format!("   PV {pv}"));
    }
    if let Some(depth) = eval.depth {
        out.push_str(&format!("   depth {depth}"));
    }
    out
}

fn render(score: Score) -> String {
    match score {
        Score::Cp(cp) => format!("{:+.2}", f64::from(cp) / 100.0),
        Score::Mate(n) => format!("#{n}"),
    }
}

/// UCI 表記の最善手順を SAN に直す。
///
/// 保持しているのは engine が返した UCI のままの形。
/// 読むのは SAN なので、局面を進めながら変換する。
/// 1 手でも解釈できなければ、そこで打ち切る。
fn pv_in_san(after: &Chess, pv: &[String]) -> Option<String> {
    if pv.is_empty() {
        return None;
    }
    let mut pos = after.clone();
    let mut sans = Vec::with_capacity(pv.len());
    for uci in pv {
        let mv = UciMove::from_ascii(uci.as_bytes())
            .ok()?
            .to_move(&pos)
            .ok()?;
        sans.push(San::from_move(&pos, mv).to_string());
        pos.play_unchecked(mv);
    }
    Some(sans.join(" "))
}
