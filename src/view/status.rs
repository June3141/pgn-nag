//! 最下段に出す評価値・最善手順・深さ。

use shakmaty::Chess;
use shakmaty::san::SanPlus;
use shakmaty::uci::UciMove;

use crate::model::{Ply, move_marker};

/// 状態行の中身を組み立てる。注釈が無ければ空になる。
///
/// `index` は現在の手の添字。最善手順の手番号を続ける位置を決める。
pub fn line(ply: Option<&Ply>, index: usize) -> String {
    let Some(ply) = ply else {
        return String::new();
    };
    let Some(eval) = ply.eval else {
        return String::new();
    };
    let mut out = format!("{:>8}", eval.score.render());
    // 深さは可変長の最善手順より前に置く。
    // 後ろに置くと、幅の狭い端末で桁の途中から切れて別の値に読める
    if let Some(depth) = eval.depth {
        out.push_str(&format!("   depth {depth}"));
    }
    if let Some(pv) = pv_in_san(&ply.position, &ply.pv, index) {
        out.push_str(&format!("   PV {pv}"));
    }
    out
}

/// UCI 表記の最善手順を SAN に直す。
///
/// 保持しているのは engine が返した UCI のままの形。
/// 読むのは SAN なので、局面を進めながら変換する。
/// 王手と詰みの記号を落とすと、詰みの評価値の隣に詰まない手順が並ぶため
/// `SanPlus` を使う。
/// 1 手でも解釈できなければ、そこで打ち切る。
fn pv_in_san(after: &Chess, pv: &[String], index: usize) -> Option<String> {
    if pv.is_empty() {
        return None;
    }
    let mut pos = after.clone();
    let mut out = String::new();
    for (i, uci) in pv.iter().enumerate() {
        let mv = UciMove::from_ascii(uci.as_bytes())
            .ok()?
            .to_move(&pos)
            .ok()?;
        let ply = index + i + 1;
        // 先頭と白番の手にだけ番号を付ける。
        // 番号が無いと、最善手順の先頭がどちらの手番か画面から分からない
        if i == 0 || ply.is_multiple_of(2) {
            out.push_str(&move_marker(ply));
            out.push(' ');
        }
        out.push_str(&SanPlus::from_move_and_play_unchecked(&mut pos, mv).to_string());
        out.push(' ');
    }
    Some(out.trim_end().to_owned())
}
