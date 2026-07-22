//! Basic enumerations to distinguish pieces of different colors.

use std::fmt::Display;

#[derive(Clone, Copy)]
pub enum Color {
    White,
    Black,
}

/// All available pieces on the chessboard.
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
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match *self {
            Self::Pawn(Color::White) => '♙',
            Self::Knight(Color::White) => '♘',
            Self::Bishop(Color::White) => '♗',
            Self::Rook(Color::White) => '♖',
            Self::Queen(Color::White) => '♕',
            Self::King(Color::White) => '♔',
            Self::Pawn(Color::Black) => '♟',
            Self::Knight(Color::Black) => '♞',
            Self::Bishop(Color::Black) => '♝',
            Self::Rook(Color::Black) => '♜',
            Self::Queen(Color::Black) => '♛',
            Self::King(Color::Black) => '♚',
        };
        write!(f, "{ch}")
    }
}
