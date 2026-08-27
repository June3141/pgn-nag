//! 注釈付き PGN を読み込んだ結果の表現。

use shakmaty::Chess;

/// 評価値。常に白視点で保持する。
///
/// UCI は手番視点の値を返すため、白視点への変換を 1 箇所に閉じ込める。
/// 変換を通さない経路を作ると、符号の誤りが静かに混ざる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    /// centipawn。
    Cp(i32),
    /// 詰みまでの手数。正が白の勝ち、負が黒の勝ち。
    ///
    /// centipawn に潰すと詰み手数が失われるため、別の値として持つ。
    Mate(i32),
}

/// 1 局面の評価。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eval {
    pub score: Score,
    /// 到達した探索の深さ。注釈に深さを持たない PGN もあるため Option にする。
    pub depth: Option<u32>,
}

/// 1 手と、その手を指した後の局面。
#[derive(Debug, Clone)]
pub struct Ply {
    pub san: String,
    /// この手を指した後の局面。
    pub position: Chess,
    /// 注釈が無い手が実在するため Option にする。
    /// 非 Option にすると、checkmate で終わる棋譜の最終手で必ず破綻する。
    pub eval: Option<Eval>,
    /// 最善手順。UCI 表記のまま保持する。
    pub pv: Vec<String>,
    /// 注釈の生のテキスト。波括弧は含まない。
    ///
    /// 書き出しはこれをそのまま出す。
    /// `eval` と `pv` はここから導出した値なので、書き換えるときは
    /// 両方を揃えないと、保持している値と出力が食い違う。
    /// 解釈しない `[%clk]` 等を落とさないために生のまま持つ。
    pub comment: Option<String>,
}

/// 1 対局。
#[derive(Debug, Clone)]
pub struct Game {
    /// タグは出現順のまま保持する。往復一致には順序が要る。
    pub tags: Vec<(String, String)>,
    pub plies: Vec<Ply>,
    /// 終局結果の表記。`1-0` `0-1` `1/2-1/2` `*` のいずれか。
    pub outcome: String,
}

/// 手順リストと PGN の双方で使う手番号の表記。
///
/// 白の手は `1.`、黒の手は `1...` になる。
/// 表記が 2 箇所に分かれると、片方だけ直る事故が起きる。
pub fn move_marker(index: usize) -> String {
    let number = index / 2 + 1;
    if index.is_multiple_of(2) {
        format!("{number}.")
    } else {
        format!("{number}...")
    }
}

impl Score {
    /// 画面に出す表記。詰みは centipawn に潰さない。
    ///
    /// 手順リストと状態行の双方で使う。
    /// 表記が 2 箇所に分かれると、片方だけ直る事故が起きる。
    pub fn render(self) -> String {
        match self {
            Self::Cp(cp) => format!("{:+.2}", f64::from(cp) / 100.0),
            Self::Mate(n) => format!("#{n}"),
        }
    }
}

impl Game {
    /// タグの値。無ければ `?` を返す。
    ///
    /// 一覧の見出しと盤の見出しの双方で使う。
    pub fn tag(&self, name: &str) -> &str {
        self.tags
            .iter()
            .find(|(k, _)| k == name)
            .map_or("?", |(_, v)| v.as_str())
    }
}
