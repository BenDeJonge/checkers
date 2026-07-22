use std::{fmt::Display, str::Chars};

use crate::{
    movgen::{
        bitboard::BitBoard,
        piece::{
            Color::{self, Black, White},
            Piece::{self, King},
        },
    },
    square::{SQUARES, Square},
};

struct PieceBoard {
    piece: Piece,
    board: BitBoard,
}

const BITBOARD_WHITE_KING: u64 = 0x00000000_10000010; // first 1 should be 0
const BITBOARD_WHITE_QUEEN: u64 = 0x00000000_00000008;
const BITBOARD_WHITE_ROOK: u64 = 0x00000000_00000081;
const BITBOARD_WHITE_BISHOP: u64 = 0x00000000_00000024;
const BITBOARD_WHITE_KNIGHT: u64 = 0x00000000_00000042;
const BITBOARD_WHITE_PAWN: u64 = 0x00000000_0000FF00;
const BITBOARD_BLACK_KING: u64 = 0x10000000_00000000;
const BITBOARD_BLACK_QUEEN: u64 = 0x08000000_00000000;
const BITBOARD_BLACK_ROOK: u64 = 0x81000000_00000000;
const BITBOARD_BLACK_BISHOP: u64 = 0x24000000_00000000;
const BITBOARD_BLACK_KNIGHT: u64 = 0x42000000_00000000;
const BITBOARD_BLACK_PAWN: u64 = 0x00FF0000_00000000;

impl PieceBoard {
    pub fn new(piece: Piece) -> Self {
        let board = BitBoard::new(match piece {
            Piece::King(White) => BITBOARD_WHITE_KING,
            Piece::Queen(White) => BITBOARD_WHITE_QUEEN,
            Piece::Rook(White) => BITBOARD_WHITE_ROOK,
            Piece::Bishop(White) => BITBOARD_WHITE_BISHOP,
            Piece::Knight(White) => BITBOARD_WHITE_KNIGHT,
            Piece::Pawn(White) => BITBOARD_WHITE_PAWN,
            Piece::King(Black) => BITBOARD_BLACK_KING,
            Piece::Queen(Black) => BITBOARD_BLACK_QUEEN,
            Piece::Rook(Black) => BITBOARD_BLACK_ROOK,
            Piece::Bishop(Black) => BITBOARD_BLACK_BISHOP,
            Piece::Knight(Black) => BITBOARD_BLACK_KNIGHT,
            Piece::Pawn(Black) => BITBOARD_BLACK_PAWN,
        });
        Self { piece, board }
    }
}

struct PieceState {
    king: PieceBoard,
    queen: PieceBoard,
    rook: PieceBoard,
    bishop: PieceBoard,
    knight: PieceBoard,
    pawn: PieceBoard,
}

impl PieceState {
    pub fn new(color: Color) -> Self {
        Self {
            king: PieceBoard::new(Piece::King(color)),
            queen: PieceBoard::new(Piece::Queen(color)),
            rook: PieceBoard::new(Piece::Rook(color)),
            bishop: PieceBoard::new(Piece::Bishop(color)),
            knight: PieceBoard::new(Piece::Knight(color)),
            pawn: PieceBoard::new(Piece::Pawn(color)),
        }
    }
}

pub struct PlayerState {
    can_castle_queenside: bool,
    can_castle_kingside: bool,
    en_passant_square: Option<Square>,
    pieces: PieceState,
}

impl PlayerState {
    pub fn new(color: Color) -> Self {
        Self {
            can_castle_queenside: true,
            can_castle_kingside: true,
            en_passant_square: None,
            pieces: PieceState::new(color),
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

pub struct GameState {
    to_play: Color,
    white: PlayerState,
    black: PlayerState,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            to_play: White,
            white: PlayerState::new(Color::White),
            black: PlayerState::new(Color::Black),
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
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
        for pieces in [&self.white.pieces, &self.black.pieces] {
            for piece in [
                &pieces.king,
                &pieces.queen,
                &pieces.rook,
                &pieces.bishop,
                &pieces.knight,
                &pieces.pawn,
            ] {
                for idx in piece.board.iter_ones() {
                    let square = SQUARES[idx];
                    board[square.rank][square.file] = piece.piece.board_representation();
                }
            }
        }
        let mut buffer = String::with_capacity(BOARD_TOP_ROW.len() * BOARD_N_ROWS);
        buffer.push_str(BOARD_TOP_ROW);
        // TODO: there is a bug here about the iteration direction.
        // Perhaps changing the convention from 0 at a1 to 0 at a8 makes sense!
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
mod tests {
    use crate::game::GameState;

    #[test]
    fn test_display_gamestate() {
        // TODO: spacing is ok but the iteration direction might be off.
        // Black pieces are placed on white squares.
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
        assert_eq!(format!("{}", gamestate), expected);
    }
}
