# oracle

Rust 実装の移植を検証するための Python 実装。
製品コードではなく、参照用として残している。

局面ごとに `ucinewgame` を送って置換表を捨てるため、同じ engine と同じ深さなら解析順に依らず同じ値が出る。
Rust 版の出力との差分がそのまま回帰テストになる。
置換表を持ち越すと、同じ棋譜でもファイル内の位置によって値が変わり、この比較が成立しない。

engine の `Hash` と `Threads` も値を変える。
差分検証のときは engine option も揃えること。

`analyse()` の戻り値に含まれる `lowerbound` と `upperbound` は探索全体の累積で、score が確定値でも真になりうる。
探索途中の境界値を弾く目的で使ってはいけない。
`go depth N` は反復の完了後に確定値を返すため、この実装は境界値を踏まない。

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

注釈が落ちるのは checkmate 局面だけで、stalemate には `0.00` が入る。
python-chess の `set_eval` が `Mate(0)` を falsy として捨てるためで、チェスの性質ではない。
Rust 実装が `#0` を忠実に書くと、この出力とは差が出る。

Rust 実装が M3 まで到達し、出力の一致を確認したら削除してよい。
