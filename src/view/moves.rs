//! 手順リストの描画。

use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::model::{Ply, move_marker};

/// 評価値を欄に収めるのに要する桁数。`-327.68` と `#-100` が最長になる。
const EVAL_WIDTH: usize = 7;
/// 手番号と SAN に要する桁数。`49...` と `exd8=Q+` が最長になる。
const PREFIX_WIDTH: usize = 5 + 1 + 7;

/// 手順を 1 手 1 行で並べる。
///
/// `cursor` は 0 が開始局面、n が `plies[n - 1]` を指した後を表す。
/// 現在の手を反転で示し、その手が画面に入る位置まで送る。
pub fn lines(plies: &[Ply], cursor: usize, height: usize, width: usize) -> Vec<Line<'static>> {
    let offset = scroll_offset(cursor, plies.len(), height);
    // 欄に収まらない評価値を出すと、桁の途中で切れて別の値に読める
    let with_eval = width >= PREFIX_WIDTH + 1 + EVAL_WIDTH;
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
            // 幅まで詰めないと、反転の帯が行の内容の長さで凸凹になる
            Line::styled(format!("{:<width$}", row(i, ply, with_eval)), style)
        })
        .collect()
}

/// 現在の手が見えるように、先頭から何手ぶん送るか。
pub(crate) fn scroll_offset(cursor: usize, total: usize, height: usize) -> usize {
    if total <= height || height == 0 {
        return 0;
    }
    // 送る基準は現在の手の添字。cursor から引くと height が 1 のとき画面外に出る
    let current = cursor.saturating_sub(1);
    // 現在の手を中央に置く。末尾では詰める
    current.saturating_sub(height / 2).min(total - height)
}

/// `1.    d4      +0.32` の形に整える。
fn row(index: usize, ply: &Ply, with_eval: bool) -> String {
    let head = format!("{:<5} {:<7}", move_marker(index), ply.san);
    if !with_eval {
        return head;
    }
    format!(
        "{head} {:>EVAL_WIDTH$}",
        // 注釈が無い手は空欄にする。0.00 と書くと互角と見分けられない
        ply.eval.map(|e| e.score.render()).unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::scroll_offset;

    #[test]
    fn keeps_the_current_ply_visible() {
        for total in [1usize, 2, 11, 12, 13, 98] {
            for height in [1usize, 2, 11, 12, 13] {
                for cursor in 0..=total {
                    let offset = scroll_offset(cursor, total, height);
                    assert!(
                        offset + height <= total.max(height),
                        "{total} {height} {cursor}"
                    );
                    if cursor == 0 {
                        continue;
                    }
                    let current = cursor - 1;
                    assert!(
                        (offset..offset + height).contains(&current),
                        "現在の手が画面外: total={total} height={height} cursor={cursor} offset={offset}"
                    );
                }
            }
        }
    }

    #[test]
    fn clamps_at_the_tail() {
        // 末尾で詰めないと、下半分が空欄のまま送られる
        assert_eq!(scroll_offset(98, 98, 12), 98 - 12);
        assert_eq!(scroll_offset(13, 13, 12), 1);
    }

    #[test]
    fn does_not_scroll_when_everything_fits() {
        assert_eq!(scroll_offset(12, 12, 12), 0);
        assert_eq!(scroll_offset(1, 5, 12), 0);
    }
}
