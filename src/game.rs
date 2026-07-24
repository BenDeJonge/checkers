use std::{fmt::Display, num::NonZero};

use crate::{
    fen::{
        FENBoard, InvalidFENString, try_get_fen_parts, try_parse_active_player, try_parse_board,
        try_parse_castling_rights, try_parse_en_passant_square, try_parse_half_move_clock,
        try_parse_move_clock,
    },
    movgen::{
        bitboard::BitBoard,
        piece::{BoardNotation, Color, Piece},
    },
    square::{SQUARES, Square},
};

const BITBOARD_DEFAULT_WHITE_KING: BitBoard = BitBoard::new(0x00000000_00000010);
const BITBOARD_DEFAULT_WHITE_QUEEN: BitBoard = BitBoard::new(0x00000000_00000008);
const BITBOARD_DEFAULT_WHITE_ROOK: BitBoard = BitBoard::new(0x00000000_00000081);
const BITBOARD_DEFAULT_WHITE_BISHOP: BitBoard = BitBoard::new(0x00000000_00000024);
const BITBOARD_DEFAULT_WHITE_KNIGHT: BitBoard = BitBoard::new(0x00000000_00000042);
const BITBOARD_DEFAULT_WHITE_PAWN: BitBoard = BitBoard::new(0x00000000_0000FF00);
const BITBOARD_DEFAULT_BLACK_KING: BitBoard = BitBoard::new(0x10000000_00000000);
const BITBOARD_DEFAULT_BLACK_QUEEN: BitBoard = BitBoard::new(0x08000000_00000000);
const BITBOARD_DEFAULT_BLACK_ROOK: BitBoard = BitBoard::new(0x81000000_00000000);
const BITBOARD_DEFAULT_BLACK_BISHOP: BitBoard = BitBoard::new(0x24000000_00000000);
const BITBOARD_DEFAULT_BLACK_KNIGHT: BitBoard = BitBoard::new(0x42000000_00000000);
const BITBOARD_DEFAULT_BLACK_PAWN: BitBoard = BitBoard::new(0x00FF0000_00000000);

#[derive(Debug, PartialEq, Eq)]
pub struct PieceState {
    king: BitBoard,
    queen: BitBoard,
    rook: BitBoard,
    bishop: BitBoard,
    knight: BitBoard,
    pawn: BitBoard,
}

impl PieceState {
    fn starting_position(color: Color) -> Self {
        match color {
            Color::White => Self {
                king: BITBOARD_DEFAULT_WHITE_KING,
                queen: BITBOARD_DEFAULT_WHITE_QUEEN,
                rook: BITBOARD_DEFAULT_WHITE_ROOK,
                bishop: BITBOARD_DEFAULT_WHITE_BISHOP,
                knight: BITBOARD_DEFAULT_WHITE_KNIGHT,
                pawn: BITBOARD_DEFAULT_WHITE_PAWN,
            },
            Color::Black => Self {
                king: BITBOARD_DEFAULT_BLACK_KING,
                queen: BITBOARD_DEFAULT_BLACK_QUEEN,
                rook: BITBOARD_DEFAULT_BLACK_ROOK,
                bishop: BITBOARD_DEFAULT_BLACK_BISHOP,
                knight: BITBOARD_DEFAULT_BLACK_KNIGHT,
                pawn: BITBOARD_DEFAULT_BLACK_PAWN,
            },
        }
    }
}

struct PieceStates {
    pub white: PieceState,
    pub black: PieceState,
}

impl From<FENBoard> for PieceStates {
    // TODO: it is very inconvenient that the fen index starts from the top left
    // whereas our index starts from the bottom left. It would be much easier if they were the same.
    fn from(value: FENBoard) -> Self {
        todo!()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct CastlingRights {
    kingside: bool,
    queenside: bool,
}

impl CastlingRights {
    pub fn new(kingside: bool, queenside: bool) -> Self {
        Self {
            kingside,
            queenside,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PlayerState {
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    pieces: PieceState,
}

impl PlayerState {
    pub fn new(
        castling_rights: CastlingRights,
        en_passant_square: Option<Square>,
        pieces: PieceState,
    ) -> Self {
        Self {
            castling_rights,
            en_passant_square,
            pieces,
        }
    }

    pub fn starting_position(color: Color) -> Self {
        Self {
            castling_rights: CastlingRights::default(),
            en_passant_square: None,
            pieces: PieceState::starting_position(color),
        }
    }

    pub fn make_move(&mut self) {
        // update relevant square in the correct piecelist
    }
    fn promote_pawn(&mut self) {
        // pops pawn, decrements n by 1 AND appends to correct piecelist, increments n by 1
    }
    fn capture(&mut self, square: Square, other: &mut PieceState) {
        // decrements n by 1 and removes from piecelist
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameState {
    to_play: Color,
    white: PlayerState,
    black: PlayerState,
    half_move_clock: usize,
    move_clock: NonZero<usize>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            to_play: Color::White,
            white: PlayerState::starting_position(Color::White),
            black: PlayerState::starting_position(Color::Black),
            half_move_clock: usize::default(),
            move_clock: NonZero::new(1).unwrap(),
        }
    }
}

impl GameState {
    pub fn new(
        to_play: Color,
        white: PlayerState,
        black: PlayerState,
        half_move_clock: usize,
        move_clock: NonZero<usize>,
    ) -> Self {
        Self {
            to_play,
            white,
            black,
            half_move_clock,
            move_clock,
        }
    }
}

impl<'a> TryFrom<&'a str> for GameState {
    type Error = InvalidFENString<'a>;
    /// Try parsing from a FEN string.
    ///
    /// ```
    /// # use checkers::game::GameState;
    /// let state = GameState::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// assert_eq!(state, Ok(GameState::default()));
    /// ```
    ///
    /// FEN strings can be invalid for many reasons, as per [`InvalidFENString`].
    ///
    /// ```
    /// # use checkers::game::GameState;
    /// // The first row (8th rank) has 9 characters.
    /// let state = GameState::try_from("rrnbqkbnr/8/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    /// assert_eq!(state, Err(InvalidFENString::InvalidRankLength(9)));
    /// ```
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let fen_parts = try_get_fen_parts(value)?;
        let to_play = try_parse_active_player(fen_parts[1])?;
        let [white_castle, black_castle] = try_parse_castling_rights(fen_parts[2])?;
        let en_passant_square = try_parse_en_passant_square(to_play, fen_parts[3])?;
        let move_clock = try_parse_move_clock(fen_parts[5])?;
        let half_move_clock = try_parse_half_move_clock(fen_parts[4], move_clock)?;
        // This is the most computational effort so do this last.
        let piece_states: PieceStates = try_parse_board(fen_parts[0])?.into();

        let [white_en_passant, black_en_passant] = match to_play {
            Color::White => [Some(en_passant_square), None],
            Color::Black => [None, Some(en_passant_square)],
        };
        let white = PlayerState::new(white_castle, white_en_passant, piece_states.white);
        let black = PlayerState::new(black_castle, black_en_passant, piece_states.black);
        Ok(GameState::new(
            to_play,
            white,
            black,
            half_move_clock,
            move_clock,
        ))
    }
}

const EMPTY_BOARD: [[char; 8]; 8] = [[' '; 8]; 8];
const BOARD_TOP_ROW: &str = "  ┌───┬───┬───┬───┬───┬───┬───┬───┐\n";
const BOARD_MIDDLE_ROW: &str = "  ├───┼───┼───┼───┼───┼───┼───┼───┤\n";
const BOARD_BOTTOM_ROW: &str =
    "  └───┴───┴───┴───┴───┴───┴───┴───┘\n    a   b   c   d   e   f   g   h";
const BOARD_N_ROWS: usize = 18;
const BOARD_SEP: char = '│';

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = EMPTY_BOARD;
        for (color, pieces) in [
            (Color::White, &self.white.pieces),
            (Color::Black, &self.black.pieces),
        ] {
            for (piece, piece_board) in [
                (Piece::King(color), &pieces.king),
                (Piece::Queen(color), &pieces.queen),
                (Piece::Rook(color), &pieces.rook),
                (Piece::Bishop(color), &pieces.bishop),
                (Piece::Knight(color), &pieces.knight),
                (Piece::Pawn(color), &pieces.pawn),
            ] {
                for idx in piece_board.iter_ones() {
                    let square = SQUARES[idx];
                    // Ranks are counted from 0 (1st rank) to 7 (8th rank).
                    // Indexing in array starts from the top-left (a8).
                    board[7 - square.rank][square.file] = piece.board_notation();
                }
            }
        }
        let mut buffer = String::with_capacity(BOARD_TOP_ROW.len() * BOARD_N_ROWS);
        buffer.push_str(BOARD_TOP_ROW);
        for (r, row) in board.iter().enumerate() {
            buffer.push_str(&format!("{} ", 8 - r));
            for (c, col) in row.iter().enumerate() {
                let square = if c != board[0].len() - 1 {
                    format!("{BOARD_SEP} {col} ")
                } else {
                    format!("{BOARD_SEP} {col} {BOARD_SEP}")
                };
                buffer.push_str(&square);
            }
            buffer.push('\n');
            let filler = if r != board.len() - 1 {
                BOARD_MIDDLE_ROW
            } else {
                BOARD_BOTTOM_ROW
            };
            buffer.push_str(filler);
        }
        write!(f, "{}", buffer)
    }
}

#[cfg(test)]
mod tests_display {
    use std::num::NonZero;

    use crate::{
        game::{CastlingRights, GameState, PieceState, PlayerState},
        movgen::{bitboard::BitBoard, piece::Color},
    };
    use pretty_assertions;

    /// Test display for the opening position.
    #[test]
    fn test_default() {
        let gamestate = GameState::default();
        let expected = String::from(
            "  ┌───┬───┬───┬───┬───┬───┬───┬───┐\n\
             8 │ r │ n │ b │ q │ k │ b │ n │ r │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             7 │ p │ p │ p │ p │ p │ p │ p │ p │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             6 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             5 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             4 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             3 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             2 │ P │ P │ P │ P │ P │ P │ P │ P │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             1 │ R │ N │ B │ Q │ K │ B │ N │ R │\n  \
               └───┴───┴───┴───┴───┴───┴───┴───┘\n    \
                 a   b   c   d   e   f   g   h",
        );
        pretty_assertions::assert_eq!(format!("{}", gamestate), expected);
    }

    /// Test display for the final position of the Evergreen game after move 24.
    /// Andersen - Dufresne, Berlin, 1852.
    /// https://en.wikipedia.org/wiki/Evergreen_Game
    #[test]
    fn test_evergreen_game() {
        let white_king = BitBoard::from(0x00000000_00000040);
        let white_queen = BitBoard::empty();
        let white_rook = BitBoard::from(0x00000000_00000008);
        let white_bishop = BitBoard::from(0x00180000_00000000);
        let white_knight = BitBoard::empty();
        let white_pawn = BitBoard::from(0x00002000_0004E100);

        let black_king = BitBoard::from(0x20000000_00000000);
        let black_queen = BitBoard::from(0x00000000_00200000);
        let black_rook = BitBoard::from(0x42000000_00000000);
        let black_bishop = BitBoard::from(0x00020200_00000000);
        let black_knight = BitBoard::empty();
        let black_pawn = BitBoard::from(0x00A50000_00000000);

        let expected = String::from(
            "  ┌───┬───┬───┬───┬───┬───┬───┬───┐\n\
             8 │   │ r │   │   │   │ k │ r │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             7 │ p │ b │ p │ B │ B │ p │   │ p │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             6 │   │ b │   │   │   │ P │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             5 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             4 │   │   │   │   │   │   │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             3 │   │   │ P │   │   │ q │   │   │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             2 │ P │   │   │   │   │ P │ P │ P │\n  \
               ├───┼───┼───┼───┼───┼───┼───┼───┤\n\
             1 │   │   │   │ R │   │   │ K │   │\n  \
               └───┴───┴───┴───┴───┴───┴───┴───┘\n    \
                 a   b   c   d   e   f   g   h",
        );

        let white_pieces = PieceState {
            king: white_king,
            queen: white_queen,
            rook: white_rook,
            bishop: white_bishop,
            knight: white_knight,
            pawn: white_pawn,
        };
        let white_player = PlayerState {
            castling_rights: CastlingRights::new(false, false),
            en_passant_square: None,
            pieces: white_pieces,
        };

        let black_pieces = PieceState {
            king: black_king,
            queen: black_queen,
            rook: black_rook,
            bishop: black_bishop,
            knight: black_knight,
            pawn: black_pawn,
        };
        let black_player = PlayerState {
            castling_rights: CastlingRights::new(false, false),
            en_passant_square: None,
            pieces: black_pieces,
        };

        let gamestate = GameState {
            white: white_player,
            black: black_player,
            to_play: Color::Black,
            half_move_clock: 0,
            move_clock: NonZero::new(24).unwrap(),
        };

        pretty_assertions::assert_eq!(format!("{}", gamestate), expected);
    }
}

#[cfg(test)]
mod tests_from_fen {
    #[test]
    fn test_starting_position() {
        todo!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    #[test]
    fn test_1e4() {
        todo!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
    }

    #[test]
    fn test_1e4c5() {
        todo!("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2")
    }

    #[test]
    fn test_1e4c5_2nf3() {
        todo!("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")
    }

    #[test]
    fn test_evergreen_game() {
        todo!("1r3kr1/pbpBBp1p/1b3P2/8/8/2P2q2/P4PPP/3R2K1 b - - 0 24")
    }

    #[test]
    fn test_stalemate() {
        todo!("8/8/8/8/8/7K/5Q2/7k b - - 0 45")
    }
}
