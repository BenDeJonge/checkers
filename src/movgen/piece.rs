//! Basic enumerations to distinguish pieces of different colors.

use std::fmt::Display;

use crate::fen::FENRepresentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl FENRepresentation for Color {
    fn fen(&self) -> &str {
        match *self {
            Color::White => "w",
            Color::Black => "b",
        }
    }
}

/// All available pieces on the chessboard.
#[derive(Debug, PartialEq, Eq)]
pub enum Piece {
    King(Color),
    Queen(Color),
    Rook(Color),
    Bishop(Color),
    Knight(Color),
    Pawn(Color),
}

impl FENRepresentation for Piece {
    fn fen(&self) -> &str {
        match self {
            Piece::King(Color::White) => "K",
            Piece::Queen(Color::White) => "Q",
            Piece::Rook(Color::White) => "R",
            Piece::Bishop(Color::White) => "B",
            Piece::Knight(Color::White) => "N",
            Piece::Pawn(Color::White) => "P",

            Piece::King(Color::Black) => "k",
            Piece::Queen(Color::Black) => "q",
            Piece::Rook(Color::Black) => "r",
            Piece::Bishop(Color::Black) => "b",
            Piece::Knight(Color::Black) => "n",
            Piece::Pawn(Color::Black) => "p",
        }
    }
}

pub trait Notation {
    /// Annotate moves in algebraic notation e.g., `1. e4 e5 2. Ke2! Ke7!!`.
    fn algebraic_notation(&self) -> char;
    /// Annotate moves in figurine algebraic notation e.g., `1. e4 e5 2. ♔e2! ♔e7!!`.
    fn figurine_notation(&self) -> char;
}

impl Notation for Piece {
    fn algebraic_notation(&self) -> char {
        match self {
            Piece::King(_) => 'K',
            Piece::Queen(_) => 'Q',
            Piece::Rook(_) => 'R',
            Piece::Bishop(_) => 'B',
            Piece::Knight(_) => 'N',
            Piece::Pawn(_) => '\0', // Pawn moves are written as simply `e4`.
        }
    }

    fn figurine_notation(&self) -> char {
        match self {
            Piece::King(_) => '♔',
            Piece::Queen(_) => '♕',
            Piece::Rook(_) => '♖',
            Piece::Bishop(_) => '♗',
            Piece::Knight(_) => '♘',
            Piece::Pawn(_) => '\0', // Pawn moves are written as simply `e4`.
        }
    }
}

pub trait BoardNotation {
    /// Get a single-space character to populate an ASCII chess board.
    fn board_notation(&self) -> char;
}
impl BoardNotation for Piece {
    fn board_notation(&self) -> char {
        match self {
            Piece::King(Color::White) => 'K',
            Piece::Queen(Color::White) => 'Q',
            Piece::Rook(Color::White) => 'R',
            Piece::Bishop(Color::White) => 'B',
            Piece::Knight(Color::White) => 'N',
            Piece::Pawn(Color::White) => 'P',

            Piece::King(Color::Black) => 'k',
            Piece::Queen(Color::Black) => 'q',
            Piece::Rook(Color::Black) => 'r',
            Piece::Bishop(Color::Black) => 'b',
            Piece::Knight(Color::Black) => 'n',
            Piece::Pawn(Color::Black) => 'p',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Piece::King(Color::White) => '♔',
            Piece::Queen(Color::White) => '♕',
            Piece::Rook(Color::White) => '♖',
            Piece::Bishop(Color::White) => '♗',
            Piece::Knight(Color::White) => '♘',
            Piece::Pawn(Color::White) => '♙',

            Piece::King(Color::Black) => '♚',
            Piece::Queen(Color::Black) => '♛',
            Piece::Rook(Color::Black) => '♜',
            Piece::Bishop(Color::Black) => '♝',
            Piece::Knight(Color::Black) => '♞',
            Piece::Pawn(Color::Black) => '♟',
        };
        write!(f, "{ch}")
    }
}
impl Piece {
    pub fn value(&self) -> usize {
        match self {
            Piece::Pawn(_) => 1,
            Piece::Knight(_) | Piece::Bishop(_) => 3,
            Piece::Rook(_) => 5,
            Piece::Queen(_) => 9,
            Piece::King(_) => 1_000,
        }
    }
}
