//! 一覧から 1 つ選ぶ画面。
//!
//! 棋譜ファイルの一覧にも、ファイル内の対局の一覧にも使う。

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
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
        let rows: Vec<Line<'static>> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.cursor {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                Line::styled(format!("  {item}"), style)
            })
            .collect();

        #[allow(clippy::cast_possible_truncation)]
        let height = (rows.len() as u16)
            .saturating_add(2)
            .min(frame.area().height);
        let area = centered(frame.area(), frame.area().width.saturating_sub(4), height);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(rows).block(Block::bordered().title(self.title)),
            area,
        );
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    cell
}
