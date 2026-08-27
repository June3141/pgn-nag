//! 盤面の描画。

use ratatui::text::{Line, Span};
use shakmaty::{Board, Color, File, Rank, Role, Square};

/// 盤面を白側から見た 8 行として組み立てる。
///
/// 空きマスは `.` にする。空白にすると列が数えにくく、盤の枠との対応も取れない。
pub fn lines(board: &Board) -> Vec<Line<'static>> {
    let mut out = Vec::with_capacity(9);
    for rank in Rank::ALL.into_iter().rev() {
        let mut spans = vec![Span::raw(format!("{}  ", rank.char()))];
        for file in File::ALL {
            spans.push(Span::raw(format!(
                "{} ",
                glyph(board, Square::from_coords(file, rank))
            )));
        }
        out.push(Line::from(spans));
    }
    out.push(Line::from("   a b c d e f g h"));
    out
}

/// 駒を 1 文字で表す。白は大文字、黒は小文字。
fn glyph(board: &Board, square: Square) -> char {
    match board.piece_at(square) {
        None => '.',
        Some(piece) => {
            let c = match piece.role {
                Role::Pawn => 'p',
                Role::Knight => 'n',
                Role::Bishop => 'b',
                Role::Rook => 'r',
                Role::Queen => 'q',
                Role::King => 'k',
            };
            match piece.color {
                Color::White => c.to_ascii_uppercase(),
                Color::Black => c,
            }
        }
    }
}
