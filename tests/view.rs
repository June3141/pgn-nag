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

/// 盤の行から評価バーを落とす。駒の並びだけを見たいとき使う。
fn squares(rows: &[String]) -> Vec<String> {
    rows.iter()
        .map(|l| l.trim_end_matches(['█', '░', ' ']).to_owned())
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
/// 状態行が占める行数。枠を含む
const STATUS_ROWS: usize = 3;
/// 手順リストの領域。盤の右端から画面の右端まで
const MOVES: std::ops::Range<usize> = BOARD.end..WIDTH;

fn press(viewer: &mut Viewer, code: KeyCode) -> Action {
    apply_key(viewer, KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn draws_the_board_from_whites_side() {
    // 段の番号と駒の並びを行ごと突き合わせる。
    // 部分一致で見ると盤を上下逆にしても通ってしまう
    let screen = squares(&pane(&draw(&viewer()), BOARD));
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
    let screen = squares(&pane(&draw(&v), BOARD));
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
    // 手順リストは状態行のぶんだけ縮む
    let rows = &screen[1..screen.len() - STATUS_ROWS - 1];
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

/// 最下段の状態行。
fn status(screen: &[String]) -> String {
    screen[screen.len() - 2].trim_matches('│').trim().to_owned()
}

#[test]
fn shows_the_current_eval_and_depth() {
    let mut v = viewer();
    v.next(); // 1. d4 { [%eval 0.32,18] }
    let line = status(&draw(&v));
    assert!(line.contains("+0.32"), "評価値が出ること: {line}");
    assert!(line.contains("18"), "深さが出ること: {line}");
}

#[test]
fn shows_the_principal_variation_in_san() {
    // 保持しているのは UCI 表記だが、読むのは SAN
    let mut v = viewer();
    v.next(); // [%pv g8f6 c2c4 e7e6 g1f3 d7d5 c1g5]
    let line = status(&draw(&v));
    assert!(line.contains("Nf6"), "UCI ではなく SAN で出ること: {line}");
    assert!(
        !line.contains("g8f6"),
        "UCI がそのまま出ていないこと: {line}"
    );
}

#[test]
fn leaves_the_status_quiet_without_an_eval() {
    // 開始局面には注釈が無い
    let line = status(&draw(&viewer()));
    assert!(
        !line.contains("+0.00"),
        "注釈が無いのに評価値を出さないこと: {line}"
    );
}

/// 評価バーが立つ列。盤の右端と枠の内側にあたる。
const BAR_COLUMN: usize = 21;

/// 盤の右端に立つ評価バーを 1 本の文字列として返す。
///
/// 個数だけを数えると、上下を反転しても気付けない。
/// 行末を詰めると、バーが空のときに駒を拾ってしまう。
fn eval_bar(viewer: &Viewer) -> String {
    draw(viewer)[1..9]
        .iter()
        .map(|l| l.chars().nth(BAR_COLUMN).unwrap_or(' '))
        .collect()
}

#[test]
fn eval_bar_fills_from_the_bottom() {
    let mut v = viewer();
    v.next(); // +0.32
    assert_eq!(eval_bar(&v), "░░░░████", "白優勢は下から埋まること");
}

#[test]
fn eval_bar_is_empty_without_an_eval() {
    // 中立を描くと、互角と注釈なしが見分けられない
    assert_eq!(eval_bar(&viewer()), "        ");
}

#[test]
fn shows_check_and_mate_in_the_principal_variation() {
    // 詰みの評価値の隣に、詰まない手順を並べない
    let mut v = viewer();
    for _ in 0..91 {
        v.next();
    }
    let line = status(&draw(&v));
    assert!(line.contains("Nf3#"), "詰みの記号が落ちないこと: {line}");

    let mut checked = viewer();
    for _ in 0..40 {
        checked.next();
    }
    assert!(
        status(&draw(&checked)).contains("Rxc8+"),
        "王手の記号が落ちないこと"
    );
}

#[test]
fn numbers_the_principal_variation() {
    // 番号が無いと、最善手順の先頭がどちらの手番か分からない
    let mut v = viewer();
    v.next();
    let line = status(&draw(&v));
    assert!(line.contains("1... Nf6"), "{line}");
}

#[test]
fn puts_depth_before_the_variation() {
    // 深さを可変長の手順の後ろに置くと、狭い端末で桁の途中から切れる
    let mut v = viewer();
    v.next();
    let line = status(&draw(&v));
    assert!(line.find("depth") < line.find("PV"), "{line}");
}

#[test]
fn handles_evals_without_depth() {
    // edge-cases には深さを持たない注釈と最善手順の無い手がある
    let game = parse(include_str!("data/edge-cases.annotated.pgn"))
        .unwrap()
        .remove(0);
    let mut v = Viewer::new(game);
    v.next();
    let line = status(&draw(&v));
    assert!(line.contains("+0.34"), "{line}");
    assert!(
        !line.contains("depth"),
        "深さが無いのに出さないこと: {line}"
    );
    assert!(
        !line.contains("PV"),
        "最善手順が無いのに出さないこと: {line}"
    );
}

#[test]
fn eval_bar_is_empty_when_black_mates() {
    // 詰みの向きを反転しても気付けるようにする
    let mut v = viewer();
    for _ in 0..94 {
        v.next();
    }
    assert_eq!(eval_bar(&v), "░░░░░░░░", "黒が詰ませる局面");
}

#[test]
fn keeps_the_board_on_a_short_terminal() {
    // 状態行を優先すると盤の段が黙って消える。
    // 逆に高さが足りないまま状態行を描くと、底辺の無い枠が残る
    for height in [11u16, 12, 13, 14] {
        let mut terminal = Terminal::new(TestBackend::new(WIDTH as u16, height)).unwrap();
        terminal.draw(|frame| viewer().render(frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let width = buffer.area.width as usize;
        let rows: Vec<String> = buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|c| c.symbol()).collect())
            .collect();
        let screen = rows.join("\n");
        assert!(
            screen.contains("a b c d e f g h"),
            "高さ {height} で盤が削られている"
        );
        let opens = screen.matches('┌').count();
        let closes = screen.matches('└').count();
        assert_eq!(opens, closes, "高さ {height} で閉じない枠が残る:\n{screen}");
    }
}

#[test]
fn eval_bar_follows_the_advantage() {
    use pgn_nag::Score;

    let game = parse(SAMPLE).unwrap().remove(0);
    let cp = |i: usize| match game.plies[i].eval.map(|e| e.score) {
        Some(Score::Cp(cp)) => Some(cp),
        _ => None,
    };
    let best = (0..game.plies.len())
        .filter(|&i| cp(i).is_some())
        .max_by_key(|&i| cp(i).unwrap())
        .unwrap();
    let worst = (0..game.plies.len())
        .filter(|&i| cp(i).is_some())
        .min_by_key(|&i| cp(i).unwrap())
        .unwrap();

    let filled = |cursor: usize| {
        let mut v = viewer();
        for _ in 0..cursor {
            v.next();
        }
        pane(&draw(&v), BOARD)
            .iter()
            .map(|l| l.matches('█').count())
            .sum::<usize>()
    };
    let white = filled(best + 1);
    let black = filled(worst + 1);
    assert!(
        white > black,
        "白優勢のほうが多く埋まること: {white} vs {black}"
    );
    assert_eq!(filled(0), 0, "注釈が無い開始局面では出さないこと");
}

#[test]
fn help_lists_every_binding() {
    // ADR-0009: ヘルプに並ぶキーと実装のキーテーブルが過不足なく一致すること
    use pgn_nag::view::keys;
    let mut terminal = Terminal::new(TestBackend::new(WIDTH as u16, 20)).unwrap();
    terminal.draw(pgn_nag::view::render_help).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area.width as usize;
    let screen: String = buffer
        .content()
        .chunks(width)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    // TestBackend は全角文字の後ろに詰め物のセルを置く。
    // 実際の端末では 2 桁で描かれるため、空白を除いて比べる
    let squeezed = screen.replace(' ', "");
    for binding in keys::BINDINGS {
        assert!(
            squeezed.contains(&binding.label.replace(' ', "")),
            "{} がヘルプに出ていない",
            binding.label
        );
        assert!(
            squeezed.contains(&binding.description.replace(' ', "")),
            "{} の説明がヘルプに出ていない",
            binding.label
        );
    }
}

#[test]
fn every_binding_key_is_handled() {
    // 表に載っているキーが実際に効くこと
    use pgn_nag::view::keys;
    for binding in keys::BINDINGS {
        for code in binding.codes {
            assert!(
                keys::command_for(KeyEvent::new(*code, KeyModifiers::NONE)).is_some(),
                "{:?} が処理されていない",
                code
            );
        }
    }
}

#[test]
fn help_is_reachable_and_dismissable() {
    use pgn_nag::view::keys::{Command, command_for};
    assert_eq!(
        command_for(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
        Some(Command::ToggleHelp)
    );
    assert_eq!(
        command_for(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(Command::CloseHelp)
    );
}
