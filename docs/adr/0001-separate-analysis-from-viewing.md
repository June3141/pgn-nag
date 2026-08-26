---
status: accepted
date: 2026-08-26
---

# 解析と閲覧を工程分離する

## Context and Problem Statement

解析機能を持つ棋譜ビューアを作る。
engine をいつ動かすかによって、実装の難易度と成果物の性質が大きく変わる。

対話的に解析する設計では、engine プロセスを長時間保持し、探索途中の情報を UI スレッドへ流し込む仕組みが要る。
この同期処理が実装の中心になり、閲覧機能そのものより大きくなる。

## Decision Drivers

* viewer の実装量を小さく保ちたい。
* 開発を途中で止めても成果が残るようにしたい。
* 対話的な解析では現実的でない深さまで探索したい。

## Considered Options

* 事前にバッチ解析し、viewer は engine を持たない。
* viewer が engine を常駐させ、局面を移動するたびに解析する。
* viewer が局面ごとに engine プロセスを起動する。

## Decision Outcome

事前にバッチ解析し、viewer は engine を持たない設計を選ぶ。

工程を 2 つに割り、間を注釈付き PGN というファイルで繋ぐ。
`analyze` が engine を使ってファイルを生成し、`view` はそのファイルだけを読む。

局面ごとに engine を起動する案は、起動のたびにハッシュテーブルが空になり探索が浅くなるため採らない。

### Confirmation

`view` 側のモジュールが engine モジュールへ依存していないことを、モジュール構成で確認する。
engine を使わずにビルドしたバイナリで閲覧が完結することを、統合テストで確認する。

## Consequences

### Good

* UI スレッドと engine の同期処理が不要になり、実装の山場が消える。
* 中間ファイルが他のツールでも開けるため、viewer の完成前から解析結果を使える。
* 対話的な解析では待てない深さを、一晩かけて処理できる。
* テストの大半が engine 無しで動くため、CI が単純になる。

### Bad

* 解析していない棋譜は評価値なしでしか見られない。
  未注釈の PGN も再生だけはできる経路を残し、閲覧そのものは阻害しない。
* 深さを変えるにはファイルの再生成が必要になる。
  `analyze` を冪等に作り、再実行の敷居を下げることで受け入れる。

## More Information

* [ADR-0002 中間形式に注釈付き PGN を使う](0002-use-annotated-pgn-as-interchange-format.md)
* [ADR-0007 CI は engine 非依存テストを主軸にする](0007-keep-ci-independent-of-the-engine.md)
