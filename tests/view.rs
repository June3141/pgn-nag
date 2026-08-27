//! 盤面の描画が現在位置からの純粋な関数であることの検証 (ADR-0011)。

use pgn_nag::{Viewer, parse};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const SAMPLE: &str = include_str!("data/sample.annotated.pgn");

fn viewer() -> Viewer {
    let game = parse(SAMPLE).unwrap().remove(0);
    Viewer::new(game)
}

fn draw(viewer: &Viewer) -> String {
    let mut terminal = Terminal::new(TestBackend::new(40, 14)).unwrap();
    terminal.draw(|frame| viewer.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(40)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn shows_initial_position() {
    let screen = draw(&viewer());
    assert!(screen.contains("r n b q k b n r"), "8 段目に黒の駒が並ぶ");
    assert!(screen.contains("R N B Q K B N R"), "1 段目に白の駒が並ぶ");
    assert!(screen.contains("a b c d e f g h"));
}

#[test]
fn advancing_changes_the_board() {
    let mut v = viewer();
    let before = draw(&v);
    v.next(); // 1. d4
    let after = draw(&v);
    assert_ne!(before, after);
    // d2 のポーンが d4 へ動いた結果、2 段目に空きができる
    assert!(after.contains("P P P . P P P P"));
}

#[test]
fn same_cursor_gives_same_screen() {
    // 経路に依らず同じ画面になること
    let mut forward = viewer();
    forward.next();
    forward.next();

    let mut roundabout = viewer();
    roundabout.last();
    roundabout.first();
    roundabout.next();
    roundabout.next();

    assert_eq!(draw(&forward), draw(&roundabout));
}

#[test]
fn movement_stops_at_both_ends() {
    let mut v = viewer();
    v.prev();
    assert_eq!(draw(&v), draw(&viewer()), "開始局面より前へは戻らない");

    let mut end = viewer();
    end.last();
    let at_end = draw(&end);
    end.next();
    assert_eq!(draw(&end), at_end, "終局より先へは進まない");
}
