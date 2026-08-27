# /// script
# dependencies = ["chess"]
# ///
"""PGN を stockfish で一括解析し、{[%eval ...]} 注釈付き PGN として書き出す。

usage: uv run pgn_annotate.py <in.pgn|dir> [-o out.pgn] [--depth 18]

出力は Lichess 互換の annotated PGN。lichess / En Croissant / python-chess が
そのまま読める。blunder 判定は eval から毎回導出できるので保存しない
(閾値を後から変えられるように)。
"""

import argparse, os, pathlib, shutil, sys
import chess, chess.engine, chess.pgn

PV_PLIES = 6  # ADR-0002。保存する最善手順の上限

# 実行パスは PGN_NAG_ENGINE か PATH から解決する。直書きすると環境を跨げない。
ENGINE = os.environ.get("PGN_NAG_ENGINE") or shutil.which("stockfish")
if not ENGINE:
    sys.exit("stockfish が見つからない。PGN_NAG_ENGINE でパスを指定するか PATH に通す")


def annotate(game, eng, depth):
    """各手に [%eval] と [%pv] を書き込む。評価は常に白視点で保存する。"""
    # centipawn の尺度は engine とそのバージョンに依存する。出所を残さないと
    # 異なる条件で解析した値が混ざったことに後から気付けない
    if "name" not in eng.id:
        # 出所不明の値を書くと、条件の違う評価値が混ざったことを検出できなくなる
        sys.exit("engine が id name を返さない。Annotator を記録できない")
    game.headers["Annotator"] = eng.id["name"]
    node = game
    while node.variations:
        node = node.variations[0]
        # game を局面ごとに変えると python-chess が ucinewgame を送る。
        # 送らないと置換表が局面をまたいで残り、評価値が解析順に依存する。
        # 同じ棋譜でもファイル内の位置が違えば別の値になり、差分比較が成立しない
        info = eng.analyse(node.board(), chess.engine.Limit(depth=depth), game=object())
        # set_eval は PovScore を受け、白視点の [%eval] として直列化する
        node.set_eval(info["score"], info.get("depth", depth))
        if pv := info.get("pv"):
            pv_uci = " ".join(m.uci() for m in pv[:PV_PLIES])
            # %pv は非標準だが、未知の %tag は他ツールが黙って無視する
            node.comment = f"{node.comment} [%pv {pv_uci}]".strip()
    return game


def main():
    p = argparse.ArgumentParser()
    p.add_argument("path", help="PGN ファイル、または .pgn を含むディレクトリ")
    p.add_argument("-o", "--out", help="出力先 (既定: <input>.annotated.pgn)")
    p.add_argument("-d", "--depth", type=int, default=18)
    a = p.parse_args()

    src = pathlib.Path(a.path)
    files = sorted(src.glob("*.pgn")) if src.is_dir() else [src]
    if not files:
        sys.exit(f"no .pgn under {src}")
    out = pathlib.Path(a.out) if a.out else src.with_suffix(".annotated.pgn")

    with chess.engine.SimpleEngine.popen_uci(ENGINE) as eng, out.open("w") as w:
        n = 0
        for f in files:
            with f.open() as fh:
                while (game := chess.pgn.read_game(fh)) is not None:
                    n += 1
                    print(f"[{n}] {f.name}: {game.headers.get('White')} vs "
                          f"{game.headers.get('Black')} ...", file=sys.stderr, flush=True)
                    print(annotate(game, eng, a.depth), file=w, end="\n\n")
    print(f"wrote {n} game(s) -> {out}", file=sys.stderr)


if __name__ == "__main__":
    main()
