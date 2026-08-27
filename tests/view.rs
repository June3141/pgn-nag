//! 盤面の描画とキー操作の検証。
//!
//! 描画が現在位置からの純粋な関数であること (ADR-0011) と、
//! キーの割り当てが慣行どおりであること (ADR-0009) を見る。

use pgn_nag::view::{Action, apply_key};
use pgn_nag::{Viewer, parse};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Modifier;

const SAMPLE: &str = include_str!("data/sample.annotated.pgn");

fn viewer() -> Viewer {
    let game = parse(SAMPLE).unwrap().remove(0);
    Viewer::new(game)
}

/// 画面を行ごとに返す。行末の余白は落とす。
fn draw(viewer: &Viewer) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH as u16, 14)).unwrap();
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

/// 指定した列の範囲を行ごとに取り出す。枠と余白は落とす。
///
/// 隣り合う枠線で空の区切りができるため、`│` での分割では領域を取り出せない。
fn pane(screen: &[String], cols: std::ops::Range<usize>) -> Vec<String> {
    let width = cols.len();
    screen
        .iter()
        .map(|line| {
            line.chars()
                .skip(cols.start)
                .take(width)
                .collect::<String>()
                .trim_matches('│')
                .trim_end()
                .to_owned()
        })
        .collect()
}

/// 画面の幅。
const WIDTH: usize = 64;
/// 盤面の領域。src/view/mod.rs の Layout と揃える
const BOARD: std::ops::Range<usize> = 0..24;
/// 手順リストの領域。盤の右端から画面の右端まで
const MOVES: std::ops::Range<usize> = BOARD.end..WIDTH;

fn press(viewer: &mut Viewer, code: KeyCode) -> Action {
    apply_key(viewer, KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn draws_the_board_from_whites_side() {
    // 段の番号と駒の並びを行ごと突き合わせる。
    // 部分一致で見ると盤を上下逆にしても通ってしまう
    let screen = pane(&draw(&viewer()), BOARD);
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
    let screen = pane(&draw(&v), BOARD);
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
    let screen = pane(&draw(&viewer()), MOVES);
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
    let screen = pane(&draw(&v), MOVES);
    assert!(
        screen.iter().any(|l| l.trim_end().ends_with("#-1")),
        "詰みを詰みとして出すこと: {screen:?}"
    );
}

/// 手順リストで反転している行を返す。
///
/// 強調は文字ではなく属性なので、テキストの比較では検出できない。
fn highlighted(viewer: &Viewer) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH as u16, 14)).unwrap();
    terminal.draw(|frame| viewer.render(frame)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area.width as usize;
    buffer
        .content()
        .chunks(width)
        .filter_map(|row| {
            let marked: String = row[MOVES.start..MOVES.end]
                .iter()
                .filter(|cell| cell.modifier.contains(Modifier::REVERSED))
                .map(|cell| cell.symbol())
                .collect();
            (!marked.trim().is_empty()).then(|| marked.trim().to_owned())
        })
        .collect()
}

#[test]
fn marks_the_current_ply() {
    let mut v = viewer();
    v.next();
    assert_eq!(highlighted(&v).len(), 1, "反転する行は 1 つだけ");
    assert!(highlighted(&v)[0].contains("d4"), "{:?}", highlighted(&v));

    v.next();
    assert!(highlighted(&v)[0].contains("c6"), "{:?}", highlighted(&v));
}

#[test]
fn marks_nothing_at_the_starting_position() {
    // 開始局面ではまだ指した手が無い
    assert!(highlighted(&viewer()).is_empty());
}

#[test]
fn scrolls_to_keep_the_current_ply_visible() {
    let mut v = viewer();
    v.last();
    let screen = pane(&draw(&v), MOVES);
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
    let screen = pane(&draw(&v), MOVES);
    let line = screen.iter().find(|l| l.contains("Qf1#")).unwrap();
    assert!(
        !line.contains("0.00"),
        "注釈が無い手を 0.00 と出さないこと: {line}"
    );
}

#[test]
fn keeps_the_last_ply_at_the_bottom() {
    // 末尾で詰めないと、下半分が空欄のまま送られてしまう
    let mut v = viewer();
    v.last();
    let screen = pane(&draw(&v), MOVES);
    let rows = &screen[1..13];
    assert!(
        rows.iter().all(|l| !l.is_empty()),
        "末尾で詰めないと下半分が空欄になる: {screen:?}"
    );
    assert!(rows.last().unwrap().contains("Qf1#"), "{screen:?}");
}

#[test]
fn shows_the_ply_counter() {
    // 何手目を見ているかが画面から分かること
    let mut v = viewer();
    v.next();
    let screen = draw(&v);
    assert!(
        screen.iter().any(|l| l.contains("1/98")),
        "現在位置と総手数が出ること: {:?}",
        &screen[0..2]
    );
}

#[test]
fn does_not_truncate_the_eval() {
    // 詰み手数が切れると #-10 が #-1 に読める
    let mut v = viewer();
    for _ in 0..90 {
        v.next();
    }
    let mut terminal = Terminal::new(TestBackend::new(46, 14)).unwrap();
    terminal.draw(|frame| v.render(frame)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area.width as usize;
    let rows: Vec<String> = buffer
        .content()
        .chunks(width)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect();
    for row in &rows {
        assert!(
            !row.contains("#-1 ") || row.contains("#-1 ") && !row.contains("#-10"),
            "詰み手数が桁の途中で切れないこと: {row}"
        );
        // 評価値の途中で切れた形が出ていないこと
        assert!(
            !row.trim_end().ends_with("-7."),
            "評価値が切れている: {row}"
        );
    }
}
