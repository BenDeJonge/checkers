//! Methods for validating and parsing FEN strings.
//!
//! A valid FEN string contains exactly 6 parts and is structured as below.
//! ```text
//! "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
//!  ┌──────────────────────────────────────────── ┬ ───┐ ─┐ ┐ ┐
//!  └ 1. Board                   2. Active player ┘    |  | | |
//!                                  3. Castling rights ┘  | | |
//!                                   4. En passant square ┘ | |
//!                                       5. Half move clock ┘ |
//!                                            6. Move counter ┘
//! ```
//!
//! This module contains methods to validate the formatting of all parts of the
//! string and some basic logical checks of the resulting board state. These
//! checks are deliberately minimal, to avoid limiting the engine to boardstates
//! that can be reached legally, as this would exclude Chess960, puzzles etc.

use std::{convert::TryInto, num::NonZero};

use crate::{
    game::CastlingRights,
    movgen::{
        bitboard::Rank,
        piece::{Color, Piece},
    },
    square::{Square, get_square_from_name},
};

pub trait FENRepresentation {
    fn fen(&self) -> &str;
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidFENString<'a> {
    PartsCount(usize),
    RankLength(usize),
    Color(&'a str),
    CastlingRights(&'a str),
    MoveLength(usize, &'a str),
    Square(&'a str),
    PawnMoveToUnreachableRank(Rank),
    Piece(&'a str),
    EnPassantRank(Rank),
    HalfMoveClockNAN(&'a str),
    HalfMoveClockOutOfBounds(usize),
    HalfMoveClockTooLargeForMoveCounter(usize, NonZero<usize>),
    MoveCounterNAN(&'a str),
    MoveCounterZero,
}

pub fn try_get_fen_parts(fen: &str) -> Result<[&str; 6], InvalidFENString<'_>> {
    let parts = fen.split(' ').collect::<Vec<&str>>();
    if parts.len() != 6 {
        Err(InvalidFENString::PartsCount(parts.len()))
    } else {
        Ok(vec_to_arr(parts))
    }
}

pub(crate) type FENBoard = [Option<Piece>; 64];

pub fn try_parse_board(board: &str) -> Result<FENBoard, InvalidFENString<'_>> {
    // https://chess.stackexchange.com/questions/1482/how-do-you-know-when-a-fen-position-is-legal
    // - count rows/cols
    // - count pieces
    // - empty squares/row <= 8
    // - 1 white and 1 black king
    // - kings separated by at least 1 square
    // - non-active color not in check
    // - active color not in illegal check (3+ or B+B, N+N, P+(P,B,N))
    // - pawns not in rank 1 and 8
    // - castling flag matches king/rook position (excludes chess 960)
    // - check that the correct piece is on the square of the last move
    todo!()
}

fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

/// Parse a color from a string exactly `"w"` or "`b`".
pub fn try_parse_active_player(player: &str) -> Result<Color, InvalidFENString<'_>> {
    match player {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(InvalidFENString::Color(player)),
    }
}

/// Parse castling rights from a string:
/// - `"KQ"` for white castling rights
/// - `"kq"` for black castling rights
/// - `"-"` for no castling rights
pub fn try_parse_castling_rights(
    castling: &str,
) -> Result<[CastlingRights; 2], InvalidFENString<'_>> {
    // There are two sides with four castling options (K, Q, KQ, -) leading to only 16 possibilities.
    let both = CastlingRights::new(true, true);
    let king = CastlingRights::new(true, false);
    let queen = CastlingRights::new(false, true);
    let neither = CastlingRights::new(false, false);

    match castling {
        // White both
        "KQkq" => Ok([both, both]),
        "KQk" => Ok([both, king]),
        "KQq" => Ok([both, queen]),
        "KQ" => Ok([both, neither]),
        // White kingside
        "Kkq" => Ok([king, both]),
        "Kk" => Ok([king, king]),
        "Kq" => Ok([king, queen]),
        "K" => Ok([king, neither]),
        // White queenside
        "Qkq" => Ok([queen, both]),
        "Qk" => Ok([queen, king]),
        "Qq" => Ok([queen, queen]),
        "Q" => Ok([queen, neither]),
        // White neither
        "kq" => Ok([neither, both]),
        "k" => Ok([neither, king]),
        "q" => Ok([neither, queen]),
        // Neither side can castle which is written as "-" instead of "".
        "-" => Ok([neither, neither]),
        _ => Err(InvalidFENString::CastlingRights(castling)),
    }
}

pub(crate) struct Move {
    piece: Piece,
    square: Square,
}

impl Move {
    pub fn new(piece: Piece, square: Square) -> Self {
        Self { piece, square }
    }
}

const fn get_invalid_pawn_ranks_passive_player(color: Color) -> [Rank; 2] {
    // This method might seem backwards but we need to get the en-passant square
    // from the passive player, not the currently active one. In other words,
    // the FEN string shows where the player on move can capture en passant.
    match color {
        Color::White => [Rank::Seven, Rank::Eight],
        Color::Black => [Rank::One, Rank::Two],
    }
}

/// Parse a move `"Ke2"` or `"d4"` into a move string.
/// Invalid pawn moves such as
pub fn try_parse_last_move(color: Color, last_move: &str) -> Result<Move, InvalidFENString<'_>> {
    match last_move.len() {
        2 => {
            let move_ =
                try_parse_square(last_move).map(|square| Move::new(Piece::Pawn(color), square))?;
            let rank = Rank::try_from(move_.square.rank).unwrap();
            if get_invalid_pawn_ranks_passive_player(color).contains(&rank) {
                Err(InvalidFENString::PawnMoveToUnreachableRank(rank))
            } else {
                Ok(move_)
            }
        }
        3 => {
            let piece = try_parse_piece(color, &last_move[..1])?;
            let square = try_parse_square(&last_move[1..])?;
            Ok(Move::new(piece, square))
        }
        len => Err(InvalidFENString::MoveLength(len, last_move)),
    }
}

fn try_parse_square(square: &str) -> Result<Square, InvalidFENString<'_>> {
    get_square_from_name(square).ok_or(InvalidFENString::Square(square))
}

fn try_parse_piece(color: Color, piece: &str) -> Result<Piece, InvalidFENString<'_>> {
    match piece {
        "K" => Ok(Piece::King(color)),
        "Q" => Ok(Piece::Queen(color)),
        "R" => Ok(Piece::Rook(color)),
        "B" => Ok(Piece::Bishop(color)),
        "N" => Ok(Piece::Knight(color)),
        _ => Err(InvalidFENString::Piece(piece)),
    }
}

const fn get_valid_en_passant_rank(color: Color) -> Rank {
    match color {
        Color::White => Rank::Three,
        Color::Black => Rank::Five,
    }
}

/// Convert an *en passant* square `"e3"` to a `Square`.
pub fn try_parse_en_passant_square(
    color: Color,
    en_passant: &str,
) -> Result<Square, InvalidFENString<'_>> {
    if let Some(square) = get_square_from_name(en_passant) {
        if square.rank != get_valid_en_passant_rank(color).into() {
            Err(InvalidFENString::EnPassantRank(
                square.rank.try_into().unwrap(),
            ))
        } else {
            Ok(square)
        }
    } else {
        Err(InvalidFENString::Square(en_passant))
    }
}

/// Parse a move clock `"42"` to an integer up to 100, smaller than twice the move counter.
pub fn try_parse_half_move_clock(
    half_move_clock: &str,
    move_clock: NonZero<usize>,
) -> Result<usize, InvalidFENString<'_>> {
    let value = half_move_clock
        .parse::<usize>()
        .map_err(|_| InvalidFENString::HalfMoveClockNAN(half_move_clock))?;
    // Imagine consecutive Knight moves from both sides from the starting position.
    // active player       | w b w b w b w b w ...
    // full-move counter   | 1 1 2 2 3 3 4 4 5 ...
    // half-move clock     | 0 1 2 3 4 5 6 7 8 ...
    // half-move clock / 2 | 0 0 1 1 2 2 3 3 4 ...
    if value / 2 >= move_clock.get() {
        Err(InvalidFENString::HalfMoveClockTooLargeForMoveCounter(
            value, move_clock,
        ))
    }
    // 9.6.2 any series of at least 75 moves have been made by each player without the movement of any pawn and without
    // any capture. If the last move resulted in checkmate, that shall take precedence.
    // https://handbook.fide.com/chapter/e012023
    else if value > 150 {
        Err(InvalidFENString::HalfMoveClockOutOfBounds(value))
    } else {
        Ok(value)
    }
}

/// Parse a move counter `"42"` into a non-zero integer.
pub fn try_parse_move_clock(move_clock: &str) -> Result<NonZero<usize>, InvalidFENString<'_>> {
    move_clock
        .parse::<usize>()
        .map_err(|_| InvalidFENString::MoveCounterNAN(move_clock))?
        .try_into()
        .map_err(|_| InvalidFENString::MoveCounterZero)
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::{
        fen::{
            InvalidFENString, try_get_fen_parts, try_parse_active_player, try_parse_board,
            try_parse_castling_rights, try_parse_en_passant_square, try_parse_half_move_clock,
            try_parse_move_clock,
        },
        game::CastlingRights,
        movgen::{bitboard::Rank, piece::Color},
        square::SQUARES,
    };

    #[test]
    fn test_try_get_fen_parts() {
        assert_eq!(
            try_get_fen_parts("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2"),
            Ok(["r6r/1b2k1bq/8/8/7B/8/8/R3K2R", "b", "KQ", "-", "3", "2"])
        );
        assert_eq!(
            try_get_fen_parts("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3"),
            Ok(["8/8/8/2k5/2pP4/8/B7/4K3", "b", "-", "d3", "0", "3"])
        );
        // This method does not take the validity of the elements into account.
        assert_eq!(
            try_get_fen_parts("my username on lichess.org is BenSchwanz"),
            Ok(["my", "username", "on", "lichess.org", "is", "BenSchwanz"])
        );
        // Missing board
        assert_eq!(
            try_get_fen_parts("b - d3 0 3"),
            Err(InvalidFENString::PartsCount(5))
        );
        // Missing board and active player.
        assert_eq!(
            try_get_fen_parts("- d3 0 3"),
            Err(InvalidFENString::PartsCount(4))
        );
        // Missing board, active player, castling rights.
        assert_eq!(
            try_get_fen_parts("d3 0 3"),
            Err(InvalidFENString::PartsCount(3))
        );
        assert_eq!(try_get_fen_parts(""), Err(InvalidFENString::PartsCount(1)));
        assert_eq!(
            try_get_fen_parts("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3 idontbelonghere"),
            Err(InvalidFENString::PartsCount(7))
        );
    }

    #[test]
    fn test_try_parse_to_play() {
        assert_eq!(try_parse_active_player("w"), Ok(Color::White));
        assert_eq!(try_parse_active_player("b"), Ok(Color::Black));
        assert_eq!(
            try_parse_active_player("W"),
            Err(InvalidFENString::Color("W"))
        );
        assert_eq!(
            try_parse_active_player("B"),
            Err(InvalidFENString::Color("B"))
        );
        assert_eq!(
            try_parse_active_player("monopoly"),
            Err(InvalidFENString::Color("monopoly"))
        );
    }

    #[test]
    fn test_try_parse_castling_rights() {
        assert_eq!(
            try_parse_castling_rights("KQkq"),
            Ok([
                CastlingRights::new(true, true),
                CastlingRights::new(true, true),
            ])
        );
        // Order is important.
        assert_eq!(
            try_parse_castling_rights("QKqk"),
            Err(InvalidFENString::CastlingRights("QKqk"))
        );
        assert_eq!(
            try_parse_castling_rights("backgammon"),
            Err(InvalidFENString::CastlingRights("backgammon"))
        );
    }

    #[test]
    fn test_try_parse_en_passant_square() {
        assert_eq!(
            try_parse_en_passant_square(Color::White, "e3"),
            Ok(SQUARES[20])
        );
        assert_eq!(
            try_parse_en_passant_square(Color::Black, "e5"),
            Ok(SQUARES[36])
        );
        // White can never have a pawn that can be captured en passant on the 4th rank.
        assert_eq!(
            try_parse_en_passant_square(Color::Black, "e4"),
            Err(InvalidFENString::EnPassantRank(Rank::Four))
        );
        // Black can never have a pawn that can be captured en passant on the 4th rank.
        assert_eq!(
            try_parse_en_passant_square(Color::Black, "e4"),
            Err(InvalidFENString::EnPassantRank(Rank::Four))
        );
        assert_eq!(
            try_parse_en_passant_square(Color::Black, "tic-tac-toe"),
            Err(InvalidFENString::Square("tic-tac-toe"))
        );
    }

    #[test]
    fn test_try_parse_half_move_clock() {
        assert_eq!(
            try_parse_half_move_clock("0", NonZero::new(1).unwrap()),
            Ok(0)
        );
        assert_eq!(
            try_parse_half_move_clock("1", NonZero::new(1).unwrap()),
            Ok(1)
        );
        assert_eq!(
            try_parse_half_move_clock("2", NonZero::new(2).unwrap()),
            Ok(2)
        );
        assert_eq!(
            try_parse_half_move_clock("3", NonZero::new(2).unwrap()),
            Ok(3)
        );
        assert_eq!(
            try_parse_half_move_clock("5", NonZero::new(11).unwrap()),
            Ok(5)
        );
        assert_eq!(
            try_parse_half_move_clock("8", NonZero::new(5).unwrap()),
            Ok(8)
        );
        assert_eq!(
            try_parse_half_move_clock("9", NonZero::new(5).unwrap()),
            Ok(9)
        );
        assert_eq!(
            try_parse_half_move_clock("10", NonZero::new(5).unwrap()),
            Err(InvalidFENString::HalfMoveClockTooLargeForMoveCounter(
                10,
                NonZero::new(5).unwrap()
            ))
        );
        assert_eq!(
            try_parse_half_move_clock("11", NonZero::new(5).unwrap()),
            Err(InvalidFENString::HalfMoveClockTooLargeForMoveCounter(
                11,
                NonZero::new(5).unwrap()
            ))
        );
        assert_eq!(
            try_parse_half_move_clock("catan", NonZero::new(5).unwrap()),
            Err(InvalidFENString::HalfMoveClockNAN("catan"))
        );
    }

    #[test]
    fn test_try_parse_move_clock() {
        assert_eq!(try_parse_move_clock("1"), Ok(NonZero::new(1).unwrap()));
        assert_eq!(
            try_parse_move_clock(&format!("{}", usize::MAX)),
            Ok(NonZero::new(usize::MAX).unwrap())
        );
        assert_eq!(
            try_parse_move_clock("0"),
            Err(InvalidFENString::MoveCounterZero)
        );
        assert_eq!(
            try_parse_move_clock("-1"),
            Err(InvalidFENString::MoveCounterNAN("-1"))
        );
        assert_eq!(
            try_parse_move_clock("Bridge"),
            Err(InvalidFENString::MoveCounterNAN("Bridge"))
        );
    }

    #[test]
    fn test_try_parse_board() {
        // Starting position
        assert_eq!(
            try_parse_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"),
            todo!()
        );
        // Evergreen game
        assert_eq!(
            try_parse_board("1r3kr1/pbpBBp1p/1b3P2/8/8/2P2q2/P4PPP/3R2K1"),
            todo!()
        );
    }
}
