//! Basic enumerations to distinguish pieces of different colors.

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

/// All available pieces on the chessboard.
#[derive(Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
}

impl Piece {
    pub fn value(&self) -> usize {
        match self {
            Self::Pawn(_) => 1,
            Self::Knight(_) | Self::Bishop(_) => 3,
            Self::Rook(_) => 5,
            Self::Queen(_) => 9,
            Self::King(_) => 1_000,
        }
    }

    /// Get a single-space character to populate an ASCII chess board.
    pub fn board_representation(&self) -> char {
        match self {
            Self::King(Color::White) => 'K',
            Self::Queen(Color::White) => 'Q',
            Self::Rook(Color::White) => 'R',
            Self::Bishop(Color::White) => 'B',
            Self::Knight(Color::White) => 'N',
            Self::Pawn(Color::White) => 'P',
            Self::King(Color::Black) => 'k',
            Self::Queen(Color::Black) => 'q',
            Self::Rook(Color::Black) => 'r',
            Self::Bishop(Color::Black) => 'b',
            Self::Knight(Color::Black) => 'n',
            Self::Pawn(Color::Black) => 'p',
        }
    }

    /// Annotate moves in algebraic notation e.g., `1. e4 e5 2. Ke2! Ke7!!`.
    pub fn algebraic_name(&self) -> char {
        match self {
            Self::King(_) => 'K',
            Self::Queen(_) => 'Q',
            Self::Rook(_) => 'R',
            Self::Bishop(_) => 'B',
            Self::Knight(_) => 'N',
            Self::Pawn(_) => '\0', // Pawn moves are written as simply `e4`.
        }
    }

    /// Annotate moves in figurine algebraic notation e.g., `1. e4 e5 2. ♔e2! ♔e7!!`.
    pub fn figurine(&self) -> char {
        match self {
            Self::King(_) => '♔',
            Self::Queen(_) => '♕',
            Self::Rook(_) => '♖',
            Self::Bishop(_) => '♗',
            Self::Knight(_) => '♘',
            Self::Pawn(_) => '\0', // Pawn moves are written as simply `e4`.
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match *self {
            Self::King(Color::White) => '♔',
            Self::Queen(Color::White) => '♕',
            Self::Rook(Color::White) => '♖',
            Self::Bishop(Color::White) => '♗',
            Self::Knight(Color::White) => '♘',
            Self::Pawn(Color::White) => '♙',
            Self::King(Color::Black) => '♚',
            Self::Queen(Color::Black) => '♛',
            Self::Rook(Color::Black) => '♜',
            Self::Bishop(Color::Black) => '♝',
            Self::Knight(Color::Black) => '♞',
            Self::Pawn(Color::Black) => '♟',
        };
        write!(f, "{ch}")
    }
}
