//! 実ファイルに現れるが、engine が生成したサンプルには含まれない書式の検証。
//!
//! 変化手順と、`[%clk]` のように本ツールが解釈しない注釈を対象にする。

use pgn_nag::{Score, parse, write};

const EDGE: &str = include_str!("data/edge-cases.annotated.pgn");

#[test]
fn reads_multiple_games() {
    let games = parse(EDGE).unwrap();
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].outcome, "1-0");
    assert_eq!(games[1].outcome, "1/2-1/2");
}

#[test]
fn skips_variations() {
    let games = parse(EDGE).unwrap();
    // 変化手順の 2 手を数えると 9 手になる
    assert_eq!(games[0].plies.len(), 7);
    assert_eq!(games[0].plies.last().unwrap().san, "Qxf7#");
}

#[test]
fn reads_mate_for_white() {
    let games = parse(EDGE).unwrap();
    let nf6 = &games[0].plies[5];
    assert_eq!(nf6.san, "Nf6");
    assert_eq!(nf6.eval.unwrap().score, Score::Mate(3));
}

#[test]
fn reads_eval_without_depth() {
    let games = parse(EDGE).unwrap();
    let first = games[0].plies[0].eval.unwrap();
    assert_eq!(first.score, Score::Cp(34));
    assert_eq!(first.depth, None, "深さを持たない注釈が実在する");

    let second = games[0].plies[1].eval.unwrap();
    assert_eq!(second.depth, Some(20));
}

#[test]
fn reads_zero_eval() {
    let games = parse(EDGE).unwrap();
    assert_eq!(games[0].plies[4].eval.unwrap().score, Score::Cp(0));
}

#[test]
fn preserves_unknown_percent_tags() {
    // 実ファイルには [%clk] が必ず入る。解釈しない注釈を落とすと、
    // 解析を通しただけで元の情報が消える
    let games = parse(EDGE).unwrap();
    let out = write(&games);
    assert!(out.contains("[%clk 0:03:00]"), "書き出しで消えないこと");
    assert_eq!(out.matches("[%clk").count(), 3);
}

#[test]
fn keeps_evals_alongside_unknown_tags() {
    let games = parse(EDGE).unwrap();
    let out = write(&games);
    assert!(out.contains("[%eval 0.34] [%clk 0:03:00]"));
}
