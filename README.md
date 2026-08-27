# pgn-nag

PGN の棋譜を UCI エンジンで一括解析し、注釈付き PGN として保存して、端末上で読み返すためのツール。

コマンド名は `nag` になる。
NAG は Numeric Annotation Glyph の略で、`?` や `??` といった手の評価記号を PGN 上で表す記法を指す。

開発中で、まだ動作するものはない。
設計判断は [docs/adr/README.md](docs/adr/README.md) にある。

## 何をするか

解析と閲覧を 2 つの工程に分け、間を注釈付き PGN で繋ぐ。

```text
PGN ──[ nag analyze ]──> annotated PGN ──[ nag view ]──> 端末 UI
         engine を使う                      engine を使わない
```

解析を事前に済ませるため、閲覧時に engine を起動しない。
生成されるファイルは Lichess をはじめとする既存のツールでも開ける。

## 想定する使い方

```bash
nag analyze games/ --depth 18 -o annotated.pgn
nag view annotated.pgn
```

`analyze` は棋譜の各局面を解析し、評価値と最善手順を注釈として書き込む。
`view` は盤面と評価値を並べて表示し、悪手を辿って移動できる。

## 進め方

各段階と完了条件は [milestone](https://github.com/June3141/pgn-nag/milestones) にある。

## ライセンス

GPL-3.0-or-later。

盤面と PGN の処理に使っている `shakmaty` と `pgn-reader` が GPL-3.0-or-later のため、静的リンクした配布物は同じ条件で提供する。
判断の経緯は [ADR-0013](docs/adr/0013-license-under-gpl-3.md) にある。
