//! 引数の解釈。
//!
//! 端末を持たない形で全分岐を検証できるよう、入出力から切り離す。

use std::ffi::OsString;
use std::path::PathBuf;

/// 引数から決まる動作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// 設定した棋譜置き場を一覧にする。
    Library,
    /// 指定したファイルまたはディレクトリを開く。
    Open(PathBuf),
    /// 使い方を出して成功で終える。
    Help,
    /// 使い方を出して失敗で終える。
    Usage,
}

/// 引数を解釈する。
///
/// `OsString` のまま扱う。`to_str` を先に挟むと、UTF-8 でない引数が
/// 引数なしと同じ枝に落ちる。Linux では合法なファイル名になる。
pub fn parse<I: IntoIterator<Item = OsString>>(args: I) -> Command {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Command::Library;
    };
    if first == "view" {
        return match (args.next(), args.next()) {
            (Some(path), None) => Command::Open(PathBuf::from(path)),
            _ => Command::Usage,
        };
    }
    if matches!(first.to_str(), Some("-h" | "--help" | "help")) && args.next().is_none() {
        return Command::Help;
    }
    Command::Usage
}
