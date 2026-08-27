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

impl Ply {
    /// 注釈を持つか。
    pub fn has_comment(&self) -> bool {
        self.eval.is_some() || !self.pv.is_empty()
    }

    /// `{ [%eval ...] [%pv ...] }` の形に組み立てる。注釈が無ければ None。
    pub fn comment(&self) -> Option<String> {
        if !self.has_comment() {
            return None;
        }
        let mut parts = Vec::new();
        if let Some(eval) = self.eval {
            parts.push(format!("[%eval {}]", eval.render()));
        }
        if !self.pv.is_empty() {
            parts.push(format!("[%pv {}]", self.pv.join(" ")));
        }
        Some(format!("{{ {} }}", parts.join(" ")))
    }
}

impl Eval {
    /// `0.32,18` や `#-1,18` の形に戻す。
    pub fn render(&self) -> String {
        let value = match self.score {
            // centipawn は小数 2 桁のポーン単位で書く
            Score::Cp(cp) => format!("{:.2}", f64::from(cp) / 100.0),
            Score::Mate(n) => format!("#{n}"),
        };
        match self.depth {
            Some(d) => format!("{value},{d}"),
            None => value,
        }
    }
}
