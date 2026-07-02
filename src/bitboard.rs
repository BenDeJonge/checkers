//! A bitboard is a `u64` representation of a chessboard, where every bit represents one square on the board.
//! By convention, the first (least-significant) bit signifies the bottom-left (a1) square.
//! Subsequent squares are counted rowwise in reading order, like in the diagram below.
//!
//! ```text
//! 8 ⬜⬛⬜⬛⬜⬛⬜⬛    8 56 57 58 59 60 61 62 63
//! 7 ⬛⬜⬛⬜⬛⬜⬛⬜    7 48 49 50 51 52 53 54 55
//! 6 ⬜⬛⬜⬛⬜⬛⬜⬛    6 40 41 42 43 44 45 46 47
//! 5 ⬛⬜⬛⬜⬛⬜⬛⬜    5 32 33 34 35 36 37 38 39
//! 4 ⬜⬛⬜⬛⬜⬛⬜⬛    4 24 25 26 27 28 29 30 31
//! 3 ⬛⬜⬛⬜⬛⬜⬛⬜    3 16 17 18 19 20 21 22 23
//! 2 ⬜⬛⬜⬛⬜⬛⬜⬛    2 08 09 10 11 12 13 14 15
//! 1 ⬛⬜⬛⬜⬛⬜⬛⬜    1 00 01 02 03 04 05 06 07
//!    a b c d e f g h       a  b  c  d  e  f  g  h
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
//! 3rd rank           | `0x00000000_00FF0000` | 8 consecutive bits between 16-24
//! A8-H1 diagonal     | `0x01020408_10204080` | The first square in row 8 (`01`), the second square in row 7 (`02`) etc.
//! H5-D1 antidiagonal | `0x00000080_40201008` | The 8th square in row 5 (`80`), the 7th square in row 4 (`40`) etc.
//!
//! The bits of a bitboard can be efficiently iterated through a [`BitBoardIterator`].
//!
//! ```rust
//! # use checkers::bitboard::{File, BitBoard};
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
    fmt::Display,
    ops::{Deref, Shl},
};

use crate::bitboard_iterator::{BitBoardIterator, MaskedBitBoardIterator};

/// Files are vertical columns on the chessboard.
/// These can be represented as ones in every eight bit.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
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

/// Ranks are horizontal rows on the chessboard.
/// These can be represented as consecutive groups of eight ones.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum Rank {
    One = 0x000000_00000000FF,
    Two = 0x000000_000000FF00,
    Three = 0x000000_0000FF0000,
    Four = 0x000000_00FF000000,
    Five = 0x000000_FF00000000,
    Six = 0x0000FF_0000000000,
    Seven = 0x00FF00_0000000000,
    Eight = 0xFF0000_0000000000,
}

/// Diagonals are NW-to-SE lines, similar to matrix terminology.
/// These can be represented by increasing and decreasing distances from the left edge.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum Diagonal {
    MinusSeven = 0x00000000_00000001, // a1 - a1
    MinusSix = 0x00000000_00000102,   // b1 - a2
    MinusFive = 0x00000000_00010204,  // c1 - a3
    MinusFour = 0x00000000_01020408,  // d1 - a4
    MinusThree = 0x00000001_02040810, // e1 - a5
    MinusTwo = 0x00000102_04081020,   // f1 - a6
    MinusOne = 0x00010204_08102040,   // g1 - a7
    Main = 0x01020408_10204080,       // h1 - a8
    PlusOne = 0x02040810_20408000,    // h2 - b8
    PlusTwo = 0x04081020_40800000,    // h3 - c8
    PlusThree = 0x08102040_80000000,  // h4 - d8
    PlusFour = 0x10204080_00000000,   // h5 - e8
    PlusFive = 0x20408000_00000000,   // h6 - f8
    PlusSix = 0x40800000_00000000,    // h7 - g8
    PlusSeven = 0x80000000_00000000,  // h8 - h8
}

/// Antidiagonals are SW-to-NE lines, similar to matrix terminology.
/// These can be represented by increasing and decreasing distances from the left edge.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum AntiDiagonal {
    MinusSeven = 0x00000000_00000080, // h1 - h1
    MinusSix = 0x00000000_00008040,   // g1 - h2
    MinusFive = 0x00000000_00804020,  // f1 - h3
    MinusFour = 0x00000000_80402010,  // e1 - h4
    MinusThree = 0x00000080_40201008, // d1 - h5
    MinusTwo = 0x00008040_20100804,   // c1 - h6
    MinusOne = 0x00804020_10080402,   // b1 - h7
    Main = 0x80402010_08040201,       // a1 - h8
    PlusOne = 0x40201008_04020100,    // a2 - g8
    PlusTwo = 0x20100804_02010000,    // a3 - f8
    PlusThree = 0x10080402_01000000,  // a4 - e8
    PlusFour = 0x08040201_00000000,   // a5 - d8
    PlusFive = 0x04020100_00000000,   // a6 - c8
    PlusSix = 0x02010000_00000000,    // a7 - b8
    PlusSeven = 0x01000000_00000000,  // a8 - a8
}

/// An unsigned 64 bit integer representation of a chessboard, where every bit represents one square.
/// Fore more information, see the [module-level docs](crate::bitboard).
#[derive(Debug)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn iter_bits(&self) -> impl Iterator<Item = bool> {
        BitBoardIterator::new(**self)
    }

    pub fn iter_bits_masked(&self, mask: u64) -> impl Iterator<Item = bool> {
        MaskedBitBoardIterator::new(**self, mask)
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

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mask: u64 = 0b1111_1111;
        let ranks = (0..64)
            .step_by(8)
            .rev()
            .map(|rank| self.iter_bits_masked(mask.shl(rank)))
            .map(|rank| {
                rank.map(|square| match square {
                    true => 'x',
                    false => '.',
                })
                .collect::<String>()
            });
        for rank in ranks {
            writeln!(f, "{}", rank)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::bitboard::{AntiDiagonal, BitBoard, Diagonal, File, Rank};

    #[test]
    fn test_rank() {
        for (rank, string) in [
            (
                Rank::One,
                "........\n........\n........\n........\n........\n........\n........\nxxxxxxxx\n",
            ),
            (
                Rank::Two,
                "........\n........\n........\n........\n........\n........\nxxxxxxxx\n........\n",
            ),
            (
                Rank::Three,
                "........\n........\n........\n........\n........\nxxxxxxxx\n........\n........\n",
            ),
            (
                Rank::Four,
                "........\n........\n........\n........\nxxxxxxxx\n........\n........\n........\n",
            ),
            (
                Rank::Five,
                "........\n........\n........\nxxxxxxxx\n........\n........\n........\n........\n",
            ),
            (
                Rank::Six,
                "........\n........\nxxxxxxxx\n........\n........\n........\n........\n........\n",
            ),
            (
                Rank::Seven,
                "........\nxxxxxxxx\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                Rank::Eight,
                "xxxxxxxx\n........\n........\n........\n........\n........\n........\n........\n",
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
                "x.......\nx.......\nx.......\nx.......\nx.......\nx.......\nx.......\nx.......\n",
            ),
            (
                File::B,
                ".x......\n.x......\n.x......\n.x......\n.x......\n.x......\n.x......\n.x......\n",
            ),
            (
                File::C,
                "..x.....\n..x.....\n..x.....\n..x.....\n..x.....\n..x.....\n..x.....\n..x.....\n",
            ),
            (
                File::D,
                "...x....\n...x....\n...x....\n...x....\n...x....\n...x....\n...x....\n...x....\n",
            ),
            (
                File::E,
                "....x...\n....x...\n....x...\n....x...\n....x...\n....x...\n....x...\n....x...\n",
            ),
            (
                File::F,
                ".....x..\n.....x..\n.....x..\n.....x..\n.....x..\n.....x..\n.....x..\n.....x..\n",
            ),
            (
                File::G,
                "......x.\n......x.\n......x.\n......x.\n......x.\n......x.\n......x.\n......x.\n",
            ),
            (
                File::H,
                ".......x\n.......x\n.......x\n.......x\n.......x\n.......x\n.......x\n.......x\n",
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
                "........\n........\n........\n........\n........\n........\n........\nx.......\n",
            ),
            (
                Diagonal::MinusSix,
                "........\n........\n........\n........\n........\n........\nx.......\n.x......\n",
            ),
            (
                Diagonal::MinusFive,
                "........\n........\n........\n........\n........\nx.......\n.x......\n..x.....\n",
            ),
            (
                Diagonal::MinusFour,
                "........\n........\n........\n........\nx.......\n.x......\n..x.....\n...x....\n",
            ),
            (
                Diagonal::MinusThree,
                "........\n........\n........\nx.......\n.x......\n..x.....\n...x....\n....x...\n",
            ),
            (
                Diagonal::MinusTwo,
                "........\n........\nx.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n",
            ),
            (
                Diagonal::MinusOne,
                "........\nx.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n",
            ),
            (
                Diagonal::Main,
                "x.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n",
            ),
            (
                Diagonal::PlusOne,
                ".x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n........\n",
            ),
            (
                Diagonal::PlusTwo,
                "..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n........\n........\n",
            ),
            (
                Diagonal::PlusThree,
                "...x....\n....x...\n.....x..\n......x.\n.......x\n........\n........\n........\n",
            ),
            (
                Diagonal::PlusFour,
                "....x...\n.....x..\n......x.\n.......x\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::PlusFive,
                ".....x..\n......x.\n.......x\n........\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::PlusSix,
                "......x.\n.......x\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::PlusSeven,
                ".......x\n........\n........\n........\n........\n........\n........\n........\n",
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
                "........\n........\n........\n........\n........\n........\n........\n.......x\n",
            ),
            (
                AntiDiagonal::MinusSix,
                "........\n........\n........\n........\n........\n........\n.......x\n......x.\n",
            ),
            (
                AntiDiagonal::MinusFive,
                "........\n........\n........\n........\n........\n.......x\n......x.\n.....x..\n",
            ),
            (
                AntiDiagonal::MinusFour,
                "........\n........\n........\n........\n.......x\n......x.\n.....x..\n....x...\n",
            ),
            (
                AntiDiagonal::MinusThree,
                "........\n........\n........\n.......x\n......x.\n.....x..\n....x...\n...x....\n",
            ),
            (
                AntiDiagonal::MinusTwo,
                "........\n........\n.......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n",
            ),
            (
                AntiDiagonal::MinusOne,
                "........\n.......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\n",
            ),
            (
                AntiDiagonal::Main,
                ".......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n",
            ),
            (
                AntiDiagonal::PlusOne,
                "......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n........\n",
            ),
            (
                AntiDiagonal::PlusTwo,
                ".....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n........\n........\n",
            ),
            (
                AntiDiagonal::PlusThree,
                "....x...\n...x....\n..x.....\n.x......\nx.......\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::PlusFour,
                "...x....\n..x.....\n.x......\nx.......\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::PlusFive,
                "..x.....\n.x......\nx.......\n........\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::PlusSix,
                ".x......\nx.......\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::PlusSeven,
                "x.......\n........\n........\n........\n........\n........\n........\n........\n",
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
