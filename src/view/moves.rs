//! 手順リストの描画。

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::model::{Eval, Ply, Score};

/// 手順を 1 手 1 行で並べる。
///
/// `cursor` は 0 が開始局面、n が `plies[n - 1]` を指した後を表す。
/// 現在の手を反転で示し、その手が入る位置まで送る。
pub fn lines(plies: &[Ply], cursor: usize, height: usize) -> Vec<Line<'static>> {
    let offset = scroll_offset(cursor, plies.len(), height);
    plies
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, ply)| {
            let style = if i + 1 == cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Line::from(Span::styled(row(i, ply), style))
        })
        .collect()
}

/// 現在の手が見えるように、先頭から何手ぶん送るか。
fn scroll_offset(cursor: usize, total: usize, height: usize) -> usize {
    if total <= height {
        return 0;
    }
    // 現在の手を中央に置く。端では詰める
    let half = height / 2;
    cursor.saturating_sub(half).min(total - height)
}

/// `1.  d4    +0.32` の形に整える。
fn row(index: usize, ply: &Ply) -> String {
    let number = index / 2 + 1;
    let marker = if index.is_multiple_of(2) {
        format!("{number}.")
    } else {
        format!("{number}...")
    };
    format!(
        "{:<6} {:<7} {}",
        marker,
        ply.san,
        ply.eval.map(render_eval).unwrap_or_default()
    )
}

/// 評価値を表示用の文字列にする。
///
/// 注釈が無い手は空欄にする。
/// 0.00 と書くと、互角と注釈なしが見分けられない。
fn render_eval(eval: Eval) -> String {
    match eval.score {
        Score::Cp(cp) => format!("{:+.2}", f64::from(cp) / 100.0),
        Score::Mate(n) => format!("#{n}"),
    }
}
