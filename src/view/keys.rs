//! キーの割り当て。
//!
//! 表とヘルプ表示を同じ定義から作る。
//! 別々に持つと、片方だけ直る事故が起きる (ADR-0009)。

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// キーに割り当てる操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Next,
    Prev,
    First,
    Last,
    ToggleHelp,
    CloseHelp,
    Quit,
}

/// 1 つの割り当て。
pub struct Binding {
    /// 受け付けるキー。
    pub codes: &'static [KeyCode],
    /// ヘルプに出す表記。
    pub label: &'static str,
    /// ヘルプに出す説明。
    pub description: &'static str,
    /// 対応する操作。
    pub command: Command,
}

/// 割り当ての一覧。ヘルプはここから作る。
pub const BINDINGS: &[Binding] = &[
    Binding {
        codes: &[KeyCode::Right, KeyCode::Char('l')],
        label: "→ / l",
        description: "次の手",
        command: Command::Next,
    },
    Binding {
        codes: &[KeyCode::Left, KeyCode::Char('h')],
        label: "← / h",
        description: "前の手",
        command: Command::Prev,
    },
    Binding {
        codes: &[KeyCode::Char('g')],
        label: "g",
        description: "開始局面へ",
        command: Command::First,
    },
    Binding {
        codes: &[KeyCode::Char('G')],
        label: "G",
        description: "終局へ",
        command: Command::Last,
    },
    Binding {
        codes: &[KeyCode::Char('?')],
        label: "?",
        description: "ヘルプの開閉",
        command: Command::ToggleHelp,
    },
    Binding {
        codes: &[KeyCode::Esc],
        label: "Esc",
        description: "ヘルプを閉じる",
        command: Command::CloseHelp,
    },
    Binding {
        codes: &[KeyCode::Char('q')],
        label: "q",
        description: "終了",
        command: Command::Quit,
    },
];

/// キー入力に対応する操作を返す。
///
/// 上下のキーは悪手を辿る移動のために空けてある (ADR-0009)。
pub fn command_for(key: KeyEvent) -> Option<Command> {
    // raw mode では ISIG が落ちて Ctrl+C が SIGINT にならない。
    // 受けないと、描画が壊れた状況で脱出手段が q だけになる
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return (key.code == KeyCode::Char('c')).then_some(Command::Quit);
    }
    BINDINGS
        .iter()
        .find(|b| b.codes.contains(&key.code))
        .map(|b| b.command)
}
