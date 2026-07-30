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

use std::{convert::TryInto, num::NonZero, ops::Deref};

use strum::IntoEnumIterator;

use crate::{
    game::CastlingRights,
    movgen::{
        bitboard::Rank,
        piece::{
            Color::{self, Black, White},
            Piece::{self, Bishop, King, Knight, Pawn, Queen, Rook},
        },
    },
    square::{Square, get_square_from_name},
};

pub trait FENRepresentation {
    fn fen(&self) -> char;
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidFENString<'a> {
    PartsCount(usize),
    // try_parse_board
    BoardOutOfBounds(usize),
    RankCount(usize),
    RankIndex(usize),
    RankEmptyOutOfBounds(usize),
    UnknownRankChar(char),
    MultipleKings(Color),
    AbsentKing(Color),
    PawnOnUnreachableRank(Color),
    NoPawnOnEnPassantSquare(usize),
    // try_parse_color
    Color(&'a str),
    // try_parse_castling_rights
    CastlingRights(&'a str),
    // try_parse_en_passant_square
    Square(&'a str),
    Piece(&'a str),
    EnPassantRank(Rank),
    // try_parse_half_move_clock
    HalfMoveClockNAN(&'a str),
    HalfMoveClockOutOfBounds(usize),
    HalfMoveClockTooLargeForMoveCounter(usize, NonZero<usize>),
    // try_parse_move_counter
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

fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

#[derive(PartialEq, Debug)]
pub struct FENBoard([Option<Piece>; 64]);
impl Deref for FENBoard {
    type Target = [Option<Piece>; 64];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct InvalidFENBoardChar(char);
impl TryFrom<[[char; 8]; 8]> for FENBoard {
    type Error = InvalidFENBoardChar;
    fn try_from(value: [[char; 8]; 8]) -> Result<Self, Self::Error> {
        let mut board = [None; 64];
        for (r, row) in value.iter().enumerate() {
            for (c, ch) in row.iter().enumerate() {
                let piece = match *ch {
                    'K' => Some(King(White)),
                    'Q' => Some(Queen(White)),
                    'R' => Some(Rook(White)),
                    'B' => Some(Bishop(White)),
                    'N' => Some(Knight(White)),
                    'P' => Some(Pawn(White)),

                    'k' => Some(King(Black)),
                    'q' => Some(Queen(Black)),
                    'r' => Some(Rook(Black)),
                    'b' => Some(Bishop(Black)),
                    'n' => Some(Knight(Black)),
                    'p' => Some(Pawn(Black)),

                    ' ' => None,

                    other => return Err(InvalidFENBoardChar(other)),
                };

                let idx = r * 8 + c;
                board[idx] = piece;
            }
        }

        Ok(FENBoard(board))
    }
}

pub fn try_parse_board<'a>(
    board: &'a str,
    en_passant: Option<&Square>,
    active_player: Color,
) -> Result<FENBoard, InvalidFENString<'a>> {
    // https://chess.stackexchange.com/questions/1482/how-do-you-know-when-a-fen-position-is-legal
    let mut fen_vec: Vec<Option<Piece>> = Vec::with_capacity(64);
    let ranks = try_get_board_ranks(board)?;
    for rank in ranks.iter() {
        fen_vec.extend(try_parse_rank(rank)?);
    }
    let fen_board = FENBoard(vec_to_arr(fen_vec));

    assert_pawns_on_allowed_ranks(&fen_board)?;
    assert_valid_checks(&fen_board)?;
    assert_exactly_one_king(&fen_board)?;
    assert_valid_en_passant_square(&fen_board, en_passant, active_player)?;

    Ok(fen_board)
}

fn try_get_board_ranks(board: &str) -> Result<[&str; 8], InvalidFENString<'_>> {
    // Two boundaries:
    // - Completely empty board of kings:
    //   17 chars: 7 empty rows "8" + 1 rows "x6x" + 7 row separators "/"
    //   K6k/8/8/8/8/8/8/8
    // - Completely full board of bishops:
    //   71 chars: 8x8 squares + 7 row separators "/"
    //   bBbBkBbB/BbBbBbBb/bBbBbBbB/BbBbbbBb/bBbBbBbB/BbBbBbBb/bBbBbBbB/BbBbKbBb
    if !(17..=71).contains(&board.len()) {
        return Err(InvalidFENString::BoardOutOfBounds(board.len()));
    }
    let ranks = board.split('/').collect::<Vec<&str>>();
    if ranks.len() != 8 {
        return Err(InvalidFENString::RankCount(ranks.len()));
    }
    Ok(vec_to_arr(ranks))
}

fn try_parse_rank(rank: &str) -> Result<[Option<Piece>; 8], InvalidFENString<'_>> {
    if rank.len() > 8 {
        return Err(InvalidFENString::RankIndex(rank.len()));
    }
    let mut array = [None; 8];
    let mut i = 0;
    let mut j = 0;
    for ch in rank.chars() {
        match ch {
            '0'..='9' => {
                j = j * 10 + ch as usize - '0' as usize;
                if j > 8 {
                    return Err(InvalidFENString::RankEmptyOutOfBounds(j));
                }
            }
            alpha => {
                let piece = match alpha {
                    'K' => Ok(Piece::King(Color::White)),
                    'Q' => Ok(Piece::Queen(Color::White)),
                    'R' => Ok(Piece::Rook(Color::White)),
                    'B' => Ok(Piece::Bishop(Color::White)),
                    'N' => Ok(Piece::Knight(Color::White)),
                    'P' => Ok(Piece::Pawn(Color::White)),

                    'k' => Ok(Piece::King(Color::Black)),
                    'q' => Ok(Piece::Queen(Color::Black)),
                    'r' => Ok(Piece::Rook(Color::Black)),
                    'b' => Ok(Piece::Bishop(Color::Black)),
                    'n' => Ok(Piece::Knight(Color::Black)),
                    'p' => Ok(Piece::Pawn(Color::Black)),
                    unknown => Err(InvalidFENString::UnknownRankChar(unknown)),
                }?;
                i += j;
                j = 0;
                if i >= 8 {
                    return Err(InvalidFENString::RankIndex(i));
                }
                array[i] = Some(piece);
                i += 1;
            }
        }
    }
    Ok(array)
}

#[derive(Default)]
struct Kings {
    white: bool,
    black: bool,
}

enum KingError {
    Multiple(Color),
    Absent(Color),
}

impl Kings {
    pub fn try_add(&mut self, color: Color) -> Result<(), KingError> {
        match color {
            Color::White => self.try_add_white(),
            Color::Black => self.try_add_black(),
        }
    }
    fn try_add_white(&mut self) -> Result<(), KingError> {
        if self.white {
            Err(KingError::Multiple(Color::White))
        } else {
            self.white = true;
            Ok(())
        }
    }
    fn try_add_black(&mut self) -> Result<(), KingError> {
        if self.black {
            Err(KingError::Multiple(Color::Black))
        } else {
            self.black = true;
            Ok(())
        }
    }
    pub fn verify(&self) -> Result<(), KingError> {
        if !self.white {
            Err(KingError::Absent(Color::White))
        } else if !self.black {
            Err(KingError::Absent(Color::Black))
        } else {
            Ok(())
        }
    }
}

fn assert_exactly_one_king<'a>(board: &FENBoard) -> Result<(), InvalidFENString<'a>> {
    let mut kings = Kings::default();
    for piece in board.iter().filter(|piece| piece.is_some()) {
        match piece {
            Some(Piece::King(color)) => kings
                .try_add(*color)
                .map_err(|_| InvalidFENString::MultipleKings(*color))?,
            _ => {
                continue;
            }
        }
    }
    kings.verify().map_err(|err| match err {
        KingError::Absent(color) => InvalidFENString::AbsentKing(color),
        _ => unreachable!(),
    })
}

fn assert_pawns_on_allowed_ranks<'a>(board: &FENBoard) -> Result<(), InvalidFENString<'a>> {
    for color in Color::iter() {
        if [&board[0..8], &board[56..64]]
            .iter()
            .any(|rank| rank.contains(&Some(Piece::Pawn(color))))
        {
            return Err(InvalidFENString::PawnOnUnreachableRank(color));
        }
    }
    Ok(())
}

fn assert_valid_checks<'a>(board: &FENBoard) -> Result<(), InvalidFENString<'a>> {
    // TODO: implement me when checks are implemented
    // - kings separated by at least 1 square
    // - non-active color not in check
    // - active color not in illegal check (3+ or B+B, N+N, P+(P,B,N))
    Ok(())
}

fn assert_valid_en_passant_square<'a>(
    board: &FENBoard,
    en_passant: Option<&Square>,
    active_player: Color,
) -> Result<(), InvalidFENString<'a>> {
    if let Some(square) = en_passant {
        let rank = match active_player {
            // White/black captures en passant "above"/"below" the black/white pawn.
            Color::White => square.rank + 1,
            Color::Black => square.rank - 1,
        };
        dbg!(rank, square);
        let idx = rank * 8 + square.file;
        dbg!(board, board[idx]);
        dbg!(board.get(idx));
        if board.get(idx) != Some(&Some(Piece::Pawn(active_player.opposite()))) {
            Err(InvalidFENString::NoPawnOnEnPassantSquare(idx))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Move {
    piece: Piece,
    square: Square,
}

impl Move {
    pub fn new(piece: Piece, square: Square) -> Self {
        Self { piece, square }
    }
}

const fn get_invalid_pawn_ranks_passive_player(active_player: Color) -> [Rank; 2] {
    // This method might seem backwards but we need to get the en-passant square
    // from the passive player, not the currently active one. In other words,
    // the FEN string shows where the player on move can capture en passant.
    match active_player {
        Color::White => [Rank::Seven, Rank::Eight],
        Color::Black => [Rank::One, Rank::Two],
    }
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

/// Get the rank of the square the active player can capture on.
fn get_valid_en_passant_rank(active_player: Color) -> Rank {
    match active_player {
        Color::White => Rank::Six,
        Color::Black => Rank::Three,
    }
}

/// Convert an *en passant* square `"e3"` to a `Square`.
pub fn try_parse_en_passant_square(
    active_player: Color,
    en_passant: &str,
) -> Result<Option<Square>, InvalidFENString<'_>> {
    if en_passant == "-" {
        return Ok(None);
    }
    if let Some(square) = get_square_from_name(en_passant) {
        if square.rank != get_valid_en_passant_rank(active_player).into() {
            Err(InvalidFENString::EnPassantRank(
                square.rank.try_into().unwrap(),
            ))
        } else {
            Ok(Some(square))
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
    // 9.6.2 any series of at least 75 moves have been made by each player
    // without the movement of any pawn and without any capture.
    // If the last move resulted in checkmate, that shall take precedence.
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
            FENBoard, InvalidFENString, try_get_fen_parts, try_parse_active_player,
            try_parse_board, try_parse_castling_rights, try_parse_en_passant_square,
            try_parse_half_move_clock, try_parse_move_clock, try_parse_rank,
        },
        game::CastlingRights,
        movgen::{
            bitboard::Rank,
            piece::{
                Color::{Black, White},
                Piece::{Bishop, King, Knight, Queen, Rook},
            },
        },
        square::get_square_from_name,
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
        assert_eq!(try_parse_active_player("w"), Ok(White));
        assert_eq!(try_parse_active_player("b"), Ok(Black));
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
            try_parse_en_passant_square(White, "e6"),
            Ok(get_square_from_name("e6"))
        );
        assert_eq!(
            try_parse_en_passant_square(Black, "e3"),
            Ok(get_square_from_name("e3"))
        );
        assert_eq!(try_parse_en_passant_square(Black, "-"), Ok(None));
        // White can never have a pawn that can be captured en passant on the 4th rank.
        assert_eq!(
            try_parse_en_passant_square(Black, "e4"),
            Err(InvalidFENString::EnPassantRank(Rank::Four))
        );
        // Black can never have a pawn that can be captured en passant on the 4th rank.
        assert_eq!(
            try_parse_en_passant_square(Black, "e4"),
            Err(InvalidFENString::EnPassantRank(Rank::Four))
        );
        assert_eq!(
            try_parse_en_passant_square(Black, "tic-tac-toe"),
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
            try_parse_move_clock("bridge"),
            Err(InvalidFENString::MoveCounterNAN("bridge"))
        );
    }

    #[test]
    fn test_try_parse_rank() {
        assert_eq!(
            try_parse_rank("RNBQKBNR"),
            Ok([
                Some(Rook(White)),
                Some(Knight(White)),
                Some(Bishop(White)),
                Some(Queen(White)),
                Some(King(White)),
                Some(Bishop(White)),
                Some(Knight(White)),
                Some(Rook(White)),
            ])
        );
        assert_eq!(
            try_parse_rank("rnbqkbnr"),
            Ok([
                Some(Rook(Black)),
                Some(Knight(Black)),
                Some(Bishop(Black)),
                Some(Queen(Black)),
                Some(King(Black)),
                Some(Bishop(Black)),
                Some(Knight(Black)),
                Some(Rook(Black)),
            ])
        );
        assert_eq!(
            try_parse_rank("r6r"),
            Ok([
                Some(Rook(Black)),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Rook(Black)),
            ])
        );
        assert_eq!(
            try_parse_rank("8"),
            Ok([None, None, None, None, None, None, None, None,])
        );
        assert_eq!(
            try_parse_rank("7Q"),
            Ok([None, None, None, None, None, None, None, Some(Queen(White)),])
        );

        assert_eq!(
            try_parse_rank("9"),
            Err(InvalidFENString::RankEmptyOutOfBounds(9))
        );
        assert_eq!(
            try_parse_rank("11"),
            Err(InvalidFENString::RankEmptyOutOfBounds(11))
        );

        assert_eq!(try_parse_rank("8K"), Err(InvalidFENString::RankIndex(8)));
        assert_eq!(try_parse_rank("pk6K"), Err(InvalidFENString::RankIndex(8)));

        assert_eq!(
            try_parse_rank("7T"),
            Err(InvalidFENString::UnknownRankChar('T'))
        );
        assert_eq!(
            try_parse_rank("pk5T"),
            Err(InvalidFENString::UnknownRankChar('T'))
        );
        // 8 letters, none of which are valid
        // https://en.wikipedia.org/wiki/Fidchell
        assert_eq!(
            try_parse_rank("fidchell"),
            Err(InvalidFENString::UnknownRankChar('f'))
        );
    }

    fn helper<'a>(board: [[char; 8]; 8]) -> Result<FENBoard, InvalidFENString<'a>> {
        Ok(FENBoard::try_from(board).unwrap())
    }

    #[test]
    fn test_try_parse_board_starting_position() {
        assert_eq!(
            try_parse_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", None, White),
            helper([
                ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
                ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
                ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
            ])
        );
    }

    #[test]
    fn test_try_parse_board_evergreen_game() {
        assert_eq!(
            try_parse_board("1r3kr1/pbpBBp1p/1b3P2/8/8/2P2q2/P4PPP/3R2K1", None, Black),
            helper([
                [' ', 'r', ' ', ' ', ' ', 'k', 'r', ' '],
                ['p', 'b', 'p', 'B', 'B', 'p', ' ', 'p'],
                [' ', 'b', ' ', ' ', ' ', 'P', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', 'P', ' ', ' ', 'q', ' ', ' '],
                ['P', ' ', ' ', ' ', ' ', 'P', 'P', 'P'],
                [' ', ' ', ' ', 'R', ' ', ' ', 'K', ' '],
            ])
        );
    }
}
