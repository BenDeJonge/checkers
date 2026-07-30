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
    "a1" => 56,
    "b1" => 57,
    "c1" => 58,
    "d1" => 59,
    "e1" => 60,
    "f1" => 61,
    "g1" => 62,
    "h1" => 63,

    "a2" => 48,
    "b2" => 49,
    "c2" => 50,
    "d2" => 51,
    "e2" => 52,
    "f2" => 53,
    "g2" => 54,
    "h2" => 55,

    "a3" => 40,
    "b3" => 41,
    "c3" => 42,
    "d3" => 43,
    "e3" => 44,
    "f3" => 45,
    "g3" => 46,
    "h3" => 47,

    "a4" => 32,
    "b4" => 33,
    "c4" => 34,
    "d4" => 35,
    "e4" => 36,
    "f4" => 37,
    "g4" => 38,
    "h4" => 39,

    "a5" => 24,
    "b5" => 25,
    "c5" => 26,
    "d5" => 27,
    "e5" => 28,
    "f5" => 29,
    "g5" => 30,
    "h5" => 31,

    "a6" => 16,
    "b6" => 17,
    "c6" => 18,
    "d6" => 19,
    "e6" => 20,
    "f6" => 21,
    "g6" => 22,
    "h6" => 23,

    "a7" => 8,
    "b7" => 9,
    "c7" => 10,
    "d7" => 11,
    "e7" => 12,
    "f7" => 13,
    "g7" => 14,
    "h7" => 15,

    "a8" => 0,
    "b8" => 1,
    "c8" => 2,
    "d8" => 3,
    "e8" => 4,
    "f8" => 5,
    "g8" => 6,
    "h8" => 7,
});

/// Match a square to its name.
/// Squares are numbered as per the indices in the [`crate::movgen::bitboard`] module-level docs.
///
/// ```
/// # use checkers::square::{get_square_from_name, SQUARES};
/// assert_eq!(get_square_from_name("a8"), Some(SQUARES[0]));
/// assert_eq!(get_square_from_name("h8"), Some(SQUARES[7]));
/// assert_eq!(get_square_from_name("e4"), Some(SQUARES[36]));
/// assert_eq!(get_square_from_name("a1"), Some(SQUARES[56]));
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
