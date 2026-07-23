//! The lowest-level primitive of board geometry is the [`Square`], which encodes information
//! to aid in localizing it on the board.

use crate::make_str_lut;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square {
    pub(crate) idx: usize,
    pub(crate) rank: usize,
    pub(crate) file: usize,
    pub(crate) board: u64,
}

impl Square {
    const fn new(i: usize) -> Self {
        Square {
            idx: i,
            rank: i / 8,
            file: i % 8,
            board: 1u64 << i,
        }
    }
}

#[derive(Debug)]
pub struct OutOfBounds;
impl TryFrom<usize> for Square {
    type Error = OutOfBounds;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 63 {
            Err(OutOfBounds)
        } else {
            Ok(Self::new(value))
        }
    }
}

pub static SQUARES: [Square; 64] = {
    let mut squares = [Square {
        idx: 0,
        rank: 0,
        file: 0,
        board: 1,
    }; 64];
    let mut i = 0;
    while i < 64 {
        squares[i] = Square::new(i);
        i += 1;
    }
    squares
};

make_str_lut!(get_square_idx, {
    "a1" => 0,
    "b1" => 1,
    "c1" => 2,
    "d1" => 3,
    "e1" => 4,
    "f1" => 5,
    "g1" => 6,
    "h1" => 7,

    "a2" => 8,
    "b2" => 9,
    "c2" => 10,
    "d2" => 11,
    "e2" => 12,
    "f2" => 13,
    "g2" => 14,
    "h2" => 15,

    "a3" => 16,
    "b3" => 17,
    "c3" => 18,
    "d3" => 19,
    "e3" => 20,
    "f3" => 21,
    "g3" => 22,
    "h3" => 23,

    "a4" => 24,
    "b4" => 25,
    "c4" => 26,
    "d4" => 27,
    "e4" => 28,
    "f4" => 29,
    "g4" => 30,
    "h4" => 31,

    "a5" => 32,
    "b5" => 33,
    "c5" => 34,
    "d5" => 35,
    "e5" => 36,
    "f5" => 37,
    "g5" => 38,
    "h5" => 39,

    "a6" => 40,
    "b6" => 41,
    "c6" => 42,
    "d6" => 43,
    "e6" => 44,
    "f6" => 45,
    "g6" => 46,
    "h6" => 47,

    "a7" => 48,
    "b7" => 49,
    "c7" => 50,
    "d7" => 51,
    "e7" => 52,
    "f7" => 53,
    "g7" => 54,
    "h7" => 55,

    "a8" => 56,
    "b8" => 57,
    "c8" => 58,
    "d8" => 59,
    "e8" => 60,
    "f8" => 61,
    "g8" => 62,
    "h8" => 63,
});

/// Match a square to its name.
/// Squares are numbered as per the indices in the [`crate::movgen::bitboard`] module-level docs.
///
/// ```
/// # use checkers::square::{get_square_from_name, SQUARES};
/// assert_eq!(get_square_from_name("a1"), Some(SQUARES[0]));
/// assert_eq!(get_square_from_name("e4"), Some(SQUARES[28]));
/// // Square names are case-sensitive.
/// assert_eq!(get_square_from_name("A1"), None);
/// assert_eq!(get_square_from_name("checkers is the superior intellectual game"), None);
/// ```
pub const fn get_square_from_name(name: &str) -> Option<Square> {
    if let Some(idx) = get_square_idx(name) {
        Some(SQUARES[idx])
    } else {
        None
    }
}
