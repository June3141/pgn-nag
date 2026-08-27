//! 注釈付き PGN を読み書きするための公開 API。

pub mod cli;
pub mod config;
pub mod library;
pub mod model;
pub mod pgn;
pub mod view;

pub use config::Config;
pub use model::{Eval, Game, Ply, Score};
pub use pgn::{ParseError, parse, write};
pub use view::Viewer;
