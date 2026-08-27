//! 注釈付き PGN の読み書き。
//!
//! 読みと書きを同じモジュールに置く。
//! 離すと、書式を変えるときに片方だけ直す事故が起きる。

use crate::model::Game;

/// 読み込みに失敗した理由。
#[derive(Debug)]
pub enum ParseError {
    /// 指せない手が現れた。
    IllegalMove { ply: usize, san: String },
    /// 注釈の書式が読めない。
    BadEvalTag(String),
}

/// 注釈付き PGN を読む。
pub fn parse(_pgn: &str) -> Result<Vec<Game>, ParseError> {
    todo!("M1")
}

/// 注釈付き PGN として書き出す。
pub fn write(_games: &[Game]) -> String {
    todo!("M1")
}
