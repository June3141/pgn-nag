"""PR タイトルが commit-rules の書式に従うかを検査する。

squash merge では PR タイトルがそのまま commit subject になるため、履歴に
直接残る唯一の手入力になる。

usage: pr-title-lint.py "<title>"
       pr-title-lint.py --selftest
"""

import re
import sys

TYPES = ("feat", "fix", "docs", "test", "refactor", "chore", "style", "perf", "sec")
EMOJIS = ("✨", "🐛", "📝", "✅", "♻️", "🔧", "🎨", "⚡️", "🔥", "💥", "🚀", "🚧", "🔒", "⬆️", "🗃️", "🎉")
MAX_LENGTH = 70

_HEAD = re.compile(
    rf"^(?:{'|'.join(TYPES)})!?: (?:{'|'.join(re.escape(e) for e in EMOJIS)}) (?P<subject>.+)$"
)


def validate(title: str) -> str | None:
    """書式違反の説明を返す。問題が無ければ None を返す。"""
    if not title.strip():
        return "タイトルが空"
    if len(title) > MAX_LENGTH:
        return f"{MAX_LENGTH} 文字を超えている ({len(title)} 文字)"
    m = _HEAD.match(title)
    if not m:
        return "書式が <type>: <emoji> <subject> になっていない"
    # 括弧は squash merge が付ける (#N) と重なって二重括弧になる
    if "(" in title or ")" in title:
        return "括弧を使わない。補足は description に書く"
    # subject の #N は auto-link されず plain text の noise になる
    if re.search(r"#\d", title):
        return "issue / PR 番号を書かない。参照は description の Refs / Closes に置く"
    if re.search(r"\[(WIP|DRAFT)\]", title, re.IGNORECASE):
        return "状態は GitHub の draft 機能で表す"
    if m.group("subject").rstrip()[-1] in ".。":
        return "subject を句点で終えない"
    return None


def _selftest() -> None:
    ok = [
        "chore: 🔧 crate 骨組みと CI を追加",
        "feat: ✨ 注釈付き PGN のパーサを実装",
        "feat!: 💥 注釈の書式を変更",
    ]
    ng = [
        "",
        "crate 骨組みと CI を追加",
        "chore: crate 骨組みと CI を追加",
        "chore: 🔧 422 を構造的にゼロ化 (#157)",
        "chore: 🔧 対応 #12",
        "chore: 🔧 [WIP] 途中",
        "chore: 🔧 追加した。",
        "chore: 🔧 " + "あ" * MAX_LENGTH,
    ]
    for t in ok:
        assert validate(t) is None, f"通るべきが落ちた: {t!r} -> {validate(t)}"
    for t in ng:
        assert validate(t) is not None, f"落ちるべきが通った: {t!r}"
    print("selftest ok")


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <title> | --selftest", file=sys.stderr)
        return 2
    if sys.argv[1] == "--selftest":
        _selftest()
        return 0
    error = validate(sys.argv[1])
    if error:
        print(f"PR タイトルが規約に反する: {error}", file=sys.stderr)
        print(f"  実際: {sys.argv[1]!r}", file=sys.stderr)
        print("  書式: <type>: <emoji> <subject>", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
