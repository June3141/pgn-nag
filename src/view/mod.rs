//! 注釈付き PGN の閲覧。
//!
//! 表示は現在の手からの純粋な関数として組み立てる (ADR-0011)。
//! widget が状態を持たないため、手の移動は添字の操作だけで完結する。

mod board;
mod evalbar;
mod help;
pub mod keys;
mod moves;
pub mod picker;
mod status;

pub use help::render as render_help;

/// 画面の中央に領域を取る。ヘルプと一覧の双方で使う。
pub(crate) fn centered(
    area: ratatui::layout::Rect,
    width: u16,
    height: u16,
) -> ratatui::layout::Rect {
    use ratatui::layout::Flex;
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    cell
}

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

    /// 表示している局面に至った手。開始局面では None になる。
    fn current_ply(&self) -> Option<&crate::model::Ply> {
        self.cursor
            .checked_sub(1)
            .and_then(|i| self.game.plies.get(i))
    }

    /// 表示している局面の評価。
    fn eval(&self) -> Option<crate::model::Eval> {
        self.current_ply().and_then(|p| p.eval)
    }

    /// 表示する局面。
    fn position(&self) -> Chess {
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

    /// 盤の見出しに出す対局者。
    fn players(&self) -> String {
        format!(" {} vs {} ", self.game.tag("White"), self.game.tag("Black"))
    }

    /// 手順リストの見出しに出す現在位置と結果。
    ///
    /// 盤の枠は対局者名で埋まるため、こちらに置く。
    fn progress(&self) -> String {
        format!(
            " {}/{}  ·  {} ",
            self.cursor,
            self.game.plies.len(),
            self.game.outcome
        )
    }

    /// 現在の状態を描く。
    pub fn render(&self, frame: &mut Frame) {
        // 盤は枠込みで 11 行要る。状態行を優先すると低い端末で盤が削られるため、
        // 盤の側を Min にして先に確保する
        let [top, bottom] =
            Layout::vertical([Constraint::Min(11), Constraint::Length(3)]).areas(frame.area());

        // 盤は段番号込みで固定幅。残りを手順リストに渡す。
        // 右を Min にすると solver がそちらを優先し、狭い端末で盤のほうが削られる
        let [left, right] =
            Layout::horizontal([Constraint::Length(24), Constraint::Fill(1)]).areas(top);

        let position = self.position();
        let board = Paragraph::new(board::lines(
            position.board(),
            &evalbar::column(self.eval()),
        ))
        .block(Block::bordered().title(self.players()));
        frame.render_widget(board, left);

        let inner_height = right.height.saturating_sub(2) as usize;
        let inner_width = right.width.saturating_sub(2) as usize;
        let list = Paragraph::new(moves::lines(
            &self.game.plies,
            self.cursor,
            inner_height,
            inner_width,
        ))
        .block(Block::bordered().title(self.progress()));
        frame.render_widget(list, right);

        // Min を優先する solver は Length(3) を割り込む。
        // 枠に足りない高さで描くと、底辺の無い枠が最下段に残る
        if bottom.height >= 3 {
            let line = Paragraph::new(status::line(
                self.current_ply(),
                self.cursor.saturating_sub(1),
            ))
            .block(Block::bordered());
            frame.render_widget(line, bottom);
        }
    }
}

/// キー入力の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}

/// キー 1 つを状態に適用する。
///
/// 入力の読み取りと分けておく。
/// 同じ関数に抱えると、キーの割り当てをテストから確認できない。
pub fn apply_key(viewer: &mut Viewer, key: ratatui::crossterm::event::KeyEvent) -> Action {
    use keys::Command;

    let Some(command) = keys::command_for(key) else {
        return Action::Continue;
    };
    match command {
        Command::Quit => return Action::Quit,
        Command::Next => viewer.next(),
        Command::Prev => viewer.prev(),
        Command::First => viewer.first(),
        Command::Last => viewer.last(),
        // ヘルプの開閉はイベントループが持つ。Viewer は棋譜の位置だけを持つ
        Command::ToggleHelp | Command::CloseHelp => {}
    }
    Action::Continue
}

/// 端末を初期化してイベントループを回す。
///
/// 端末の後始末は ratatui の `restore` が行う。
/// panic しても端末が壊れたままにならないよう、`init` が hook を差し込む。
pub fn run(mut viewer: Viewer) -> std::io::Result<()> {
    // init は失敗時に panic する。端末が無い経路でも呼び出し側へ返す
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &mut viewer);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, viewer: &mut Viewer) -> std::io::Result<()> {
    use ratatui::crossterm::event::{self, Event, KeyEventKind};

    // ヘルプの開閉はここが持つ。Viewer に持たせると棋譜の位置以外の状態が増える
    let mut help_open = false;
    loop {
        terminal.draw(|frame| {
            viewer.render(frame);
            if help_open {
                help::render(frame);
            }
        })?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        // 押下だけを見る。Windows では離したときにも届く
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match keys::command_for(key) {
            Some(keys::Command::ToggleHelp) => help_open = !help_open,
            Some(keys::Command::CloseHelp) => help_open = false,
            _ => {
                if apply_key(viewer, key) == Action::Quit {
                    return Ok(());
                }
            }
        }
    }
}

/// 一覧から 1 つ選ばせる。取り消されたら None を返す。
pub fn choose(items: Vec<String>, title: &'static str) -> std::io::Result<Option<usize>> {
    if items.is_empty() {
        // 0 件で Picker を作ると、Enter が存在しない添字を返す
        return Ok(None);
    }
    if items.len() == 1 {
        // 1 つしか無い一覧を見せても選ぶ余地が無い
        return Ok(Some(0));
    }
    let mut terminal = ratatui::try_init()?;
    // 早期 return で restore を飛ばすと、raw mode のまま抜ける。
    // init が差し込む hook が拾うのは panic だけになる
    let result = pick(&mut terminal, picker::Picker::new(title, items));
    ratatui::restore();
    result
}

fn pick(
    terminal: &mut ratatui::DefaultTerminal,
    mut picker: picker::Picker,
) -> std::io::Result<Option<usize>> {
    use picker::PickerAction;
    use ratatui::crossterm::event::{self, Event, KeyEventKind};

    loop {
        terminal.draw(|frame| picker.render(frame))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match picker.apply(key) {
            PickerAction::Choose(i) => return Ok(Some(i)),
            PickerAction::Cancel => return Ok(None),
            PickerAction::Continue => {}
        }
    }
}
