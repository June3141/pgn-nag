//! 注釈付き PGN の読み込みと再書き出しの検証。
//!
//! 入力は固定のデータなので、engine のバージョンに影響されない。

use pgn_nag::{Score, parse, write};

const SAMPLE: &str = include_str!("data/sample.annotated.pgn");

#[test]
fn reads_single_game() {
    let games = parse(SAMPLE).expect("読めること");
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].outcome, "0-1");
}

#[test]
fn reads_every_ply() {
    let games = parse(SAMPLE).unwrap();
    assert_eq!(games[0].plies.len(), 98);
}

#[test]
fn final_mate_ply_has_no_eval() {
    let games = parse(SAMPLE).unwrap();
    let plies = &games[0].plies;
    let annotated = plies.iter().filter(|p| p.eval.is_some()).count();
    assert_eq!(annotated, 97, "checkmate の局面だけ注釈が落ちる");

    let last = plies.last().unwrap();
    assert_eq!(last.san, "Qf1#");
    assert!(last.eval.is_none());
}

#[test]
fn scores_are_white_pov() {
    let games = parse(SAMPLE).unwrap();
    let plies = &games[0].plies;

    // 1. d4 { [%eval 0.32,18] }
    let first = plies[0].eval.unwrap();
    assert_eq!(first.score, Score::Cp(32));
    assert_eq!(first.depth, Some(18));

    // 黒が優勢な局面は、指した側に依らず負の値になる
    let worst = plies
        .iter()
        .filter_map(|p| match p.eval.map(|e| e.score) {
            Some(Score::Cp(cp)) => Some(cp),
            _ => None,
        })
        .min()
        .unwrap();
    assert!(worst < 0, "黒優勢の局面が負で表現されること");
}

#[test]
fn mate_is_distinct_from_centipawns() {
    let games = parse(SAMPLE).unwrap();
    let mates: Vec<i32> = games[0]
        .plies
        .iter()
        .filter_map(|p| match p.eval.map(|e| e.score) {
            Some(Score::Mate(n)) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(mates.len(), 16);
    assert!(mates.iter().all(|&n| n < 0), "黒が詰ませる側になる");
    assert_eq!(*mates.last().unwrap(), -1);
}

#[test]
fn reads_principal_variation() {
    let games = parse(SAMPLE).unwrap();
    // 1. d4 { [%pv g8f6 c2c4 e7e6 g1f3 d7d5 c1g5] }
    assert_eq!(
        games[0].plies[0].pv,
        ["g8f6", "c2c4", "e7e6", "g1f3", "d7d5", "c1g5"]
    );
}

#[test]
fn write_after_parse_is_byte_identical() {
    let games = parse(SAMPLE).unwrap();
    assert_eq!(write(&games), SAMPLE);
}
