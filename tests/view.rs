//! 盤面の描画とキー操作の検証。
//!
//! 描画が現在位置からの純粋な関数であること (ADR-0011) と、
//! キーの割り当てが慣行どおりであること (ADR-0009) を見る。

use pgn_nag::view::{Action, apply_key};
use pgn_nag::{Viewer, parse};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const SAMPLE: &str = include_str!("data/sample.annotated.pgn");

fn viewer() -> Viewer {
    let game = parse(SAMPLE).unwrap().remove(0);
    Viewer::new(game)
}

/// 画面を行ごとに返す。行末の余白は落とす。
fn draw(viewer: &Viewer) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(64, 14)).unwrap();
    terminal.draw(|frame| viewer.render(frame)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area.width as usize;
    buffer
        .content()
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

/// 枠で区切られた n 番目の領域を行ごとに取り出す。
fn pane(screen: &[String], index: usize) -> Vec<String> {
    screen
        .iter()
        .map(|line| {
            line.split('│')
                .nth(index + 1)
                .unwrap_or("")
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn press(viewer: &mut Viewer, code: KeyCode) -> Action {
    apply_key(viewer, KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn draws_the_board_from_whites_side() {
    // 段の番号と駒の並びを行ごと突き合わせる。
    // 部分一致で見ると盤を上下逆にしても通ってしまう
    let screen = pane(&draw(&viewer()), 0);
    let board: Vec<&str> = screen[1..10].iter().map(String::as_str).collect();
    assert_eq!(
        board,
        [
            "8  r n b q k b n r",
            "7  p p p p p p p p",
            "6  . . . . . . . .",
            "5  . . . . . . . .",
            "4  . . . . . . . .",
            "3  . . . . . . . .",
            "2  P P P P P P P P",
            "1  R N B Q K B N R",
            "   a b c d e f g h",
        ]
    );
}

#[test]
fn advancing_moves_a_single_pawn() {
    let mut v = viewer();
    v.next(); // 1. d4
    let screen = pane(&draw(&v), 0);
    assert_eq!(screen[5], "4  . . . P . . . .");
    assert_eq!(screen[7], "2  P P P . P P P P");
}

#[test]
fn same_cursor_gives_same_screen() {
    // 経路に依らず同じ画面になること (ADR-0011 の Confirmation)
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
fn stepping_back_returns_to_the_previous_ply() {
    let mut one = viewer();
    one.next();

    let mut back = viewer();
    back.next();
    back.next();
    back.prev();

    assert_eq!(draw(&back), draw(&one), "1 手だけ戻ること");
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

#[test]
fn arrows_and_hl_agree() {
    // ADR-0009: 矢印キーと h / l の双方で同じ結果になること
    let mut arrows = viewer();
    let mut letters = viewer();
    for _ in 0..3 {
        press(&mut arrows, KeyCode::Right);
        press(&mut letters, KeyCode::Char('l'));
    }
    assert_eq!(draw(&arrows), draw(&letters));

    press(&mut arrows, KeyCode::Left);
    press(&mut letters, KeyCode::Char('h'));
    assert_eq!(draw(&arrows), draw(&letters));
}

#[test]
fn vertical_keys_do_not_move_between_plies() {
    // ADR-0009: 上下は 1 手単位の移動に割り当てない
    let mut v = viewer();
    v.next();
    let before = draw(&v);
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Char('j'),
        KeyCode::Char('k'),
    ] {
        press(&mut v, code);
        assert_eq!(draw(&v), before, "{code:?} で手が動かないこと");
    }
}

#[test]
fn jumps_to_both_ends() {
    let mut v = viewer();
    press(&mut v, KeyCode::Char('G'));
    let mut end = viewer();
    end.last();
    assert_eq!(draw(&v), draw(&end));

    press(&mut v, KeyCode::Char('g'));
    assert_eq!(draw(&v), draw(&viewer()));
}

#[test]
fn quits_on_q_and_ctrl_c() {
    // raw mode では ISIG が落ちるため Ctrl+C は SIGINT にならない。
    // 明示的に受けないと q 以外の脱出手段が無くなる
    let mut v = viewer();
    assert_eq!(press(&mut v, KeyCode::Char('q')), Action::Quit);
    assert_eq!(
        apply_key(
            &mut v,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
        ),
        Action::Quit
    );
}

#[test]
fn modifier_keys_do_not_move() {
    // Ctrl+L で手が進んでしまわないこと
    let mut v = viewer();
    let before = draw(&v);
    apply_key(
        &mut v,
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
    );
    assert_eq!(draw(&v), before);
}

#[test]
fn lists_moves_with_evals() {
    let screen = pane(&draw(&viewer()), 1);
    // 1. d4 { [%eval 0.32,18] }  1... c6 { [%eval 0.32,18] }
    assert!(
        screen
            .iter()
            .any(|l| l.contains("1.") && l.contains("d4") && l.contains("+0.32")),
        "手番号と SAN と評価値が並ぶこと: {screen:?}"
    );
    assert!(
        screen
            .iter()
            .any(|l| l.contains("1...") && l.contains("c6"))
    );
}

#[test]
fn shows_mate_as_mate() {
    // 終盤は [%eval #-1,18]。centipawn に潰して表示しない
    let mut v = viewer();
    v.last();
    let screen = pane(&draw(&v), 1);
    assert!(
        screen.iter().any(|l| l.contains("#-1")),
        "詰みを詰みとして出すこと: {screen:?}"
    );
}

#[test]
fn marks_the_current_ply() {
    let mut v = viewer();
    v.next();
    let one = pane(&draw(&v), 1);
    v.next();
    let two = pane(&draw(&v), 1);
    assert_ne!(one, two, "現在の手が変われば手順リストの見た目も変わること");
}

#[test]
fn scrolls_to_keep_the_current_ply_visible() {
    let mut v = viewer();
    v.last();
    let screen = pane(&draw(&v), 1);
    assert!(
        screen.iter().any(|l| l.contains("Qf1#")),
        "終局手が見えること: {screen:?}"
    );
}

#[test]
fn leaves_the_eval_blank_when_absent() {
    // 終局手には注釈が無い。0.00 と紛れる表示にしない
    let mut v = viewer();
    v.last();
    let screen = pane(&draw(&v), 1);
    let line = screen.iter().find(|l| l.contains("Qf1#")).unwrap();
    assert!(
        !line.contains("0.00"),
        "注釈が無い手を 0.00 と出さないこと: {line}"
    );
}
