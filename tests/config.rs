//! 設定の読み込み。
//!
//! 設定は利用者が書き、`nag` は読むだけになる (ADR-0012)。

use std::path::Path;

use pgn_nag::config::{self, Config};

fn write(dir: &Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("config.toml");
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn missing_file_is_not_an_error() {
    let dir = tempdir();
    let found = config::load_from(&dir.join("config.toml")).unwrap();
    assert!(
        found.is_none(),
        "設定が無い状態と壊れている状態を区別すること"
    );
}

#[test]
fn reading_does_not_create_anything() {
    // ADR-0012 の Confirmation。nag は設定を読むだけで書かない
    let dir = tempdir();
    let path = dir.join("absent").join("config.toml");
    assert!(config::load_from(&path).unwrap().is_none());
    assert!(!path.exists(), "読み取りでファイルを作らないこと");
    assert!(
        !path.parent().unwrap().exists(),
        "ディレクトリも作らないこと"
    );
}

#[test]
fn reads_all_fields() {
    let dir = tempdir();
    let path = write(
        &dir,
        r#"
engine = "/usr/games/stockfish"
games_dir = "/home/me/games"

[thresholds]
inaccuracy = 40
mistake = 90
blunder = 180
"#,
    );
    let c = config::load_from(&path).unwrap().unwrap();
    assert_eq!(c.engine.unwrap().to_str().unwrap(), "/usr/games/stockfish");
    assert_eq!(c.games_dir.unwrap().to_str().unwrap(), "/home/me/games");
    assert_eq!(c.thresholds.inaccuracy, 40);
    assert_eq!(c.thresholds.mistake, 90);
    assert_eq!(c.thresholds.blunder, 180);
}

#[test]
fn omitted_fields_fall_back_to_defaults() {
    let dir = tempdir();
    let path = write(&dir, "games_dir = \"/tmp/games\"\n");
    let c = config::load_from(&path).unwrap().unwrap();
    assert!(c.engine.is_none(), "engine は PATH から解決する余地を残す");
    assert_eq!(c.thresholds, Config::default().thresholds);
}

#[test]
fn default_thresholds_match_the_documented_values() {
    let t = Config::default().thresholds;
    assert_eq!((t.inaccuracy, t.mistake, t.blunder), (50, 100, 200));
}

#[test]
fn broken_file_is_an_error() {
    let dir = tempdir();
    let path = write(&dir, "engine = \n");
    assert!(
        config::load_from(&path).is_err(),
        "壊れた設定を既定値で黙って上書きしないこと"
    );
}

#[test]
fn unknown_keys_are_rejected() {
    // 綴りを間違えた設定が黙って無視されると、効いていないことに気付けない
    let dir = tempdir();
    let path = write(&dir, "engien = \"/usr/games/stockfish\"\n");
    assert!(config::load_from(&path).is_err());
}

/// テスト用の一時ディレクトリ。`HOME` には触れない。
fn tempdir() -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!(
        "pgn-nag-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&base).unwrap();
    base
}
