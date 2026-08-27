//! 棋譜ファイルの一覧。
//!
//! 一覧はディレクトリの内容そのものを使う (ADR-0012)。

use std::path::PathBuf;

use pgn_nag::library;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pgn-nag-lib-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn lists_pgn_files_sorted() {
    let dir = scratch("sorted");
    for name in ["b.pgn", "a.pgn", "c.txt"] {
        std::fs::write(dir.join(name), "").unwrap();
    }
    let found = library::pgn_files(&dir).unwrap();
    let names: Vec<String> = found
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        ["a.pgn", "b.pgn"],
        "拡張子で絞り、名前順に並べること"
    );
}

#[test]
fn does_not_descend_into_subdirectories() {
    // ADR-0012: サブディレクトリは辿らない
    let dir = scratch("flat");
    std::fs::write(dir.join("top.pgn"), "").unwrap();
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("sub").join("deep.pgn"), "").unwrap();
    let found = library::pgn_files(&dir).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("top.pgn"));
}

#[test]
fn keeps_unannotated_files() {
    // ADR-0001 が未注釈の PGN も再生できる経路を残すため、絞り込まない
    let dir = scratch("plain");
    std::fs::write(dir.join("plain.pgn"), "1. e4 e5 *\n").unwrap();
    assert_eq!(library::pgn_files(&dir).unwrap().len(), 1);
}

#[test]
fn missing_directory_is_an_error() {
    let dir = scratch("gone").join("nope");
    assert!(library::pgn_files(&dir).is_err(), "空の一覧と区別すること");
}

#[test]
fn ignores_directories_named_like_pgn() {
    // `x.pgn` という名前のディレクトリを一覧に混ぜない
    let dir = scratch("dirlike");
    std::fs::write(dir.join("real.pgn"), "").unwrap();
    std::fs::create_dir_all(dir.join("fake.pgn")).unwrap();
    let found = library::pgn_files(&dir).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("real.pgn"));
}
