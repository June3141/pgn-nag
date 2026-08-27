# Architecture Decision Records

`pgn-nag` の設計判断の記録。
判断そのものだけでなく、なぜその案を選び、なぜ他の案を採らなかったかを残す。

## 記録の単位

後から「なぜこうなっているのか」を問われうる判断を 1 件 1 ファイルにする。
実装の詳細や、コードを読めば分かることは記録しない。

判断が実装の過程で変わった場合は、該当する ADR を更新して追認する。
記録と実装が食い違ったまま放置しない。

## 命名と番号

ファイル名は `NNNN-英語の短い要約.md` とする。
番号は既存の最大値に 1 を加える。

## status

| status | 意味 |
| --- | --- |
| `proposed` | 提案中で、まだ合意されていない |
| `accepted` | 採用され、実装の前提になっている |
| `superseded` | 後続の ADR に置き換えられた |

`superseded` にする場合は、置き換え先の ADR へのリンクを本文に書く。

## 書き方

[template.md](template.md) をコピーして書き始める。
MADR に沿った構成で、`Considered Options` には却下した案も必ず書く。
`Consequences` は利点と欠点の双方を書き、欠点には緩和策か、何と引き換えに受け入れるかを添える。

## 運用

この index は全 ADR が触る直列化点になる。
複数の ADR を並行して出すときは、番号順に線形へ積んで衝突を避ける。

まだ merge されていない後続 ADR への参照は、リンクにせず本文中の記述に留める。
その branch では参照先が存在せず、リンクが切れるためである。
リンク切れは CI では検出されないので、追加時に自分で確認する。

## 一覧

| 番号 | タイトル | status |
| --- | --- | --- |
| 0001 | [解析と閲覧を工程分離する](0001-separate-analysis-from-viewing.md) | accepted |
| 0002 | [中間形式に注釈付き PGN を使う](0002-use-annotated-pgn-as-interchange-format.md) | accepted |
| 0003 | [実装言語に Rust を選ぶ](0003-implement-in-rust.md) | accepted |
| 0004 | [PGN の書き出しを独立した crate にしない](0004-keep-the-pgn-writer-internal.md) | accepted |
| 0005 | [v1 は主手順のみを対象とする](0005-support-mainline-only-in-v1.md) | accepted |
| 0006 | [導出値を保存しない](0006-do-not-store-derived-values.md) | accepted |
| 0007 | [CI は engine 非依存テストを主軸にする](0007-keep-ci-independent-of-the-engine.md) | accepted |
| 0008 | [評価値の出所を記録する](0008-record-the-source-of-evaluations.md) | accepted |
| 0009 | [キーバインドを端末ツールの慣行に揃える](0009-follow-terminal-key-binding-conventions.md) | accepted |
| 0010 | [複雑度を CI でゲートする](0010-gate-complexity-in-ci.md) | accepted |
| 0011 | [viewer を即時モードで描画する](0011-render-the-viewer-in-immediate-mode.md) | accepted |
| 0012 | [保持する状態を設定だけに限る](0012-keep-state-in-config-only.md) | accepted |
