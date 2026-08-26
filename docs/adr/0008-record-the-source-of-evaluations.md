---
status: accepted
date: 2026-08-26
---

# 評価値の出所を記録する

## Context and Problem Statement

評価値の尺度は engine ごとに異なり、同じ engine でもバージョンによって変わる。
Stockfish は内部値を定数で割って出力しており、その定数は過去に 348、361、394 と変化している。

出所を記録しないと、異なる条件で解析した評価値が同じファイルに混在したとき、それを判別できない。

## Decision Drivers

* 比較できない評価値どうしを比較してしまう事故を防ぎたい。
* 再解析が必要かどうかを判断できるようにしたい。

## Considered Options

* engine の名前とバージョンを PGN のヘッダに記録する。
* 記録せず、注釈に含まれる深さだけで判断する。

## Decision Outcome

engine の名前とバージョンを PGN の `Annotator` ヘッダに記録する。

```text
[Annotator "Stockfish 18"]
```

`Annotator` は棋譜に注釈を付けた主体を示す標準のヘッダであり、この用途に合う。

深さだけで判断する案は採らない。
より浅い深さでも新しい engine のほうが精度が高い場合があり、深さの比較では再解析の要否を判断できない。

既定の engine には Stockfish を使う。
最も強い engine が無償かつ GPU を要求しない状態にあるため、商用の engine を検討する理由がない。
Lc0 は評価が勝率に基づくため中心視点の値と噛み合わず、置き換えではなく併用の候補として扱う。

### Confirmation

生成したファイルに `Annotator` ヘッダが含まれることを確認する。
`Annotator` が異なる棋譜どうしで、平均損失が同じ集計単位に混ざらないことを確認する。

## Consequences

### Good

* 異なる条件で解析された評価値の混在を検出できる。
* 再解析の要否を、engine の同一性を含めて判断できる。

### Bad

* 平均損失を engine 横断で集計できない。
  集計単位を `Annotator` で分けることで、意味のない比較を防ぐ。
* engine を更新すると、過去の棋譜との比較のために再解析が必要になる。
  比較が必要な場面に限って再解析する運用とし、全件の再生成は求めない。

## More Information

* [ADR-0002 中間形式に注釈付き PGN を使う](0002-use-annotated-pgn-as-interchange-format.md)
* [ADR-0006 導出値を保存しない](0006-do-not-store-derived-values.md)
