# oracle

Rust 実装の移植を検証するための Python 実装。
製品コードではなく、参照用として残している。

同じ engine と同じ深さで動かせば評価値は決定的に一致するため、Rust 版の出力との差分がそのまま回帰テストになる。

engine の実行パスは `PGN_NAG_ENGINE` か `PATH` から解決する。

```bash
uv run tools/oracle/pgn_annotate.py tests/data/sample.pgn --depth 18 -o /tmp/out.pgn
uv run tools/oracle/readback.py /tmp/out.pgn
```

`pgn_annotate.py` は棋譜を解析して注釈付き PGN を書き出す。
`readback.py` は engine を使わずに注釈を読み戻し、悪手を導出する。

engine のバージョンが違えば評価値も変わる。
差分検証は同一の engine で行うこと。
CI の合格条件には使わない。

Rust 実装が M3 まで到達し、出力の一致を確認したら削除してよい。
