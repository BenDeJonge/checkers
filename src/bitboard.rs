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
    A1toA1 = 0x00000000_00000001,
    A2toB1 = 0x00000000_00000102,
    A3toC1 = 0x00000000_00010204,
    A4toD1 = 0x00000000_01020408,
    A5toE1 = 0x00000001_02040810,
    A6toF1 = 0x00000102_04081020,
    A7toG1 = 0x00010204_08102040,
    A8toH1 = 0x01020408_10204080,
    H2toB8 = 0x02040810_20408000,
    H3toC8 = 0x04081020_40800000,
    H4toD8 = 0x08102040_80000000,
    H5toE8 = 0x10204080_00000000,
    H6toF8 = 0x20408000_00000000,
    H7toG8 = 0x40800000_00000000,
    H8toH8 = 0x80000000_00000000,
}

/// Antidiagonals are SW-to-NE lines, similar to matrix terminology.
/// These can be represented by increasing and decreasing distances from the left edge.
#[repr(u64)]
#[derive(Clone, Copy, Debug)]
pub enum AntiDiagonal {
    A8toA8 = 0x01000000_00000000,
    A7toB8 = 0x02010000_00000000,
    A6toC8 = 0x04020100_00000000,
    A5toD8 = 0x08040201_00000000,
    A4toE8 = 0x10080402_01000000,
    A3toF8 = 0x20100804_02010000,
    A2toG8 = 0x40201008_04020100,
    A1toH8 = 0x80402010_08040201,
    H7toB1 = 0x00804020_10080402,
    H6toC1 = 0x00008040_20100804,
    H5toD1 = 0x00000080_40201008,
    H4toE1 = 0x00000000_80402010,
    H3toF1 = 0x00000000_00804020,
    H2toG1 = 0x00000000_00008040,
    H1toH1 = 0x00000000_00000080,
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
            .inspect(|shift| {
                dbg!(shift);
            })
            .inspect(|rank| {
                dbg!(format!("{:064b}", mask.shl(rank)));
            })
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
                Diagonal::A1toA1,
                "........\n........\n........\n........\n........\n........\n........\nx.......\n",
            ),
            (
                Diagonal::A2toB1,
                "........\n........\n........\n........\n........\n........\nx.......\n.x......\n",
            ),
            (
                Diagonal::A3toC1,
                "........\n........\n........\n........\n........\nx.......\n.x......\n..x.....\n",
            ),
            (
                Diagonal::A4toD1,
                "........\n........\n........\n........\nx.......\n.x......\n..x.....\n...x....\n",
            ),
            (
                Diagonal::A5toE1,
                "........\n........\n........\nx.......\n.x......\n..x.....\n...x....\n....x...\n",
            ),
            (
                Diagonal::A6toF1,
                "........\n........\nx.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n",
            ),
            (
                Diagonal::A7toG1,
                "........\nx.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n",
            ),
            (
                Diagonal::A8toH1,
                "x.......\n.x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n",
            ),
            (
                Diagonal::H2toB8,
                ".x......\n..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n........\n",
            ),
            (
                Diagonal::H3toC8,
                "..x.....\n...x....\n....x...\n.....x..\n......x.\n.......x\n........\n........\n",
            ),
            (
                Diagonal::H4toD8,
                "...x....\n....x...\n.....x..\n......x.\n.......x\n........\n........\n........\n",
            ),
            (
                Diagonal::H5toE8,
                "....x...\n.....x..\n......x.\n.......x\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::H6toF8,
                ".....x..\n......x.\n.......x\n........\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::H7toG8,
                "......x.\n.......x\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                Diagonal::H8toH8,
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
                AntiDiagonal::A1toH8,
                ".......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n",
            ),
            (
                AntiDiagonal::A2toG8,
                "......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n........\n",
            ),
            (
                AntiDiagonal::A3toF8,
                ".....x..\n....x...\n...x....\n..x.....\n.x......\nx.......\n........\n........\n",
            ),
            (
                AntiDiagonal::A4toE8,
                "....x...\n...x....\n..x.....\n.x......\nx.......\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::A5toD8,
                "...x....\n..x.....\n.x......\nx.......\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::A6toC8,
                "..x.....\n.x......\nx.......\n........\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::A7toB8,
                ".x......\nx.......\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::A8toA8,
                "x.......\n........\n........\n........\n........\n........\n........\n........\n",
            ),
            (
                AntiDiagonal::H7toB1,
                "........\n.......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n.x......\n",
            ),
            (
                AntiDiagonal::H6toC1,
                "........\n........\n.......x\n......x.\n.....x..\n....x...\n...x....\n..x.....\n",
            ),
            (
                AntiDiagonal::H5toD1,
                "........\n........\n........\n.......x\n......x.\n.....x..\n....x...\n...x....\n",
            ),
            (
                AntiDiagonal::H4toE1,
                "........\n........\n........\n........\n.......x\n......x.\n.....x..\n....x...\n",
            ),
            (
                AntiDiagonal::H3toF1,
                "........\n........\n........\n........\n........\n.......x\n......x.\n.....x..\n",
            ),
            (
                AntiDiagonal::H2toG1,
                "........\n........\n........\n........\n........\n........\n.......x\n......x.\n",
            ),
            (
                AntiDiagonal::H1toH1,
                "........\n........\n........\n........\n........\n........\n........\n.......x\n",
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
