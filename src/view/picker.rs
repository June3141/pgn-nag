//! 一覧から 1 つ選ぶ画面。
//!
//! 棋譜ファイルの一覧にも、ファイル内の対局の一覧にも使う。

use ratatui::Frame;

use super::centered;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// キー入力の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerAction {
    Continue,
    Choose(usize),
    Cancel,
}

/// 一覧に添える操作の案内。
///
/// 一覧画面はヘルプを持たないため、画面内に書く。
const LEGEND: &str = " ↑↓ 移動   Enter 決定   q 取消 ";

/// 選択中の一覧。
pub struct Picker {
    title: &'static str,
    items: Vec<String>,
    cursor: usize,
}

impl Picker {
    pub fn new(title: &'static str, items: Vec<String>) -> Self {
        Self {
            title,
            items,
            cursor: 0,
        }
    }

    pub fn selected(&self) -> usize {
        self.cursor
    }

    /// キー 1 つを適用する。
    ///
    /// 一覧では上下が行の移動になる。盤面と違って手が無いため、
    /// ADR-0009 が上下を空けておく理由が当てはまらない。
    pub fn apply(&mut self, key: KeyEvent) -> PickerAction {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return if key.code == KeyCode::Char('c') {
                PickerAction::Cancel
            } else {
                PickerAction::Continue
            };
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(self.items.len().saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Char('g') => self.cursor = 0,
            KeyCode::Char('G') => self.cursor = self.items.len().saturating_sub(1),
            KeyCode::Enter => return PickerAction::Choose(self.cursor),
            KeyCode::Char('q') | KeyCode::Esc => return PickerAction::Cancel,
            _ => {}
        }
        PickerAction::Continue
    }

    pub fn render(&self, frame: &mut Frame) {
        #[allow(clippy::cast_possible_truncation)]
        let height = (self.items.len() as u16)
            .saturating_add(2)
            .min(frame.area().height);
        let area = centered(frame.area(), frame.area().width.saturating_sub(4), height);
        let visible = area.height.saturating_sub(2) as usize;

        // 全件を渡すと、端末に収まらない一覧で選択中の行が画面外に出る。
        // 件数はディレクトリの中身しだいで、収まる保証が無い
        let offset = scroll_offset(self.cursor, self.items.len(), visible);
        let rows: Vec<Line<'static>> = self
            .items
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible)
            .map(|(i, item)| {
                let style = if i == self.cursor {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                Line::styled(format!("  {item}"), style)
            })
            .collect();

        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(rows).block(Block::bordered().title(self.title).title_bottom(LEGEND)),
            area,
        );
    }
}

/// 選択中の行が見えるように、先頭から何件送るか。
fn scroll_offset(cursor: usize, total: usize, visible: usize) -> usize {
    if total <= visible || visible == 0 {
        return 0;
    }
    cursor.saturating_sub(visible / 2).min(total - visible)
}

#[cfg(test)]
mod tests {
    use super::scroll_offset;

    #[test]
    fn keeps_the_selection_visible() {
        for total in [1usize, 5, 50] {
            for visible in [1usize, 3, 10] {
                for cursor in 0..total {
                    let offset = scroll_offset(cursor, total, visible);
                    assert!(
                        (offset..offset + visible).contains(&cursor),
                        "選択中の行が画面外: total={total} visible={visible} cursor={cursor}"
                    );
                }
            }
        }
    }
}
