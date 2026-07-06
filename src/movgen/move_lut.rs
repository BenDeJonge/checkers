use std::ops::Add;

use strum::{EnumIter, IntoEnumIterator};

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
            *rook &= bishop;
        });
    rook_lut
}

const KNIGHT_JUMPS: [u64; 4] = [6, 10, 15, 17];

// TODO: review this. Why can not use into() instead of as u64?
fn generate_knight_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let ranks: u64 = match square.rank {
            0 => Rank::Two as u64 | Rank::Three as u64,
            1 => Rank::One as u64 | Rank::Three as u64 | Rank::Four as u64,
            6 => Rank::Four as u64 | Rank::Five as u64 | Rank::Seven as u64,
            7 => Rank::Six as u64 | Rank::Seven as u64,
            _ => u64::MAX,
        };
        let files: u64 = match square.file {
            0 => File::B as u64 | File::C as u64,
            1 => File::A as u64 | File::C as u64 | File::D as u64,
            6 => File::E as u64 | File::F as u64 | File::H as u64,
            7 => File::F as u64 | File::G as u64,
            _ => u64::MAX,
        };
        let mut knight = 0;
        for jump in KNIGHT_JUMPS {
            knight |= square.board.add(jump) | square.board.saturating_sub(jump);
        }
        knight &= ranks | files;
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
