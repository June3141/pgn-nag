//! 注釈付き PGN の読み書き。
//!
//! 読みと書きを同じモジュールに置く。
//! 離すと、書式を変えるときに片方だけ直す事故が起きる。

use std::ops::ControlFlow;

use pgn_reader::{Outcome, RawComment, RawTag, Reader, SanPlus, Visitor};
use shakmaty::{Chess, Position};

use crate::model::{Eval, Game, Ply, Score, move_marker};

/// 読み込みに失敗した理由。
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// 指せない手が現れた。
    IllegalMove { ply: usize, san: String },
    /// 注釈の書式が読めない。
    BadEvalTag(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalMove { ply, san } => write!(f, "{ply} 手目 {san} が指せない"),
            Self::BadEvalTag(tag) => write!(f, "注釈が読めない: {tag}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// 注釈付き PGN を読む。
///
/// 変化手順は読み飛ばす (ADR-0005)。
pub fn parse(pgn: &str) -> Result<Vec<Game>, ParseError> {
    let mut visitor = GameVisitor;
    let mut reader = Reader::new(pgn.as_bytes());
    let mut games = Vec::new();
    // 入力は文字列なので io::Error は起きえない
    while let Some(game) = reader.read_game(&mut visitor).expect("メモリ上の入力") {
        games.push(game?);
    }
    Ok(games)
}

struct GameVisitor;

/// 1 対局分の途中状態。
struct Building {
    tags: Vec<(String, String)>,
    plies: Vec<Ply>,
    position: Chess,
    outcome: String,
    error: Option<ParseError>,
}

impl Visitor for GameVisitor {
    type Tags = Vec<(String, String)>;
    type Movetext = Building;
    type Output = Result<Game, ParseError>;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(Vec::new())
    }

    fn tag(
        &mut self,
        tags: &mut Self::Tags,
        name: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        tags.push((
            String::from_utf8_lossy(name).into_owned(),
            String::from_utf8_lossy(&value.decode()).into_owned(),
        ));
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(Building {
            tags,
            plies: Vec::new(),
            position: Chess::default(),
            outcome: String::from("*"),
            error: None,
        })
    }

    fn san(&mut self, b: &mut Self::Movetext, san_plus: SanPlus) -> ControlFlow<Self::Output> {
        if b.error.is_some() {
            return ControlFlow::Continue(());
        }
        let san = san_plus.to_string();
        match san_plus.san.to_move(&b.position) {
            Ok(mv) => {
                b.position.play_unchecked(mv);
                b.plies.push(Ply {
                    san,
                    position: b.position.clone(),
                    eval: None,
                    pv: Vec::new(),
                    comment: None,
                });
            }
            Err(_) => {
                b.error = Some(ParseError::IllegalMove {
                    ply: b.plies.len() + 1,
                    san,
                });
            }
        }
        ControlFlow::Continue(())
    }

    fn comment(
        &mut self,
        b: &mut Self::Movetext,
        comment: RawComment<'_>,
    ) -> ControlFlow<Self::Output> {
        if b.error.is_some() {
            return ControlFlow::Continue(());
        }
        let text = String::from_utf8_lossy(comment.as_bytes()).into_owned();
        // 注釈は直前の手に属する。手より先に現れる注釈は開始局面のものなので捨てる
        let Some(ply) = b.plies.last_mut() else {
            return ControlFlow::Continue(());
        };
        ply.comment = Some(text.clone());
        match apply_comment(ply, &text) {
            Ok(()) => ControlFlow::Continue(()),
            Err(e) => {
                b.error = Some(e);
                ControlFlow::Continue(())
            }
        }
    }

    fn outcome(&mut self, b: &mut Self::Movetext, outcome: Outcome) -> ControlFlow<Self::Output> {
        b.outcome = outcome.to_string();
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, b: Self::Movetext) -> Self::Output {
        match b.error {
            Some(e) => Err(e),
            None => Ok(Game {
                tags: b.tags,
                plies: b.plies,
                outcome: b.outcome,
            }),
        }
    }
}

/// コメントから `[%eval]` と `[%pv]` を取り出して手に載せる。
fn apply_comment(ply: &mut Ply, text: &str) -> Result<(), ParseError> {
    for (name, body) in percent_tags(text) {
        match name {
            "eval" => ply.eval = Some(parse_eval(body)?),
            "pv" => ply.pv = body.split_whitespace().map(str::to_owned).collect(),
            // 未知の %tag は無視する。他ツールが付けた [%clk] 等が該当する
            _ => {}
        }
    }
    Ok(())
}

/// コメント内の `[%name body]` を順に返す。
fn percent_tags(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.split("[%").skip(1).filter_map(|chunk| {
        let body = chunk.strip_suffix(']').or_else(|| {
            // 末尾以外の tag は `] ` までが本体になる
            chunk.find(']').map(|i| &chunk[..i])
        })?;
        let (name, rest) = body.split_once(' ')?;
        Some((name, rest.trim()))
    })
}

/// `0.32,18` や `#-1,18` の形を読む。深さは省略されることがある。
fn parse_eval(body: &str) -> Result<Eval, ParseError> {
    let bad = || ParseError::BadEvalTag(body.to_owned());
    let (value, depth) = match body.split_once(',') {
        Some((v, d)) => (v, Some(d.trim().parse::<u32>().map_err(|_| bad())?)),
        None => (body, None),
    };
    let value = value.trim();
    let score = match value.strip_prefix('#') {
        Some(mate) => Score::Mate(mate.parse::<i32>().map_err(|_| bad())?),
        // centipawn は小数 2 桁のポーン単位で書かれる
        None => {
            let pawns = value.parse::<f64>().map_err(|_| bad())?;
            Score::Cp((pawns * 100.0).round() as i32)
        }
    };
    Ok(Eval { score, depth })
}

/// 注釈付き PGN として書き出す。
///
/// movetext を折り返さない。参照実装が 6000 文字を超える 1 行を出力し、
/// それを主要なツールが問題なく読めている。折り返しを入れると、その位置が
/// 往復一致の対象に加わり、実装の自由度を削るだけになる。
pub fn write(games: &[Game]) -> String {
    let mut out = String::new();
    for game in games {
        for (name, value) in &game.tags {
            out.push_str(&format!("[{name} \"{}\"]\n", escape_tag(value)));
        }
        out.push('\n');
        out.push_str(&movetext(game));
        // 対局の区切りに空行を置く。連結したファイルが 1 局に見えなくなる
        out.push_str("\n\n");
    }
    out
}

/// タグ値の `"` と `\` を PGN の規則に従って退避する。
fn escape_tag(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn movetext(game: &Game) -> String {
    let mut out = String::new();
    for (i, ply) in game.plies.iter().enumerate() {
        if i.is_multiple_of(2) {
            out.push_str(&format!("{} ", move_marker(i)));
        } else if i == 0 || game.plies[i - 1].comment.is_some() {
            // 黒の手は、直前に注釈が入って手番が離れたときだけ番号を繰り返す
            out.push_str(&format!("{} ", move_marker(i)));
        }
        out.push_str(&ply.san);
        out.push(' ');
        if let Some(comment) = &ply.comment {
            out.push('{');
            out.push_str(comment);
            out.push_str("} ");
        }
    }
    out.push_str(&game.outcome);
    out
}
