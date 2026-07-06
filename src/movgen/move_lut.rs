use std::ops::Add;

use strum::IntoEnumIterator;

use crate::{
    movgen::{
        bitboard::{AntiDiagonal, BitBoard, ContainingSquare, Diagonal, File, Rank},
        piece::{
            Color::{Black, White},
            Piece,
        },
    },
    square::{SQUARES, Square},
};

/// A lookup table of bitboards for each square on the chess board.
/// This can be used to e.g. store the available pseudo-legal moves a piece has from every square.
/// A move is pseudo-legal if it assumes an otherwise empty board.
type BitBoardLUT = [BitBoard; 64];

struct MoveLUT(BitBoardLUT);

impl MoveLUT {
    /// Get the available pseudo-legal moves from a given start square.
    pub fn get(&self, square: &Square) -> &BitBoard {
        self.0.get(square.idx).expect("idx in [0, 63]")
    }
}

/// Intersect two iterators at a square and take their union.
///
/// E.g., the square e3 intersects at the e-file and the 3rd rank.
///
/// ```text
/// 8 . . . . | . . .
/// 7 . . . . | . . .
/// 6 . . . . | . . .
/// 5 . . . . | . . .
/// 4 . . . . | . . .
/// 3 --------+------
/// 2 . . . . | . . .
/// 1 . . . . | . . .
///   a b c d e f g h
/// ```
fn union_at_intersection<T, U>(
    square: &Square,
    mut iter1: impl ContainingSquare<T>,
    mut iter2: impl ContainingSquare<U>,
) -> u64
where
    T: Copy + Into<u64>,
    U: Copy + Into<u64>,
{
    iter1.find_containing_square(square.board).unwrap()
        | iter2.find_containing_square(square.board).unwrap()
}

fn generate_bishop_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        boards[square.idx] = BitBoard::new(
            union_at_intersection(square, Diagonal::iter(), AntiDiagonal::iter()) ^ square.board,
        );
    }
    boards
}

fn generate_rook_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        boards[square.idx] =
            BitBoard::new(union_at_intersection(square, File::iter(), Rank::iter()) ^ square.board);
    }
    boards
}

fn generate_queen_lut() -> BitBoardLUT {
    let mut rook_lut = generate_rook_lut();
    rook_lut
        .iter_mut()
        .zip(generate_bishop_lut())
        .for_each(|(rook, bishop)| {
            *rook |= bishop;
        });
    rook_lut
}

fn generate_knight_lut() -> BitBoardLUT {
    const KNIGHT_JUMPS: [u64; 4] = [6, 10, 15, 17];
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let mut ranks: u64 = 0;
        for added in [Rank::One, Rank::Two] {
            ranks |= u64::from(rank.saturating_add(&added));
            ranks |= u64::from(rank.saturating_sub(&added));
        }

        let file = File::try_from(square.file).unwrap();
        let mut files: u64 = 0;
        for added in [File::A, File::B] {
            files &= u64::from(file.saturating_add(&added));
            files &= u64::from(file.saturating_sub(&added));
        }

        let mut knight = 0;
        for jump in KNIGHT_JUMPS {
            knight |= square.board.add(jump) | square.board.saturating_sub(jump);
        }
        knight &= (ranks & files) ^ square.board;
        boards[square.idx] = BitBoard::from(knight);
    }
    boards
}

fn generate_king_lut() -> BitBoardLUT {
    todo!()
}

fn generate_pawn_white_lut() -> BitBoardLUT {
    todo!()
}

fn generate_pawn_black_lut() -> BitBoardLUT {
    todo!()
}

// TODO: index by piece somehow, distinguishing white/black pawns
// Generate moves from ranks etc.:
// - bishop: intersecting diagonals
// - rook: intersecting rank + file
// - queen: intersecting all
// - king: 1s but clip to rows/col +- 1
// - knight: 1s clipped to row/col +- 1 and NOT rook or bishop
struct PieceLUT {
    bishop: MoveLUT,
    knight: MoveLUT,
    rook: MoveLUT,
    queen: MoveLUT,
    king: MoveLUT,
    pawn_white: MoveLUT,
    pawn_black: MoveLUT,
}

impl PieceLUT {
    pub fn new() -> Self {
        Self {
            bishop: MoveLUT(generate_bishop_lut()),
            knight: MoveLUT(generate_king_lut()),
            rook: MoveLUT(generate_rook_lut()),
            queen: MoveLUT(generate_queen_lut()),
            king: MoveLUT(generate_king_lut()),
            pawn_white: MoveLUT(generate_pawn_white_lut()),
            pawn_black: MoveLUT(generate_pawn_black_lut()),
        }
    }

    pub fn get(&self, piece: &Piece, square: &Square) -> &BitBoard {
        match *piece {
            Piece::Bishop => self.bishop.get(square),
            Piece::Knight => self.knight.get(square),
            Piece::Rook => self.rook.get(square),
            Piece::Queen => self.queen.get(square),
            Piece::King => self.king.get(square),
            Piece::Pawn(White) => self.pawn_white.get(square),
            Piece::Pawn(Black) => self.pawn_black.get(square),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::movgen::{
        bitboard::BitBoard,
        move_lut::{BitBoardLUT, generate_bishop_lut},
    };

    #[repr(usize)]
    #[derive(Debug, Clone, Copy)]
    enum TestSquare {
        A1 = 0,
        H1 = 7,
        A8 = 56,
        H8 = 63,
        D4 = 27,
        E4 = 28,
    }

    #[test]
    fn test_bishop_lut() {
        let lut = generate_bishop_lut();
        helper(
            lut,
            TestSquare::A1,
            vec![
                9, 18, 27, 36, 45, 54, 63, // b2, c3, d4, e5, f6, g7, h8
            ],
        );
        helper(
            lut,
            TestSquare::H1,
            vec![
                14, 21, 28, 35, 42, 49, 56, // g2, f3, e4, d5, c6, b7, a8
            ],
        );
        helper(
            lut,
            TestSquare::A8,
            vec![
                7, 14, 21, 28, 35, 42, 49, // h1, g2, f3, e4, d5, c6, b7, a8
            ],
        );
        helper(
            lut,
            TestSquare::H8,
            vec![
                0, 9, 18, 27, 36, 45, 54, // a1, b2, c3, d4, e5, f6, g7
            ],
        );
        helper(
            lut,
            TestSquare::D4,
            vec![
                0, 9, 18, 36, 45, 54, 63, // a1, b2, c3, e5, f6, g7, h8
                6, 13, 20, 34, 41, 48, // g1, f2, e3, c5, b6, a7
            ],
        );
        helper(
            lut,
            TestSquare::E4,
            vec![
                7, 14, 21, 35, 42, 49, 56, // h1, g2, f3, d5, c6, b7, a8
                1, 10, 19, 37, 46, 55, // b1, c2, d3, f5, g6, h7
            ],
        );
    }

    #[test]
    fn test_rook_lut() {
        todo!()
    }

    #[test]
    fn test_queen_lut() {
        todo!()
    }

    #[test]
    fn test_knight_lut() {
        todo!()
    }

    #[test]
    fn test_king_lut() {
        todo!()
    }

    #[test]
    fn test_pawn_white_lut() {
        todo!()
    }

    #[test]
    fn test_pawn_black_lut() {
        todo!()
    }

    fn helper(lut: BitBoardLUT, square: TestSquare, squares: Vec<u64>) {
        let idx = square as usize;
        let actual = lut[idx];
        let expected = squares.into_iter().fold(0, |acc, e| acc | 1 << e);
        assert_eq!(actual, BitBoard::new(expected), "{:?}", square);
    }
}
