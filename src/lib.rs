//! 注釈付き PGN を読み書きするための公開 API。

pub mod model;
pub mod pgn;

pub use model::{Eval, Game, Ply, Score};
pub use pgn::{ParseError, parse, write};
