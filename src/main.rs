use std::process::ExitCode;

use pgn_nag::{Viewer, parse};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("view"), Some(path)) => run_view(&path),
        _ => {
            eprintln!("usage: nag view <annotated.pgn>");
            ExitCode::FAILURE
        }
    }
}

fn run_view(path: &str) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{path} を読めない: {e}");
            return ExitCode::FAILURE;
        }
    };
    let games = match parse(&text) {
        Ok(games) => games,
        Err(e) => {
            eprintln!("{path} を解析できない: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(game) = games.into_iter().next() else {
        eprintln!("{path} に対局が無い");
        return ExitCode::FAILURE;
    };

    match pgn_nag::view::run(Viewer::new(game)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("端末の操作に失敗した: {e}");
            ExitCode::FAILURE
        }
    }
}
