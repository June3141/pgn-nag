---
status: accepted
date: 2026-08-26
---

# CI は engine 非依存テストを主軸にする

## Context and Problem Statement

テストに engine を要求すると、CI の各ジョブで engine の導入が必要になる。
また、engine が返す評価値をテストの期待値に使うと、engine のバージョンに結果が左右される。

## Decision Drivers

* CI の実行時間と設定の複雑さを抑えたい。
* engine の更新でテストが壊れる状態を避けたい。

## Considered Options

* engine を必要とするテストを分離し、既定では実行しない。
* すべてのテストで engine を要求する。

## Decision Outcome

engine を必要とするテストを分離し、既定では実行しない。

engine に触れるモジュールを 1 つに限定した結果、解析の読み書きと集計のテストは engine 無しで完結する。
engine を要する統合テストだけを別のジョブに切り出す。

engine を使うテストでは、評価値の一致を期待値にしない。
評価値の尺度は engine のバージョンによって変わるため、値の一致を合格条件にすると engine の更新で必ず壊れる。
代わりに、白視点への変換が符号として正しいこと、探索途中の境界値が除外されること、終局局面で評価値が存在しないことという性質を検証する。

engine の取得には配布元のパッケージを使う。
公式の配布物を直接取得する必要が生じた場合は、ハッシュを固定して検証する。

### Confirmation

engine を導入していない環境で、既定のテストがすべて通ることを確認する。

## Consequences

### Good

* 大半のジョブが engine の導入を必要とせず、CI の設定と実行時間が軽くなる。
* engine を更新してもテストが壊れない。

### Bad

* engine との結合部分が、既定のテストで検証されない。
  分離したジョブを CI で実行し、検証自体は継続する。
* 評価値の正しさそのものは検証できない。
  性質の検証で誤りの大半を捉えられると判断し、値の正しさは engine 側の責任として扱う。

## More Information

* [ADR-0001 解析と閲覧を工程分離する](0001-separate-analysis-from-viewing.md)
* [ADR-0008 評価値の出所を記録する](0008-record-the-source-of-evaluations.md)
