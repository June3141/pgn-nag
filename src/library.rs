//! 棋譜ファイルの一覧。
//!
//! 一覧はディレクトリの内容そのものを使う (ADR-0012)。
//! 別に索引を持つと、ファイルの移動や削除との食い違いが生まれる。

use std::path::{Path, PathBuf};

/// ディレクトリ直下の `.pgn` を名前順に返す。
///
/// サブディレクトリは辿らない。
/// 注釈の有無で絞り込まないのは、ADR-0001 が未注釈の PGN も再生できる
/// 経路を残すと決めているため。拡張子だけで判断するので、一覧を作るために
/// ファイルを開く必要も無い。
pub fn pgn_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_pgn(p))
        .collect();
    found.sort();
    Ok(found)
}

fn is_pgn(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pgn"))
}
