//! 設定の読み込み。
//!
//! 設定は利用者が書き、`nag` は読むだけにする (ADR-0012)。
//! 書き込む先を持たないため、保持した状態が古くなる経路が存在しない。

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// 設定ファイルの中身。
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// engine の実行パス。省略すると `PATH` から解決する。
    pub engine: Option<PathBuf>,
    /// 引数なしで起動したときに一覧にするディレクトリ。
    pub games_dir: Option<PathBuf>,
    #[serde(default)]
    pub thresholds: Thresholds,
}

/// 悪手の閾値。centipawn の損失で表す。
///
/// 保存せずに読み込み時へ算出すると決めているため、ここを変えるだけで
/// 表示が変わる (ADR-0006)。
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub inaccuracy: i32,
    pub mistake: i32,
    pub blunder: i32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            inaccuracy: 50,
            mistake: 100,
            blunder: 200,
        }
    }
}

/// 設定を読めなかった理由。
#[derive(Debug)]
pub enum ConfigError {
    Read(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "読めない: {e}"),
            Self::Parse(e) => write!(f, "書式が正しくない: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 指定した場所から設定を読む。
///
/// ファイルが無い場合は `None` を返す。
/// 既定値で埋めて返すと、設定が無い状態と壊れている状態を区別できない。
pub fn load_from(path: &Path) -> Result<Option<Config>, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ConfigError::Read(e)),
    };
    toml::from_str(&text).map(Some).map_err(ConfigError::Parse)
}

/// 既定の設定ファイルの場所。
///
/// 各 OS の慣行の解釈は crate ごとに割れるため、`etcetera` の結果に従う。
pub fn default_path() -> Option<PathBuf> {
    use etcetera::BaseStrategy;
    etcetera::choose_base_strategy()
        .ok()
        .map(|base| base.config_dir().join("nag").join("config.toml"))
}

/// 既定の場所から設定を読む。
pub fn load() -> Result<Option<Config>, ConfigError> {
    match default_path() {
        Some(path) => load_from(&path),
        None => Ok(None),
    }
}
