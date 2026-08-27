use std::path::{Path, PathBuf};
use std::process::ExitCode;

use pgn_nag::{Viewer, config, library, parse, view};

const USAGE: &str = "usage: nag [view <annotated.pgn|dir>]";

fn main() -> ExitCode {
    // args() は不正 UTF-8 を含む引数で panic する。
    // Linux では合法なファイル名なので、入力の境界で握らない
    let mut args = std::env::args_os().skip(1);
    let first = args.next();
    // OsStr のまま比べる。to_str を挟むと、UTF-8 でない引数が
    // 引数なしと同じ枝に落ちて設定案内が出る
    match first.as_deref() {
        // 引数なしは、設定した棋譜置き場を一覧にする (ADR-0012)
        None => run_library(),
        Some(command) if command == "view" => match (args.next(), args.next()) {
            (Some(path), None) => open(&PathBuf::from(path)),
            _ => usage(ExitCode::FAILURE),
        },
        Some(command)
            if matches!(command.to_str(), Some("-h" | "--help" | "help"))
                && args.next().is_none() =>
        {
            usage(ExitCode::SUCCESS)
        }
        Some(_) => usage(ExitCode::FAILURE),
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

/// 設定された棋譜置き場を開く。
fn run_library() -> ExitCode {
    let settings = match config::load() {
        Ok(Some(c)) => c,
        Ok(None) => return explain_missing_config(),
        Err(e) => {
            eprintln!("設定を読めない: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(dir) = settings.games_dir else {
        return explain_missing_config();
    };
    if !dir.is_dir() {
        // ADR-0012 はディレクトリが存在しない場合も書き方を示すと決めている。
        // PGN として開こうとすると、設定の誤りが読み取りの失敗として出る
        eprintln!("games_dir がディレクトリでない: {}", dir.display());
        return explain_missing_config();
    }
    open(&dir)
}

/// 設定が無いことを、書き方とともに伝える。
///
/// 空の一覧を出すと、設定の誤りとファイルが 1 つも無い状態が区別できない。
fn explain_missing_config() -> ExitCode {
    eprintln!("棋譜の置き場所が設定されていない。");
    eprintln!();
    print_config_hint();
    ExitCode::FAILURE
}

/// 設定の書き方を示す。
fn print_config_hint() {
    let path = config::default_path().map_or_else(
        || "設定ディレクトリ".to_owned(),
        |p| p.display().to_string(),
    );
    eprintln!("{path} に次を書く:");
    eprintln!();
    eprintln!("    games_dir = \"/path/to/games\"");
    eprintln!();
    eprintln!("パスを直接指定して開くこともできる: nag view <annotated.pgn|dir>");
}

/// ファイルまたはディレクトリを開く。
fn open(path: &Path) -> ExitCode {
    let file = if path.is_dir() {
        match choose_file(path) {
            Ok(Some(f)) => f,
            Ok(None) => return ExitCode::SUCCESS,
            Err(code) => return code,
        }
    } else {
        path.to_path_buf()
    };
    open_file(&file)
}

fn choose_file(dir: &Path) -> Result<Option<PathBuf>, ExitCode> {
    let files = match library::pgn_files(dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{} を読めない: {e}", dir.display());
            return Err(ExitCode::FAILURE);
        }
    };
    if files.is_empty() {
        eprintln!("{} に .pgn が無い", dir.display());
        return Err(ExitCode::FAILURE);
    }
    let labels = files
        .iter()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    match view::choose(labels, " files ") {
        Ok(Some(i)) => Ok(Some(files[i].clone())),
        Ok(None) => Ok(None),
        Err(e) => {
            eprintln!("端末の操作に失敗した: {e}");
            Err(ExitCode::FAILURE)
        }
    }
}

fn open_file(path: &Path) -> ExitCode {
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
    if games.is_empty() {
        eprintln!("{} に対局が無い", path.display());
        return ExitCode::FAILURE;
    }
    let labels = games.iter().map(label).collect();
    let chosen = match view::choose(labels, " games ") {
        Ok(Some(i)) => i,
        Ok(None) => return ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("端末の操作に失敗した: {e}");
            return ExitCode::FAILURE;
        }
    };

    match view::run(Viewer::new(
        games.into_iter().nth(chosen).expect("選んだ対局"),
    )) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("端末の操作に失敗した: {e}");
            ExitCode::FAILURE
        }
    }
}

/// 一覧に出す対局の見出し。
fn label(game: &pgn_nag::Game) -> String {
    format!(
        "{} vs {}  {}  {}",
        game.tag("White"),
        game.tag("Black"),
        game.tag("Date"),
        game.outcome
    )
}
