use std::path::{Path, PathBuf};
use std::process::ExitCode;

use pgn_nag::{Viewer, parse};

const USAGE: &str = "usage: nag view <annotated.pgn>";

fn main() -> ExitCode {
    // args() は不正 UTF-8 を含む引数で panic する。
    // Linux では合法なファイル名なので、入力の境界で握らない
    let mut args = std::env::args_os().skip(1);
    let first = args.next();
    match first.as_deref().and_then(std::ffi::OsStr::to_str) {
        Some("view") => match (args.next(), args.next()) {
            (Some(path), None) => run_view(&PathBuf::from(path)),
            _ => usage(ExitCode::FAILURE),
        },
        // 明示的なヘルプ要求は stdout に出して成功で終える
        Some("-h" | "--help" | "help") if args.next().is_none() => usage(ExitCode::SUCCESS),
        _ => usage(ExitCode::FAILURE),
    }
}

fn usage(code: ExitCode) -> ExitCode {
    if code == ExitCode::SUCCESS {
        println!("{USAGE}");
    } else {
        eprintln!("{USAGE}");
    }
    code
}

fn run_view(path: &Path) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{} を読めない: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let games = match parse(&text) {
        Ok(games) => games,
        Err(e) => {
            eprintln!("{} を解析できない: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let total = games.len();
    let Some(game) = games.into_iter().next() else {
        eprintln!("{} に対局が無い", path.display());
        return ExitCode::FAILURE;
    };
    if total > 1 {
        // 無言で捨てると、開いている棋譜が違うことに気付けない
        eprintln!("{total} 局のうち 1 局目を開く");
    }

    match pgn_nag::view::run(Viewer::new(game)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("端末の操作に失敗した: {e}");
            ExitCode::FAILURE
        }
    }
}
