//! It would be wasteful to compute legal moves for a given piece continuously at runtime.
//! Therefore, these moves are stored in a lookup table [`MoveLUT`] for each piece.
//! The overarching [`PieceLUT`] can be constructed once during engine initialization and continuously queried
//! with `O(1)` lookups of legal moves for a given piece on a given square.
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

/// The knight can move to the opposite squares of the queen in a 5x5 grid.
///
/// ```text
/// x . x . x
/// . x x x .
/// x x x x x
/// . x x x .
/// x . x . x
/// ```
fn generate_knight_lut() -> BitBoardLUT {
    let mut queen = generate_queen_lut();
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let mut ranks: u64 = 0;
        for added in [1, 2] {
            ranks |= u64::from(rank.saturating_add(added));
            ranks |= u64::from(rank.saturating_sub(added));
        }

        let file = File::try_from(square.file).unwrap();
        let mut files: u64 = 0;
        for added in [1, 2] {
            files |= u64::from(file.saturating_add(added));
            files |= u64::from(file.saturating_sub(added));
        }

        queen[square.idx] = BitBoard::new(ranks & files & !square.board) & !queen[square.idx];
    }
    queen
}

fn generate_king_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let ranks =
            u64::from(rank) | u64::from(rank.saturating_add(1)) | u64::from(rank.saturating_sub(1));
        let file = File::try_from(square.file).unwrap();
        let files =
            u64::from(file) | u64::from(file.saturating_add(1)) | u64::from(file.saturating_sub(1));
        boards[square.idx] = BitBoard::new(ranks & files & !square.board);
    }
    boards
}

fn generate_pawn_white_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let file = u64::from(File::try_from(square.file).unwrap());
        let ranks = ((rank >= Rank::Two) as u64 * u64::MAX) & u64::from(rank.saturating_add(1))
            | ((rank == Rank::Two) as u64 * u64::MAX) & u64::from(rank.saturating_add(2));
        boards[square.idx] = BitBoard::new(ranks & file & !square.board);
    }
    boards
}

fn generate_pawn_black_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let file = u64::from(File::try_from(square.file).unwrap());
        let ranks = ((rank <= Rank::Seven) as u64 * u64::MAX) & u64::from(rank.saturating_sub(1))
            | ((rank == Rank::Seven) as u64 * u64::MAX) & u64::from(rank.saturating_sub(2));
        boards[square.idx] = BitBoard::new(ranks & file & !square.board);
    }
    boards
}

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
            Piece::Bishop(_) => self.bishop.get(square),
            Piece::Knight(_) => self.knight.get(square),
            Piece::Rook(_) => self.rook.get(square),
            Piece::Queen(_) => self.queen.get(square),
            Piece::King(_) => self.king.get(square),
            // The pawns are distinguished by color as their move bitboards are unique i.e., they move in opposite directions.
            Piece::Pawn(White) => self.pawn_white.get(square),
            Piece::Pawn(Black) => self.pawn_black.get(square),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::movgen::{
        bitboard::BitBoard,
        move_lut::{
            BitBoardLUT, generate_bishop_lut, generate_king_lut, generate_knight_lut,
            generate_pawn_black_lut, generate_pawn_white_lut, generate_queen_lut,
            generate_rook_lut,
        },
    };

    #[repr(usize)]
    #[derive(Debug, Clone, Copy)]
    enum TestSquare {
        A1 = 0,
        H1 = 7,
        C2 = 10,
        D4 = 27,
        E4 = 28,
        F7 = 53,
        A8 = 56,
        H8 = 63,
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
            TestSquare::C2,
            vec![
                1, 19, 28, 37, 46, 55, // b1, d3, e4, f5, g6, h7
                3, 17, 24, // d1, b3 a4
            ],
        );
        helper(
            lut,
            TestSquare::F7,
            vec![
                39, 46, 60, // h5,g6,e8
                8, 17, 26, 35, 44, 62, // a2, b3, c4, d5, e6, g8
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
        let lut = generate_rook_lut();
        helper(
            lut,
            TestSquare::A1,
            vec![
                1, 2, 3, 4, 5, 6, 7, // b1, c1, d1, e1, f1, g1, h1
                8, 16, 24, 32, 40, 48, 56, // a2, a3, a4, a5, a6, a7, a8
            ],
        );
        helper(
            lut,
            TestSquare::H1,
            vec![
                0, 1, 2, 3, 4, 5, 6, // a1, b1, c1, d1, e1, f1, g1, h1
                15, 23, 31, 39, 47, 55, 63, // h2, h3, h4, h5, h6, h7, h8
            ],
        );
        helper(
            lut,
            TestSquare::A8,
            vec![
                0, 8, 16, 24, 32, 40, 48, // a1, a2, a3, a4, a5, a6, a7
                57, 58, 59, 60, 61, 62, 63, // b8, c8, d8, e8, f8, g8, h8
            ],
        );
        helper(
            lut,
            TestSquare::H8,
            vec![
                7, 15, 23, 31, 39, 47, 55, // h1, h2, h3, h4, h5, h6, h7
                56, 57, 58, 59, 60, 61, 62, // a8, b8, c8, d8, e8, f8, g8
            ],
        );
        helper(
            lut,
            TestSquare::C2,
            vec![
                2, 18, 26, 34, 42, 50, 58, // c1, c3, c4, c5, c6, c7, c8
                8, 9, 11, 12, 13, 14, 15, // a2, b2, d2, e2, f2, g2, h2
            ],
        );
        helper(
            lut,
            TestSquare::F7,
            vec![
                5, 13, 21, 29, 37, 45, 61, // f1, f2, f3, f4, f5, f6, f8
                48, 49, 50, 51, 52, 54, 55, // a7, b7, d7, e7, f7, g7, h7
            ],
        );
        helper(
            lut,
            TestSquare::D4,
            vec![
                3, 11, 19, 35, 43, 51, 59, // d1, d2, d3, d5, d6, d7, d8
                24, 25, 26, 28, 29, 30, 31, // a4, b4, c4, e4, f4, g4, h4
            ],
        );
        helper(
            lut,
            TestSquare::E4,
            vec![
                4, 12, 20, 36, 44, 52, 60, // e1, e2, e3, e5, e6, e7, e8
                24, 25, 26, 27, 29, 30, 31, // a4, b4, c4, d4, f4, g4, h4
            ],
        );
    }

    #[test]
    fn test_queen_lut() {
        let lut = generate_queen_lut();
        helper(
            lut,
            TestSquare::A1,
            vec![
                9, 18, 27, 36, 45, 54, 63, // b2, c3, d4, e5, f6, g7, h8
                1, 2, 3, 4, 5, 6, 7, // b1, c1, d1, e1, f1, g1, h1
                8, 16, 24, 32, 40, 48, 56, // a2, a3, a4, a5, a6, a7, a8
            ],
        );
        helper(
            lut,
            TestSquare::H1,
            vec![
                14, 21, 28, 35, 42, 49, 56, // g2, f3, e4, d5, c6, b7, a8
                0, 1, 2, 3, 4, 5, 6, // a1, b1, c1, d1, e1, f1, g1, h1
                15, 23, 31, 39, 47, 55, 63, // h2, h3, h4, h5, h6, h7, h8
            ],
        );
        helper(
            lut,
            TestSquare::A8,
            vec![
                7, 14, 21, 28, 35, 42, 49, // h1, g2, f3, e4, d5, c6, b7, a8
                0, 8, 16, 24, 32, 40, 48, // a1, a2, a3, a4, a5, a6, a7
                57, 58, 59, 60, 61, 62, 63, // b8, c8, d8, e8, f8, g8, h8
            ],
        );
        helper(
            lut,
            TestSquare::H8,
            vec![
                0, 9, 18, 27, 36, 45, 54, // a1, b2, c3, d4, e5, f6, g7
                7, 15, 23, 31, 39, 47, 55, // h1, h2, h3, h4, h5, h6, h7
                56, 57, 58, 59, 60, 61, 62, // a8, b8, c8, d8, e8, f8, g8
            ],
        );
        helper(
            lut,
            TestSquare::C2,
            vec![
                2, 18, 26, 34, 42, 50, 58, // c1, c3, c4, c5, c6, c7, c8
                8, 9, 11, 12, 13, 14, 15, // a2, b2, d2, e2, f2, g2, h2
                1, 19, 28, 37, 46, 55, // b1, d3, e4, f5, g6, h7
                3, 17, 24, // d1, b3 a4
            ],
        );
        helper(
            lut,
            TestSquare::F7,
            vec![
                5, 13, 21, 29, 37, 45, 61, // f1, f2, f3, f4, f5, f6, f8
                48, 49, 50, 51, 52, 54, 55, // a7, b7, d7, e7, f7, g7, h7
                39, 46, 60, // h5,g6,e8
                8, 17, 26, 35, 44, 62, // a2, b3, c4, d5, e6, g8
            ],
        );
        helper(
            lut,
            TestSquare::D4,
            vec![
                0, 9, 18, 36, 45, 54, 63, // a1, b2, c3, e5, f6, g7, h8
                6, 13, 20, 34, 41, 48, // g1, f2, e3, c5, b6, a7
                3, 11, 19, 35, 43, 51, 59, // d1, d2, d3, d5, d6, d7, d8
                24, 25, 26, 28, 29, 30, 31, // a4, b4, c4, e4, f4, g4, h4
            ],
        );
        helper(
            lut,
            TestSquare::E4,
            vec![
                7, 14, 21, 35, 42, 49, 56, // h1, g2, f3, d5, c6, b7, a8
                1, 10, 19, 37, 46, 55, // b1, c2, d3, f5, g6, h7
                4, 12, 20, 36, 44, 52, 60, // e1, e2, e3, e5, e6, e7, e8
                24, 25, 26, 27, 29, 30, 31, // a4, b4, c4, d4, f4, g4, h4
            ],
        );
    }

    #[test]
    fn test_knight_lut() {
        let lut = generate_knight_lut();
        helper(lut, TestSquare::A1, vec![10, 17]); // c2, b3
        helper(lut, TestSquare::H1, vec![13, 22]); // f2, g3
        helper(lut, TestSquare::A8, vec![41, 50]); // b6, c7
        helper(lut, TestSquare::H8, vec![46, 53]); // g6, f7
        helper(lut, TestSquare::C2, vec![0, 16, 25, 27, 4, 20]); // a1, a3, b4, d4, e1, e3
        helper(lut, TestSquare::F7, vec![43, 59, 36, 38, 47, 63]); // d6, d8, e5, g5, h6, h8
        helper(lut, TestSquare::D4, vec![10, 12, 17, 21, 33, 37, 42, 44]); // c2, e2, b3, f3, b5, f5, c6, e6
        helper(lut, TestSquare::E4, vec![11, 13, 18, 22, 34, 38, 43, 45]); // d2, f2, c3, g3, c5, g5, d6, f6
    }

    #[test]
    fn test_king_lut() {
        let lut = generate_king_lut();
        helper(lut, TestSquare::A1, vec![1, 8, 9]); // b1, a2, b2
        helper(lut, TestSquare::H1, vec![6, 14, 15]); // g1, g2, h2
        helper(lut, TestSquare::A8, vec![48, 49, 57]); // a7, b7, b8
        helper(lut, TestSquare::H8, vec![54, 55, 62]); // g7, h7, g8
        helper(lut, TestSquare::C2, vec![1, 2, 3, 9, 11, 17, 18, 19]); // b1, c1, d1, b2, d2, b3, c3, d3
        helper(lut, TestSquare::F7, vec![44, 45, 46, 52, 54, 60, 61, 62]); // e6, g6, g6, e7, g7, e8, f8, g8
        helper(lut, TestSquare::D4, vec![18, 19, 20, 26, 28, 34, 35, 36]); // c3, d3, e3, c4, e4, c5, d5, e5
        helper(lut, TestSquare::E4, vec![19, 20, 21, 27, 29, 35, 36, 37]); // d3, e3, f3, d4, f4, d5, e5, f5 
    }

    #[test]
    fn test_pawn_white_lut() {
        let lut = generate_pawn_white_lut();
        helper(lut, TestSquare::A1, vec![]); // no legal moves
        helper(lut, TestSquare::H1, vec![]); // no legal moves
        helper(lut, TestSquare::A8, vec![]); // no legal moves
        helper(lut, TestSquare::H8, vec![]); // no legal moves
        helper(lut, TestSquare::C2, vec![18, 26]); // c3, c4
        helper(lut, TestSquare::F7, vec![61]); // f8
        helper(lut, TestSquare::D4, vec![35]); // d5 
        helper(lut, TestSquare::E4, vec![36]); // e5
    }

    #[test]
    fn test_pawn_black_lut() {
        let lut = generate_pawn_black_lut();
        helper(lut, TestSquare::A1, vec![]); // no legal moves
        helper(lut, TestSquare::H1, vec![]); // no legal moves
        helper(lut, TestSquare::A8, vec![]); // no legal moves
        helper(lut, TestSquare::H8, vec![]); // no legal moves
        helper(lut, TestSquare::C2, vec![2]); // c1
        helper(lut, TestSquare::F7, vec![37, 45]); // f5, f6
        helper(lut, TestSquare::D4, vec![19]); // d3 
        helper(lut, TestSquare::E4, vec![20]); // e3   
    }

    fn helper(lut: BitBoardLUT, square: TestSquare, squares: Vec<u64>) {
        let idx = square as usize;
        let actual = lut[idx];
        let expected = squares.into_iter().fold(0, |acc, e| acc | 1 << e);
        assert_eq!(actual, BitBoard::new(expected), "{:?}", square);
    }
}
