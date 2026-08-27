//! 注釈付き PGN の閲覧。
//!
//! 表示は現在の手からの純粋な関数として組み立てる (ADR-0011)。
//! widget が状態を持たないため、手の移動は添字の操作だけで完結する。

mod board;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Paragraph};
use shakmaty::{Chess, Position};

use crate::model::Game;

/// 閲覧中の状態。
///
/// 保持するのは対局と現在位置だけにする。
/// 表示に必要な値をここに増やすと、位置と表示が食い違う余地が生まれる。
pub struct Viewer {
    game: Game,
    /// 表示する局面。0 は開始局面、1 以降は `plies[cursor - 1]` を指した後。
    cursor: usize,
}

impl Viewer {
    pub fn new(game: Game) -> Self {
        Self { game, cursor: 0 }
    }

    /// 表示する局面。
    pub fn position(&self) -> Chess {
        match self.cursor.checked_sub(1) {
            None => Chess::default(),
            Some(i) => self.game.plies[i].position.clone(),
        }
    }

    pub fn next(&mut self) {
        self.cursor = (self.cursor + 1).min(self.game.plies.len());
    }

    pub fn prev(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn first(&mut self) {
        self.cursor = 0;
    }

    pub fn last(&mut self) {
        self.cursor = self.game.plies.len();
    }

    /// 見出しに出す対局者と結果。
    fn title(&self) -> String {
        let tag = |name: &str| {
            self.game
                .tags
                .iter()
                .find(|(k, _)| k == name)
                .map_or("?", |(_, v)| v.as_str())
                .to_owned()
        };
        format!(
            " {} vs {}  ·  {}  ·  {}/{} ",
            tag("White"),
            tag("Black"),
            self.game.outcome,
            self.cursor,
            self.game.plies.len()
        )
    }

    /// 現在の状態を描く。
    pub fn render(&self, frame: &mut Frame) {
        let [area] = Layout::vertical([Constraint::Min(0)]).areas(frame.area());
        let position = self.position();
        let widget = Paragraph::new(board::lines(position.board()))
            .block(Block::bordered().title(self.title()));
        frame.render_widget(widget, area);
    }
}

/// 端末を初期化してイベントループを回す。
///
/// 端末の後始末は ratatui の `restore` が行う。
/// panic しても端末が壊れたままにならないよう、`init` が hook を差し込む。
pub fn run(mut viewer: Viewer) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut viewer);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, viewer: &mut Viewer) -> std::io::Result<()> {
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
    loop {
        terminal.draw(|frame| viewer.render(frame))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        // 押下だけを見る。Windows では離したときにも届く
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Right | KeyCode::Char('l') => viewer.next(),
            KeyCode::Left | KeyCode::Char('h') => viewer.prev(),
            KeyCode::Char('g') => viewer.first(),
            KeyCode::Char('G') => viewer.last(),
            _ => {}
        }
    }
}
