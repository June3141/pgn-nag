# /// script
# dependencies = ["chess"]
# ///
"""注釈付き PGN を engine なしで読み、blunder を導出する (= viewer 側がやる仕事)。"""
import sys, chess.pgn

BLUNDER_CP = -200  # 閾値の探索がこの実装を残す理由の 1 つなので、式に埋めない
g = chess.pgn.read_game(open(sys.argv[1]))
prev = None
for node in g.mainline():
    score = node.eval()
    if score is None:
        # 注釈が無い手。checkmate 局面がこれになる。python-chess の set_eval が
        # Mate(0) を falsy と扱って捨てるためで、stalemate には 0.00 が入る。
        # None 前提にすると詰みで終わる棋譜の最終手で落ちる
        print(f"{node.san():8} -")
        prev = None  # 基準を持ち越すと次の手の損失が 2 手前を基準に出る
        continue
    cp = score.white().score(mate_score=10000)
    mover_white = not node.board().turn  # push 済みなので turn は相手
    loss = None if prev is None else ((cp - prev) if mover_white else (prev - cp))
    tag = " <-- ??" if loss is not None and loss < BLUNDER_CP else ""
    print(f"{node.san():8} {cp/100:+.2f}{tag}")
    prev = cp
