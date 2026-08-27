//! ヘルプの表示。

use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use super::centered;
use super::keys::BINDINGS;

/// キーの一覧を画面の中央に重ねて出す。
pub fn render(frame: &mut Frame) {
    let rows: Vec<Line<'static>> = BINDINGS
        .iter()
        .map(|b| Line::from(format!("  {:<8} {}", b.label, b.description)))
        .collect();

    #[allow(clippy::cast_possible_truncation)]
    let area = centered(frame.area(), 34, rows.len() as u16 + 2);
    // 下の画面を消さないと、幅の狭い行から地が透ける
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(rows).block(Block::bordered().title(" keys ")),
        area,
    );
}
