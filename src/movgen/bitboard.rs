//! A bitboard is a `u64` representation of a chessboard, where every bit represents one square on the board.
//! All descriptions of orientation are written from the white perspective, which is best practice in chess analysis.
//! By convention, the first (least-significant) bit signifies the bottom-left (a1) square.
//! Subsequent squares are counted rowwise in reading order, like in the diagram below.
//!
//! ```text
//!   ┌───┬───┬───┬───┬───┬───┬───┬───┐
//! 8 │00 │01 │02 │03 │04 │05 │06 │07 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 7 │08 │09 │10 │11 │12 │13 │14 │15 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 6 │16 │17 │18 │19 │20 │21 │22 │23 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 5 │24 │25 │26 │27 │28 │29 │30 │31 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 4 │32 │33 │34 │35 │36 │37 │38 │39 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 3 │40 │41 │42 │43 │44 │45 │46 │47 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 2 │48 │49 │50 │51 │52 │53 │54 │55 │
//!   ├───┼───┼───┼───┼───┼───┼───┼───┤
//! 1 │56 │57 │58 │59 │60 │61 │62 │63 │
//!   └───┴───┴───┴───┴───┴───┴───┴───┘
//!     a   b   c   d   e   f   g   h
//! ```
//!
//! This diagram has the index of the corresponding square translated into hexadecimal through `1 << idx`.
//!
//! ```text
//!   ┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
//! 8 │0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│
//!   |0000 0001│0000 0002│0000 0004│0000 0008│0000 0010│0000 0020│0000 0040│0000 0080|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 7 │0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│
//!   |0000 0100│0000 0200│0000 0400│0000 0800│0000 1000│0000 2000│0000 4000│0000 8000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 6 │0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│
//!   |0001 0000│0002 0000│0004 0000│0008 0000│0010 0000│0020 0000│0040 0000│0080 0000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 5 │0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│
//!   |0100 0000│0200 0000│0400 0000│0800 0000│1000 0000│2000 0000│4000 0000│8000 0000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 4 │0000 0001│0000 0002│0000 0004│0000 0008│0000 0010│0000 0020│0000 0040│0000 0080│
//!   |0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 3 │0000 0100│0000 0200│0000 0400│0000 0800│0000 1000│0000 2000│0000 4000│0000 8000│
//!   |0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 2 │0001 0000│0002 0000│0004 0000│0008 0000│0010 0000│0020 0000│0040 0000│0080 0000│
//!   |0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000|
//!   ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
//! 1 │0100 0000│0200 0000│0400 0000│0800 0000│1000 0000│2000 0000│4000 0000│8000 0000│
//!   |0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000│0000 0000|   
//!   └─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘
//!        a         b         c         d         e         f         g         h
//! ```
//!
//! The magic numbers for the enums in this module are written in hexadecimal.
//! Hence, every two digits represent one rank on the chessboard.
//! The files in each rank are numbered as follows:
//!
//! File | Hex
//! -----|-----
//!  A   | 01
//!  B   | 02
//!  C   | 04
//!  D   | 08
//!  E   | 10
//!  F   | 20
//!  G   | 40
//!  H   | 80
//!
//! Some examples make this a lot clearer:
//!
//! Example            | Hex                   | Explanation
//! -------------------|-----------------------|-------------------------------------------------------------------------
//! A file             | `0x01010101_01010101` | The first square in each row
//! 3rd rank           | `0x0000FF00_00000000` | 8 consecutive bits between 40-47
//! A8-H1 diagonal     | `0x01020408_10204080` | The first square in row 8 (`01`), the second square in row 7 (`02`) etc.
//! H5-D1 antidiagonal | `0x00000080_40201008` | The 8th square in row 5 (`80`), the 7th square in row 4 (`40`) etc.
//!
//! The bits of a bitboard can be efficiently iterated through a [`BitBoardIterator`].
//!
//! ```rust
//! # use checkers::movgen::bitboard::{File, BitBoard};
//! # use std::iter;
//! // 8 x . . . . . . .
//! // 7 x . . . . . . .
//! // 6 x . . . . . . .
//! // 5 x . . . . . . .
//! // 4 x . . . . . . .
//! // 3 x . . . . . . .
//! // 2 x . . . . . . .
//! // 1 x . . . . . . .
//! //   a b c d e f g h
//! let board = BitBoard::from(File::A as u64);
//! let actual: Vec<bool> = board.iter_bits().collect();
//! let expected: Vec<bool> = iter::once(true).chain(iter::repeat_n(false, 7)).cycle().take(64).collect();
//! assert_eq!(actual, expected);
//! ```

use std::{
    fmt::{Debug, Display},
    ops::{self, BitAnd, Deref, Not, Shl},
};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    impl_enum_index_math,
    square::{OutOfBounds, get_square_from_name},
};

/// Find the iterator element that intersects with some square.
/// From this, the rank, file, diagonal and antidiagonal of a square are found.
/// This helps generating the bitboards for each piece and starting square.
pub trait ContainingSquare<I>: Iterator<Item = I> + Sized
where
    I: Copy + Into<u64>,
{
    fn find_containing_square(&mut self, square: u64) -> Option<u64> {
        self.find(|&el| el.into() & square != 0).map(|el| el.into())
    }
}

/// Files are vertical columns on the chessboard.
/// These can be represented as ones in every eight bit.
#[repr(u64)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd)]
pub enum File {
    A = 0x01010101_01010101,
    B = 0x02020202_02020202,
    C = 0x04040404_04040404,
    D = 0x08080808_08080808,
    E = 0x10101010_10101010,
    F = 0x20202020_20202020,
    G = 0x40404040_40404040,
    H = 0x80808080_80808080,
}

impl From<File> for u64 {
    fn from(val: File) -> Self {
        val as u64
    }
}

impl_enum_index_math!(File);

impl ContainingSquare<File> for FileIter {}

/// Ranks are horizontal rows on the chessboard.
/// These can be represented as consecutive groups of eight ones.
#[repr(u64)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, PartialOrd, Eq)]
pub enum Rank {
    Eight = 0x000000_00000000FF,
    Seven = 0x000000_000000FF00,
    Six = 0x000000_0000FF0000,
    Five = 0x000000_00FF000000,
    Four = 0x000000_FF00000000,
    Three = 0x0000FF_0000000000,
    Two = 0x00FF00_0000000000,
    One = 0xFF0000_0000000000,
}

impl From<Rank> for u64 {
    fn from(value: Rank) -> Self {
        value as u64
    }
}

impl_enum_index_math!(Rank);

impl ContainingSquare<Rank> for RankIter {}

/// Diagonals are NW-to-SE lines, similar to matrix terminology.
/// These can be represented by increasing and decreasing distances from the left edge.
#[repr(u64)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, PartialOrd)]
pub enum Diagonal {
    /// h8 - h8
    PlusSeven = 0x00000000_00000080,
    /// h7 - g8
    PlusSix = 0x00000000_00008040,
    /// h6 - f8
    PlusFive = 0x00000000_00804020,
    /// h5 - e8
    PlusFour = 0x00000000_80402010,
    /// h4 - d8
    PlusThree = 0x00000080_40201008,
    /// h3 - c8
    PlusTwo = 0x00008040_20100804,
    /// h2 - b8
    PlusOne = 0x00804020_10080402,
    /// h1 - a8
    Main = 0x80402010_08040201,
    /// g1 - a7
    MinusOne = 0x40201008_04020100,
    /// f1 - a6
    MinusTwo = 0x20100804_02010000,
    /// e1 - a5
    MinusThree = 0x10080402_01000000,
    /// d1 - a4
    MinusFour = 0x08040201_00000000,
    /// c1 - a3
    MinusFive = 0x04020100_00000000,
    /// b1 - a2
    MinusSix = 0x02010000_00000000,
    /// a1 - a1
    MinusSeven = 0x01000000_00000000,
}

impl From<Diagonal> for u64 {
    fn from(value: Diagonal) -> Self {
        value as u64
    }
}

impl_enum_index_math!(Diagonal);

impl ContainingSquare<Diagonal> for DiagonalIter {}

/// Antidiagonals are SW-to-NE lines, similar to matrix terminology.
/// These can be represented by increasing and decreasing distances from the left edge.
#[repr(u64)]
#[derive(Clone, Copy, Debug, EnumIter, PartialEq, PartialOrd)]
pub enum AntiDiagonal {
    /// a8 - a8
    PlusSeven = 0x00000000_00000001,
    /// a7 - b8
    PlusSix = 0x00000000_00000102,
    /// a6 - c8
    PlusFive = 0x00000000_00010204,
    /// a5 - d8
    PlusFour = 0x00000000_01020408,
    /// a4 - e8
    PlusThree = 0x00000001_02040810,
    /// a3 - f8
    PlusTwo = 0x00000102_04081020,
    /// a2 - g8
    PlusOne = 0x00010204_08102040,
    /// a1 - h8
    Main = 0x01020408_10204080,
    /// b1 - h7
    MinusOne = 0x02040810_20408000,
    /// c1 - h6
    MinusTwo = 0x04081020_40800000,
    /// d1 - h5
    MinusThree = 0x08102040_80000000,
    /// e1 - h4
    MinusFour = 0x10204080_00000000,
    /// f1 - h3
    MinusFive = 0x20408000_00000000,
    /// g1 - h2
    MinusSix = 0x40800000_00000000,
    /// h1 - h1
    MinusSeven = 0x80000000_00000000,
}

impl From<AntiDiagonal> for u64 {
    fn from(value: AntiDiagonal) -> Self {
        value as u64
    }
}

impl_enum_index_math!(AntiDiagonal);

impl ContainingSquare<AntiDiagonal> for AntiDiagonalIter {}

/// An unsigned 64 bit integer representation of a chessboard, where every bit represents one square.
/// Fore more information, see the [`crate::movgen::bitboard`] module-level docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const fn new(board: u64) -> Self {
        Self(board)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn iter_bits(&self) -> impl Iterator<Item = bool> {
        BitBoardIterator::new(**self)
    }

    pub fn iter_bits_masked(&self, mask: u64) -> impl Iterator<Item = bool> {
        MaskedBitBoardIterator::new(**self, mask)
    }

    pub fn iter_ones(&self) -> impl Iterator<Item = usize> {
        BitBoardOnesIterator::new(*self)
    }
}

impl ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl ops::BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Deref for BitBoard {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        BitBoard(value)
    }
}

impl Not for BitBoard {
    type Output = Self;
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

const BITBOARD_TRUE_CHAR: char = 'x';
const BITBOARD_FALSE_CHAR: char = '.';

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mask: u64 = 0b1111_1111;
        let ranks = (0..64)
            .step_by(8)
            .map(|rank| self.iter_bits_masked(mask.shl(rank)))
            .map(|rank| {
                rank.map(|square| match square {
                    true => BITBOARD_TRUE_CHAR,
                    false => BITBOARD_FALSE_CHAR,
                })
                .collect::<String>()
            });
        for rank in ranks {
            writeln!(f, "{}", rank)?
        }
        Ok(())
    }
}

impl std::fmt::Binary for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:064b}", self.0)
    }
}

struct MaskedBitBoardIterator {
    board: u64,
    left: u64,
    right: u64,
}

impl MaskedBitBoardIterator {
    pub fn new(board: u64, mask: u64) -> Self {
        Self {
            board,
            left: mask.lowest_one().unwrap_or_default().into(),
            right: mask.highest_one().unwrap_or_default().into(),
        }
    }
}

impl Iterator for MaskedBitBoardIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.left <= self.right {
            let result = self.board & 1u64.shl(self.left) != 0;
            self.left += 1;
            Some(result)
        } else {
            None
        }
    }
}
struct BitBoardIterator(MaskedBitBoardIterator);

impl BitBoardIterator {
    pub fn new(board: u64) -> Self {
        Self(MaskedBitBoardIterator::new(board, u64::MAX))
    }
}

impl Iterator for BitBoardIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

struct BitBoardOnesIterator(u64);

impl BitBoardOnesIterator {
    pub fn new(board: BitBoard) -> Self {
        Self(board.0)
    }
}

impl Iterator for BitBoardOnesIterator {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.0.lowest_one() {
            // Mask the last one.
            // 0b0010 0100 ^ (1 << 2) = 0b0010 0100 ^ 0b0000 00100 = 0b0010 0000
            self.0 ^= 1 << idx;
            Some(idx as usize)
        } else {
            None
        }
    }
}

/// Transform a list of square ("e2", "c5", "f3") into a [BitBoard]. The names are assumed to be correct.
pub fn bitboard_from_squares<'a>(names: impl IntoIterator<Item = &'a str>) -> BitBoard {
    BitBoard::from(names.into_iter().fold(0, |mut acc, name| {
        acc |= get_square_from_name(name).unwrap().board;
        acc
    }))
}

/// The ordering of enum names carries semantic meaning, as these are indexed in the [`impl_enum_math` macro][crate::macros].
/// These tests avoid a regression by reordering the enum.
#[cfg(test)]
mod test_enum_ordering {
    use strum::IntoEnumIterator;

    use crate::movgen::bitboard::{AntiDiagonal, Diagonal, File, Rank};

    #[test]
    fn test_rank_ordered_from_top_to_bottom_row() {
        assert!(is_increasing(Rank::iter()));
    }

    #[test]
    fn test_file_ordered_from_left_to_right_column() {
        assert!(is_increasing(File::iter()));
    }

    #[test]
    fn test_diagonal_ordered_from_top_to_bottom() {
        assert!(is_increasing(Diagonal::iter().take(8)));
        assert!(is_decreasing(Diagonal::iter().skip(7)));
    }

    #[test]
    fn test_antidiagonal_ordered_from_top_to_bottom() {
        assert!(is_increasing(AntiDiagonal::iter()));
    }

    fn is_increasing<T: PartialOrd>(iter: impl Iterator<Item = T>) -> bool {
        let values: Vec<T> = iter.collect::<Vec<T>>();
        values.windows(2).all(|window| window[0] < window[1])
    }

    fn is_decreasing<T: PartialOrd>(iter: impl Iterator<Item = T>) -> bool {
        let values: Vec<T> = iter.collect::<Vec<T>>();
        values.windows(2).all(|window| window[0] > window[1])
    }
}

#[cfg(test)]
mod test_to_string {
    use crate::movgen::bitboard::{AntiDiagonal, BitBoard, Diagonal, File, Rank};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_rank() {
        for (rank, string) in [
            (
                Rank::One,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 xxxxxxxx\n",
            ),
            (
                Rank::Two,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 xxxxxxxx\n\
                 ........\n",
            ),
            (
                Rank::Three,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 xxxxxxxx\n\
                 ........\n\
                 ........\n",
            ),
            (
                Rank::Four,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 xxxxxxxx\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Rank::Five,
                "........\n\
                 ........\n\
                 ........\n\
                 xxxxxxxx\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Rank::Six,
                "........\n\
                 ........\n\
                 xxxxxxxx\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Rank::Seven,
                "........\n\
                 xxxxxxxx\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Rank::Eight,
                "xxxxxxxx\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
        ] {
            assert_eq!(BitBoard(rank as u64).to_string(), string, "{rank:?}",);
        }
    }

    #[test]
    fn test_file() {
        for (file, string) in [
            (
                File::A,
                "x.......\n\
                 x.......\n\
                 x.......\n\
                 x.......\n\
                 x.......\n\
                 x.......\n\
                 x.......\n\
                 x.......\n",
            ),
            (
                File::B,
                ".x......\n\
                 .x......\n\
                 .x......\n\
                 .x......\n\
                 .x......\n\
                 .x......\n\
                 .x......\n\
                 .x......\n",
            ),
            (
                File::C,
                "..x.....\n\
                 ..x.....\n\
                 ..x.....\n\
                 ..x.....\n\
                 ..x.....\n\
                 ..x.....\n\
                 ..x.....\n\
                 ..x.....\n",
            ),
            (
                File::D,
                "...x....\n\
                 ...x....\n\
                 ...x....\n\
                 ...x....\n\
                 ...x....\n\
                 ...x....\n\
                 ...x....\n\
                 ...x....\n",
            ),
            (
                File::E,
                "....x...\n\
                 ....x...\n\
                 ....x...\n\
                 ....x...\n\
                 ....x...\n\
                 ....x...\n\
                 ....x...\n\
                 ....x...\n",
            ),
            (
                File::F,
                ".....x..\n\
                 .....x..\n\
                 .....x..\n\
                 .....x..\n\
                 .....x..\n\
                 .....x..\n\
                 .....x..\n\
                 .....x..\n",
            ),
            (
                File::G,
                "......x.\n\
                 ......x.\n\
                 ......x.\n\
                 ......x.\n\
                 ......x.\n\
                 ......x.\n\
                 ......x.\n\
                 ......x.\n",
            ),
            (
                File::H,
                ".......x\n\
                 .......x\n\
                 .......x\n\
                 .......x\n\
                 .......x\n\
                 .......x\n\
                 .......x\n\
                 .......x\n",
            ),
        ] {
            assert_eq!(BitBoard(file as u64).to_string(), string, "{file:?}",);
        }
    }

    #[test]
    fn test_diagonal() {
        for (diagonal, string) in [
            (
                Diagonal::MinusSeven,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 x.......\n",
            ),
            (
                Diagonal::MinusSix,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 x.......\n\
                 .x......\n",
            ),
            (
                Diagonal::MinusFive,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 x.......\n\
                 .x......\n\
                 ..x.....\n",
            ),
            (
                Diagonal::MinusFour,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 x.......\n\
                 .x......\n\
                 ..x.....\n\
                 ...x....\n",
            ),
            (
                Diagonal::MinusThree,
                "........\n\
                 ........\n\
                 ........\n\
                 x.......\n\
                 .x......\n\
                 ..x.....\n\
                 ...x....\n\
                 ....x...\n",
            ),
            (
                Diagonal::MinusTwo,
                "........\n\
                 ........\n\
                 x.......\n\
                 .x......\n\
                 ..x.....\n\
                 ...x....\n\
                 ....x...\n\
                 .....x..\n",
            ),
            (
                Diagonal::MinusOne,
                "........\n\
                 x.......\n\
                 .x......\n\
                 ..x.....\n\
                 ...x....\n\
                 ....x...\n\
                 .....x..\n\
                 ......x.\n",
            ),
            (
                Diagonal::Main,
                "x.......\n\
                 .x......\n\
                 ..x.....\n\
                 ...x....\n\
                 ....x...\n\
                 .....x..\n\
                 ......x.\n\
                 .......x\n",
            ),
            (
                Diagonal::PlusOne,
                ".x......\n\
                 ..x.....\n\
                 ...x....\n\
                 ....x...\n\
                 .....x..\n\
                 ......x.\n\
                 .......x\n\
                 ........\n",
            ),
            (
                Diagonal::PlusTwo,
                "..x.....\n\
                 ...x....\n\
                 ....x...\n\
                 .....x..\n\
                 ......x.\n\
                 .......x\n\
                 ........\n\
                 ........\n",
            ),
            (
                Diagonal::PlusThree,
                "...x....\n\
                 ....x...\n\
                 .....x..\n\
                 ......x.\n\
                 .......x\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Diagonal::PlusFour,
                "....x...\n\
                 .....x..\n\
                 ......x.\n\
                 .......x\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Diagonal::PlusFive,
                ".....x..\n\
                 ......x.\n\
                 .......x\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Diagonal::PlusSix,
                "......x.\n\
                 .......x\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                Diagonal::PlusSeven,
                ".......x\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
        ] {
            assert_eq!(
                BitBoard(diagonal as u64).to_string(),
                string,
                "{diagonal:?}",
            );
        }
    }

    #[test]
    fn test_antidiagonal() {
        for (antidiagonal, string) in [
            (
                AntiDiagonal::MinusSeven,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 .......x\n",
            ),
            (
                AntiDiagonal::MinusSix,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 .......x\n\
                 ......x.\n",
            ),
            (
                AntiDiagonal::MinusFive,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 .......x\n\
                 ......x.\n\
                 .....x..\n",
            ),
            (
                AntiDiagonal::MinusFour,
                "........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 .......x\n\
                 ......x.\n\
                 .....x..\n\
                 ....x...\n",
            ),
            (
                AntiDiagonal::MinusThree,
                "........\n\
                 ........\n\
                 ........\n\
                 .......x\n\
                 ......x.\n\
                 .....x..\n\
                 ....x...\n\
                 ...x....\n",
            ),
            (
                AntiDiagonal::MinusTwo,
                "........\n\
                 ........\n\
                 .......x\n\
                 ......x.\n\
                 .....x..\n\
                 ....x...\n\
                 ...x....\n\
                 ..x.....\n",
            ),
            (
                AntiDiagonal::MinusOne,
                "........\n\
                 .......x\n\
                 ......x.\n\
                 .....x..\n\
                 ....x...\n\
                 ...x....\n\
                 ..x.....\n\
                 .x......\n",
            ),
            (
                AntiDiagonal::Main,
                ".......x\n\
                 ......x.\n\
                 .....x..\n\
                 ....x...\n\
                 ...x....\n\
                 ..x.....\n\
                 .x......\n\
                 x.......\n",
            ),
            (
                AntiDiagonal::PlusOne,
                "......x.\n\
                 .....x..\n\
                 ....x...\n\
                 ...x....\n\
                 ..x.....\n\
                 .x......\n\
                 x.......\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusTwo,
                ".....x..\n\
                 ....x...\n\
                 ...x....\n\
                 ..x.....\n\
                 .x......\n\
                 x.......\n\
                 ........\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusThree,
                "....x...\n\
                 ...x....\n\
                 ..x.....\n\
                 .x......\n\
                 x.......\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusFour,
                "...x....\n\
                 ..x.....\n\
                 .x......\n\
                 x.......\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusFive,
                "..x.....\n\
                 .x......\n\
                 x.......\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusSix,
                ".x......\n\
                 x.......\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
            (
                AntiDiagonal::PlusSeven,
                "x.......\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n\
                 ........\n",
            ),
        ] {
            assert_eq!(
                BitBoard(antidiagonal as u64).to_string(),
                string,
                "{antidiagonal:?}",
            );
        }
    }
}

#[cfg(test)]
mod tests_iter {
    use crate::movgen::bitboard::{
        AntiDiagonal, BitBoard, BitBoardIterator, ContainingSquare, Diagonal, File,
        MaskedBitBoardIterator, Rank,
    };
    use pretty_assertions::assert_eq;
    use std::collections::VecDeque;
    use std::{fmt::Debug, iter, ops::Shl};
    use strum::IntoEnumIterator;

    /// Place a single 1 in every position on the board.
    #[test]
    fn test_bitboard_iter() {
        for i in 0..64 {
            let actual = BitBoardIterator::new(1 << i);
            let expected = iter::repeat_n(false, i)
                .chain(iter::once(true))
                .chain(iter::repeat_n(false, 64 - i - 1));
            test_iter_helper(actual, expected);
        }
    }

    /// Mask a group of 8 1s in a board that is all 1s.
    #[test]
    fn test_masked_bitboard_iter() {
        for i in 0..(64 - 8) {
            let actual = MaskedBitBoardIterator::new(u64::MAX, 255u64.shl(i));
            let expected = iter::repeat_n(true, 8);
            test_iter_helper(actual, expected);
        }
    }

    fn test_iter_helper<T: Eq + Debug>(it1: impl Iterator<Item = T>, it2: impl Iterator<Item = T>) {
        assert_eq!(it1.collect::<Vec<T>>(), it2.collect::<Vec<T>>())
    }

    /// Test if each square of the board is found in the correct rank.
    #[test]
    fn test_rank_containing_square() {
        for (row, expected) in (0..8).zip(Rank::iter()) {
            for col in 0..8 {
                let square = 1u64.shl(row * 8 + col);
                assert_eq!(
                    Rank::iter().find_containing_square(square),
                    Some(expected.into())
                );
            }
        }
    }

    /// Test if each square of the board is found in the correct file.
    #[test]
    fn test_file_containing_square() {
        for (idx, expected) in (0..64).zip(File::iter().cycle()) {
            let square = 1u64.shl(idx);
            assert_eq!(
                File::iter().find_containing_square(square),
                Some(expected.into())
            );
        }
    }

    /// Test if each square of the board is found in the correct diagonal.
    #[test]
    fn test_diagonal_containing_square() {
        // When looking at the indices of a 3x3 grid
        // 0 1 2
        // 3 4 5
        // 6 7 8
        // The diagonals can be iterated as follows:
        // Square idx   |  0  1  2  3  4  5  6  7  8
        // Diagonal idx |  0 +1 +2 -1  0 +1 -2 -1  0
        // When going down in ranks from the 8th to the 1st, we need to add
        // 1 negative antidiagonal in front of the main and fill
        // up the rest of the rank with positive antidiagonals in order.
        let positive = [
            Diagonal::PlusOne,
            Diagonal::PlusTwo,
            Diagonal::PlusThree,
            Diagonal::PlusFour,
            Diagonal::PlusFive,
            Diagonal::PlusSix,
            Diagonal::PlusSeven,
        ];
        let negative = [
            Diagonal::MinusOne,
            Diagonal::MinusTwo,
            Diagonal::MinusThree,
            Diagonal::MinusFour,
            Diagonal::MinusFive,
            Diagonal::MinusSix,
            Diagonal::MinusSeven,
        ];
        let mut diagonals = VecDeque::from(positive);
        diagonals.push_front(Diagonal::Main);
        for row in 0..8 {
            for (col, &expected) in (0..8).zip(diagonals.iter()) {
                let idx = row * 8 + col;
                let square = 1u64.shl(idx);
                assert_eq!(
                    Diagonal::iter().find_containing_square(square),
                    Some(expected.into()),
                    "{} {:?} {:?}",
                    idx,
                    square,
                    expected
                );
            }
            // The or will only get triggered at the last iteration.
            diagonals.push_front(*negative.get(row).unwrap_or(&Diagonal::Main));
            diagonals.pop_back();
        }
    }

    /// Test if each square of the board is found in the correct antidiagonal.
    #[test]
    fn test_antidiagonal_containing_square() {
        // When looking at the indices of a 3x3 grid
        // 0 1 2
        // 3 4 5
        // 6 7 8
        // The antidiagonals can be iterated as follows:
        // Square idx   |  0  1  2  3  4  5  6  7  8
        // Diagonal idx | +2 +1  0 +1  0 -1  0 -1 -2
        // When going down in ranks from the 8th to the 1st, we need to remove
        // 1 positive antidiagonal in front of the main and fill
        // up the rest of the rank with negative antidiagonals in order.
        let positive = [
            AntiDiagonal::PlusSeven,
            AntiDiagonal::PlusSix,
            AntiDiagonal::PlusFive,
            AntiDiagonal::PlusFour,
            AntiDiagonal::PlusThree,
            AntiDiagonal::PlusTwo,
            AntiDiagonal::PlusOne,
        ];
        let negative = [
            AntiDiagonal::MinusOne,
            AntiDiagonal::MinusTwo,
            AntiDiagonal::MinusThree,
            AntiDiagonal::MinusFour,
            AntiDiagonal::MinusFive,
            AntiDiagonal::MinusSix,
            AntiDiagonal::MinusSeven,
        ];
        let mut antidiagonals = VecDeque::from(positive);
        antidiagonals.push_back(AntiDiagonal::Main);
        for row in 0..8 {
            for (col, &expected) in (0..8).zip(antidiagonals.iter()) {
                let idx = row * 8 + col;
                let square = 1u64.shl(idx);
                assert_eq!(
                    AntiDiagonal::iter().find_containing_square(square),
                    Some(expected.into()),
                    "{} {:?} {:?}",
                    idx,
                    square,
                    expected
                );
            }
            // The or will only get triggered at the last iteration.
            antidiagonals.push_back(*negative.get(row).unwrap_or(&AntiDiagonal::Main));
            antidiagonals.pop_front();
        }
    }

    #[test]
    fn test_bitboard_ones_iter() {
        test_bitboard_ones_iter_helper(BitBoard::new(0b0000_0000), vec![]);
        test_bitboard_ones_iter_helper(BitBoard::new(0b1111_1111), vec![0, 1, 2, 3, 4, 5, 6, 7]);
        test_bitboard_ones_iter_helper(BitBoard::new(0b0000_0001), vec![0]);
        test_bitboard_ones_iter_helper(BitBoard::new(0b1000_0000), vec![7]);
        test_bitboard_ones_iter_helper(BitBoard::new(0b1000_0001), vec![0, 7]);
        // Bender's appartment number in Futurama.
        test_bitboard_ones_iter_helper(BitBoard::new(0b0010_0100), vec![2, 5]);
    }

    fn test_bitboard_ones_iter_helper(board: BitBoard, ones: Vec<usize>) {
        test_iter_helper(board.iter_ones(), ones.into_iter());
    }
}

#[cfg(test)]
mod tests_enum_math {
    use crate::movgen::bitboard::{Diagonal, File, Rank};

    #[test]
    fn test_rank_add() {
        assert_eq!(Rank::One.saturating_add(0), Rank::One);
        assert_eq!(Rank::One.saturating_add(1), Rank::One);
        assert_eq!(Rank::One.saturating_add(2), Rank::One);
        assert_eq!(Rank::One.saturating_add(6), Rank::One);
        assert_eq!(Rank::One.saturating_add(7), Rank::One);
        assert_eq!(Rank::One.saturating_add(8), Rank::One);
        assert_eq!(Rank::One.saturating_add(1_000), Rank::One);
    }

    #[test]
    fn test_rank_sub() {
        assert_eq!(Rank::One.saturating_sub(0), Rank::One);
        assert_eq!(Rank::One.saturating_sub(1), Rank::Two);
        assert_eq!(Rank::One.saturating_sub(2), Rank::Three);
        assert_eq!(Rank::One.saturating_sub(6), Rank::Seven);
        assert_eq!(Rank::One.saturating_sub(7), Rank::Eight);
        assert_eq!(Rank::One.saturating_sub(8), Rank::Eight);
        assert_eq!(Rank::One.saturating_sub(1_000), Rank::Eight);
    }

    #[test]
    fn test_file_add() {
        assert_eq!(File::A.saturating_add(0), File::A);
        assert_eq!(File::A.saturating_add(1), File::B);
        assert_eq!(File::A.saturating_add(2), File::C);
        assert_eq!(File::A.saturating_add(6), File::G);
        assert_eq!(File::A.saturating_add(7), File::H);
        assert_eq!(File::A.saturating_add(8), File::H);
        assert_eq!(File::A.saturating_add(1_000), File::H);
    }

    #[test]
    fn test_file_sub() {
        assert_eq!(File::A.saturating_sub(0), File::A);
        assert_eq!(File::A.saturating_sub(1), File::A);
        assert_eq!(File::A.saturating_sub(2), File::A);
        assert_eq!(File::A.saturating_sub(6), File::A);
        assert_eq!(File::A.saturating_sub(7), File::A);
        assert_eq!(File::A.saturating_sub(8), File::A);
        assert_eq!(File::A.saturating_sub(1_000), File::A);
    }

    #[test]
    fn test_diagonal_add() {
        assert_eq!(Diagonal::Main.saturating_add(0), Diagonal::Main);
        assert_eq!(Diagonal::Main.saturating_add(1), Diagonal::MinusOne);
        assert_eq!(Diagonal::Main.saturating_add(2), Diagonal::MinusTwo);
        assert_eq!(Diagonal::Main.saturating_add(6), Diagonal::MinusSix);
        assert_eq!(Diagonal::Main.saturating_add(7), Diagonal::MinusSeven);
        assert_eq!(Diagonal::Main.saturating_add(8), Diagonal::MinusSeven);
        assert_eq!(Diagonal::Main.saturating_add(1_000), Diagonal::MinusSeven);
    }

    #[test]
    fn test_diagonal_sub() {
        assert_eq!(Diagonal::Main.saturating_sub(0), Diagonal::Main);
        assert_eq!(Diagonal::Main.saturating_sub(1), Diagonal::PlusOne);
        assert_eq!(Diagonal::Main.saturating_sub(2), Diagonal::PlusTwo);
        assert_eq!(Diagonal::Main.saturating_sub(6), Diagonal::PlusSix);
        assert_eq!(Diagonal::Main.saturating_sub(7), Diagonal::PlusSeven);
        assert_eq!(Diagonal::Main.saturating_sub(8), Diagonal::PlusSeven);
        assert_eq!(Diagonal::Main.saturating_sub(1_000), Diagonal::PlusSeven);
    }
}

#[cfg(test)]
mod tests_bitboard_from_squares {
    use crate::{
        movgen::bitboard::{BitBoard, bitboard_from_squares},
        square::get_square_from_name,
    };

    #[test]
    fn test_valid_squares() {
        assert_eq!(
            BitBoard::from(get_square_from_name("e4").unwrap().board),
            bitboard_from_squares(["e4"])
        );
        assert_eq!(
            BitBoard::from(
                get_square_from_name("e4").unwrap().board
                    | get_square_from_name("e5").unwrap().board
            ),
            bitboard_from_squares(["e4", "e5"])
        );
    }

    #[test]
    fn test_empty_squares() {
        assert_eq!(BitBoard::empty(), bitboard_from_squares([]));
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_invalid_squares() {
        bitboard_from_squares(["x9"]);
    }
}
