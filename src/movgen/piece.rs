//! Basic enumerations to distinguish pieces of different colors.

pub enum Color {
    White,
    Black,
}

/// All available pieces on the chessboard.
/// The pawns are distinguished by color as their move bitboards are unique i.e., they move in opposite directions.
pub enum Piece {
    Pawn(Color),
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn value(&self) -> usize {
        match self {
            Self::Pawn(_) => 1,
            Self::Knight | Self::Bishop => 3,
            Self::Rook => 5,
            Self::Queen => 9,
            Self::King => 1_000,
        }
    }
}
