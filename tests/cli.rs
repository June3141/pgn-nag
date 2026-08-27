//! 引数の解釈。
//!
//! 端末を持たない経路なので、全分岐をテストで固定できる。

use std::ffi::OsString;
use std::path::PathBuf;

use pgn_nag::cli::{Command, parse};

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn no_arguments_opens_the_library() {
    assert_eq!(parse(args(&[])), Command::Library);
}

#[test]
fn view_takes_exactly_one_path() {
    assert_eq!(
        parse(args(&["view", "a.pgn"])),
        Command::Open(PathBuf::from("a.pgn"))
    );
    assert_eq!(parse(args(&["view"])), Command::Usage);
    assert_eq!(parse(args(&["view", "a.pgn", "extra"])), Command::Usage);
}

#[test]
fn help_is_accepted_in_three_forms() {
    for form in ["-h", "--help", "help"] {
        assert_eq!(parse(args(&[form])), Command::Help, "{form}");
    }
    assert_eq!(parse(args(&["--help", "extra"])), Command::Usage);
}

#[test]
fn unknown_command_is_usage() {
    assert_eq!(parse(args(&["analyze"])), Command::Usage);
}

#[test]
fn non_utf8_argument_is_not_treated_as_absent() {
    // args() は非 UTF-8 で panic する。args_os に替えた意味を保つ
    use std::os::unix::ffi::OsStringExt;
    let broken = OsString::from_vec(vec![0xff, 0xfe]);
    assert_eq!(parse(vec![broken]), Command::Usage);
}

#[test]
fn non_utf8_path_is_kept() {
    use std::os::unix::ffi::OsStringExt;
    let path = OsString::from_vec(vec![0xff, b'.', b'p', b'g', b'n']);
    assert_eq!(
        parse(vec![OsString::from("view"), path.clone()]),
        Command::Open(PathBuf::from(path))
    );
}
