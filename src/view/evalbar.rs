//! 盤の右に置く評価バー。

use crate::model::{Eval, Score};

/// バーの高さ。盤の 8 段に合わせる。
pub const HEIGHT: usize = 8;

/// 白から見た優劣を、下から埋まる縦棒として表す。
///
/// 横棒ではなく縦棒にするのは、盤の高さと揃い、白優勢と黒優勢の向きが
/// 直感に合うため。
/// 注釈が無い局面では空にする。ここで中立を描くと、互角と注釈なしが
/// 見分けられない。
pub fn column(eval: Option<Eval>) -> Vec<char> {
    let Some(eval) = eval else {
        return vec![' '; HEIGHT];
    };
    let filled = filled_rows(eval.score);
    (0..HEIGHT)
        .map(|i| {
            // 先頭が上の段になるため、下から数え直す
            if HEIGHT - i <= filled { '█' } else { '░' }
        })
        .collect()
}

/// 白側が占める段数。
///
/// 4 段を互角とし、±4 ポーンで振り切る。
/// 実戦の評価値はこの範囲に収まることが多く、外側の差は勝敗の判断を変えない。
fn filled_rows(score: Score) -> usize {
    const FULL_SCALE_CP: f64 = 400.0;
    let ratio = match score {
        Score::Mate(n) if n > 0 => 1.0,
        Score::Mate(_) => 0.0,
        Score::Cp(cp) => 0.5 + f64::from(cp) / (FULL_SCALE_CP * 2.0),
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rows = (ratio.clamp(0.0, 1.0) * HEIGHT as f64).round() as usize;
    rows.min(HEIGHT)
}
