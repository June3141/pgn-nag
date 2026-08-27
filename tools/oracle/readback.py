# /// script
# dependencies = ["chess"]
# ///
"""注釈付き PGN を engine なしで読み、blunder を導出する (= viewer 側がやる仕事)。"""
import sys, chess.pgn
g = chess.pgn.read_game(open(sys.argv[1]))
prev = None
for node in g.mainline():
    cp = node.eval().white().score(mate_score=10000)
    mover_white = not node.board().turn  # push 済みなので turn は相手
    loss = None if prev is None else ((cp - prev) if mover_white else (prev - cp))
    tag = " <-- ??" if loss is not None and loss < -200 else ""
    print(f"{node.san():8} {cp/100:+.2f}{tag}")
    prev = cp
