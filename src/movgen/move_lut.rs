//! It would be wasteful to compute legal moves for a given piece continuously at runtime.
//! Therefore, these moves are stored in a lookup table [`MoveLUT`] for each piece.
//! The overarching [`PieceLUT`] can be constructed once during engine initialization and continuously queried
//! with `O(1)` lookups of legal moves for a given piece on a given square.
use strum::IntoEnumIterator;

use crate::{
    movgen::{
        bitboard::{AntiDiagonal, BitBoard, ContainingSquare, Diagonal, File, Rank},
        piece::{
            Color::{Black, White},
            Piece,
        },
    },
    square::{SQUARES, Square},
};

/// A lookup table of bitboards for each square on the chess board.
/// This can be used to e.g. store the available pseudo-legal moves a piece has from every square.
/// A move is pseudo-legal if it assumes an otherwise empty board.
type BitBoardLUT = [BitBoard; 64];

struct MoveLUT(BitBoardLUT);

impl MoveLUT {
    /// Get the available pseudo-legal moves from a given start square.
    pub fn get(&self, square: &Square) -> &BitBoard {
        self.0.get(square.idx).expect("idx in [0, 63]")
    }
}

/// Intersect two iterators at a square and take their union.
///
/// E.g., the square e3 intersects at the e-file and the 3rd rank.
///
/// ```text
/// 8 . . . . | . . .
/// 7 . . . . | . . .
/// 6 . . . . | . . .
/// 5 . . . . | . . .
/// 4 . . . . | . . .
/// 3 --------+------
/// 2 . . . . | . . .
/// 1 . . . . | . . .
///   a b c d e f g h
/// ```
fn union_at_intersection<T, U>(
    square: &Square,
    mut iter1: impl ContainingSquare<T>,
    mut iter2: impl ContainingSquare<U>,
) -> u64
where
    T: Copy + Into<u64>,
    U: Copy + Into<u64>,
{
    iter1.find_containing_square(square.board).unwrap()
        | iter2.find_containing_square(square.board).unwrap()
}

fn generate_bishop_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        boards[square.idx] = BitBoard::new(
            union_at_intersection(square, Diagonal::iter(), AntiDiagonal::iter()) ^ square.board,
        );
    }
    boards
}

fn generate_rook_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        boards[square.idx] =
            BitBoard::new(union_at_intersection(square, File::iter(), Rank::iter()) ^ square.board);
    }
    boards
}

fn generate_queen_lut() -> BitBoardLUT {
    let mut rook_lut = generate_rook_lut();
    rook_lut
        .iter_mut()
        .zip(generate_bishop_lut())
        .for_each(|(rook, bishop)| {
            *rook |= bishop;
        });
    rook_lut
}

/// The knight can move to the opposite squares of the queen in a 5x5 grid.
///
/// ```text
/// x . x . x
/// . x x x .
/// x x x x x
/// . x x x .
/// x . x . x
/// ```
fn generate_knight_lut() -> BitBoardLUT {
    let mut queen = generate_queen_lut();
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let mut ranks: u64 = 0;
        for added in [1, 2] {
            ranks |= u64::from(rank.saturating_add(added));
            ranks |= u64::from(rank.saturating_sub(added));
        }

        let file = File::try_from(square.file).unwrap();
        let mut files: u64 = 0;
        for added in [1, 2] {
            files |= u64::from(file.saturating_add(added));
            files |= u64::from(file.saturating_sub(added));
        }

        queen[square.idx] = BitBoard::new(ranks & files & !square.board) & !queen[square.idx];
    }
    queen
}

fn generate_king_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let ranks =
            u64::from(rank) | u64::from(rank.saturating_add(1)) | u64::from(rank.saturating_sub(1));
        let file = File::try_from(square.file).unwrap();
        let files =
            u64::from(file) | u64::from(file.saturating_add(1)) | u64::from(file.saturating_sub(1));
        boards[square.idx] = BitBoard::new(ranks & files & !square.board);
    }
    boards
}

fn generate_pawn_white_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let file = u64::from(File::try_from(square.file).unwrap());
        let ranks = ((rank >= Rank::Two) as u64 * u64::MAX) & u64::from(rank.saturating_add(1))
            | ((rank == Rank::Two) as u64 * u64::MAX) & u64::from(rank.saturating_add(2));
        boards[square.idx] = BitBoard::new(ranks & file & !square.board);
    }
    boards
}

fn generate_pawn_black_lut() -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let file = u64::from(File::try_from(square.file).unwrap());
        let ranks = ((rank <= Rank::Seven) as u64 * u64::MAX) & u64::from(rank.saturating_sub(1))
            | ((rank == Rank::Seven) as u64 * u64::MAX) & u64::from(rank.saturating_sub(2));
        boards[square.idx] = BitBoard::new(ranks & file & !square.board);
    }
    boards
}

struct PieceLUT {
    bishop: MoveLUT,
    knight: MoveLUT,
    rook: MoveLUT,
    queen: MoveLUT,
    king: MoveLUT,
    pawn_white: MoveLUT,
    pawn_black: MoveLUT,
}

impl PieceLUT {
    pub fn new() -> Self {
        Self {
            bishop: MoveLUT(generate_bishop_lut()),
            knight: MoveLUT(generate_king_lut()),
            rook: MoveLUT(generate_rook_lut()),
            queen: MoveLUT(generate_queen_lut()),
            king: MoveLUT(generate_king_lut()),
            pawn_white: MoveLUT(generate_pawn_white_lut()),
            pawn_black: MoveLUT(generate_pawn_black_lut()),
        }
    }

    pub fn get(&self, piece: &Piece, square: &Square) -> &BitBoard {
        match *piece {
            Piece::Bishop(_) => self.bishop.get(square),
            Piece::Knight(_) => self.knight.get(square),
            Piece::Rook(_) => self.rook.get(square),
            Piece::Queen(_) => self.queen.get(square),
            Piece::King(_) => self.king.get(square),
            // The pawns are distinguished by color as their move bitboards are unique i.e., they move in opposite directions.
            Piece::Pawn(White) => self.pawn_white.get(square),
            Piece::Pawn(Black) => self.pawn_black.get(square),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::movgen::{
        bitboard::BitBoard,
        move_lut::{
            BitBoardLUT, generate_bishop_lut, generate_king_lut, generate_knight_lut,
            generate_pawn_black_lut, generate_pawn_white_lut, generate_queen_lut,
            generate_rook_lut,
        },
    };

    #[repr(usize)]
    #[derive(Debug, Clone, Copy)]
    enum TestSquare {
        A1 = 0,
        H1 = 7,
        C2 = 10,
        D4 = 27,
        E4 = 28,
        F7 = 53,
        A8 = 56,
        H8 = 63,
    }

    #[test]
    fn test_bishop_lut() {
        let lut = generate_bishop_lut();
        helper(
            lut,
            TestSquare::A1,
            ".......x\n\
                      ......x.\n\
                      .....x..\n\
                      ....x...\n\
                      ...x....\n\
                      ..x.....\n\
                      .x......\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "x.......\n\
                      .x......\n\
                      ..x.....\n\
                      ...x....\n\
                      ....x...\n\
                      .....x..\n\
                      ......x.\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::A8,
            "........\n\
                      .x......\n\
                      ..x.....\n\
                      ...x....\n\
                      ....x...\n\
                      .....x..\n\
                      ......x.\n\
                      .......x\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "........\n\
                      ......x.\n\
                      .....x..\n\
                      ....x...\n\
                      ...x....\n\
                      ..x.....\n\
                      .x......\n\
                      x.......\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "........\n\
                      .......x\n\
                      ......x.\n\
                      .....x..\n\
                      x...x...\n\
                      .x.x....\n\
                      ........\n\
                      .x.x....\n",
        );
        helper(
            lut,
            TestSquare::F7,
            "....x.x.\n\
                      ........\n\
                      ....x.x.\n\
                      ...x...x\n\
                      ..x.....\n\
                      .x......\n\
                      x.......\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::D4,
            ".......x\n\
                      x.....x.\n\
                      .x...x..\n\
                      ..x.x...\n\
                      ........\n\
                      ..x.x...\n\
                      .x...x..\n\
                      x.....x.\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "x.......\n\
                      .x.....x\n\
                      ..x...x.\n\
                      ...x.x..\n\
                      ........\n\
                      ...x.x..\n\
                      ..x...x.\n\
                      .x.....x\n",
        );
    }

    #[test]
    fn test_rook_lut() {
        let lut = generate_rook_lut();
        helper(
            lut,
            TestSquare::A1,
            "x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      .xxxxxxx\n",
        );
        helper(
            lut,
            TestSquare::H1,
            ".......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      xxxxxxx.\n",
        );
        helper(
            lut,
            TestSquare::A8,
            ".xxxxxxx\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n\
                      x.......\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "xxxxxxx.\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n\
                      .......x\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "..x.....\n\
                      ..x.....\n\
                      ..x.....\n\
                      ..x.....\n\
                      ..x.....\n\
                      ..x.....\n\
                      xx.xxxxx\n\
                      ..x.....\n",
        );
        helper(
            lut,
            TestSquare::F7,
            ".....x..\n\
                      xxxxx.xx\n\
                      .....x..\n\
                      .....x..\n\
                      .....x..\n\
                      .....x..\n\
                      .....x..\n\
                      .....x..\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "...x....\n\
                      ...x....\n\
                      ...x....\n\
                      ...x....\n\
                      xxx.xxxx\n\
                      ...x....\n\
                      ...x....\n\
                      ...x....\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "....x...\n\
                      ....x...\n\
                      ....x...\n\
                      ....x...\n\
                      xxxx.xxx\n\
                      ....x...\n\
                      ....x...\n\
                      ....x...\n",
        );
    }

    #[test]
    fn test_queen_lut() {
        let lut = generate_queen_lut();
        helper(
            lut,
            TestSquare::A1,
            "x......x\n\
                      x.....x.\n\
                      x....x..\n\
                      x...x...\n\
                      x..x....\n\
                      x.x.....\n\
                      xx......\n\
                      .xxxxxxx\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "x......x\n\
                      .x.....x\n\
                      ..x....x\n\
                      ...x...x\n\
                      ....x..x\n\
                      .....x.x\n\
                      ......xx\n\
                      xxxxxxx.\n",
        );
        helper(
            lut,
            TestSquare::A8,
            ".xxxxxxx\n\
                      xx......\n\
                      x.x.....\n\
                      x..x....\n\
                      x...x...\n\
                      x....x..\n\
                      x.....x.\n\
                      x......x\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "xxxxxxx.\n\
                      ......xx\n\
                      .....x.x\n\
                      ....x..x\n\
                      ...x...x\n\
                      ..x....x\n\
                      .x.....x\n\
                      x......x\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "..x.....\n\
                      ..x....x\n\
                      ..x...x.\n\
                      ..x..x..\n\
                      x.x.x...\n\
                      .xxx....\n\
                      xx.xxxxx\n\
                      .xxx....\n",
        );
        helper(
            lut,
            TestSquare::F7,
            "....xxx.\n\
                      xxxxx.xx\n\
                      ....xxx.\n\
                      ...x.x.x\n\
                      ..x..x..\n\
                      .x...x..\n\
                      x....x..\n\
                      .....x..\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "...x...x\n\
                      x..x..x.\n\
                      .x.x.x..\n\
                      ..xxx...\n\
                      xxx.xxxx\n\
                      ..xxx...\n\
                      .x.x.x..\n\
                      x..x..x.\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "x...x...\n\
                      .x..x..x\n\
                      ..x.x.x.\n\
                      ...xxx..\n\
                      xxxx.xxx\n\
                      ...xxx..\n\
                      ..x.x.x.\n\
                      .x..x..x\n",
        );
    }

    #[test]
    fn test_knight_lut() {
        let lut = generate_knight_lut();
        helper(
            lut,
            TestSquare::A1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      .x......\n\
                      ..x.....\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ......x.\n\
                      .....x..\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::A8,
            "........\n\
                      ..x.....\n\
                      .x......\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "........\n\
                      .....x..\n\
                      ......x.\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      .x.x....\n\
                      x...x...\n\
                      ........\n\
                      x...x...\n",
        );
        helper(
            lut,
            TestSquare::F7,
            "...x...x\n\
                      ........\n\
                      ...x...x\n\
                      ....x.x.\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "........\n\
                      ........\n\
                      ..x.x...\n\
                      .x...x..\n\
                      ........\n\
                      .x...x..\n\
                      ..x.x...\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "........\n\
                      ........\n\
                      ...x.x..\n\
                      ..x...x.\n\
                      ........\n\
                      ..x...x.\n\
                      ...x.x..\n\
                      ........\n",
        );
    }

    #[test]
    fn test_king_lut() {
        let lut = generate_king_lut();
        helper(
            lut,
            TestSquare::A1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      xx......\n\
                      .x......\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ......xx\n\
                      ......x.\n",
        );
        helper(
            lut,
            TestSquare::A8,
            ".x......\n\
                      xx......\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "......x.\n\
                      ......xx\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      .xxx....\n\
                      .x.x....\n\
                      .xxx....\n",
        );
        helper(
            lut,
            TestSquare::F7,
            "....xxx.\n\
                      ....x.x.\n\
                      ....xxx.\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "........\n\
                      ........\n\
                      ........\n\
                      ..xxx...\n\
                      ..x.x...\n\
                      ..xxx...\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "........\n\
                      ........\n\
                      ........\n\
                      ...xxx..\n\
                      ...x.x..\n\
                      ...xxx..\n\
                      ........\n\
                      ........\n",
        );
    }

    #[test]
    fn test_pawn_white_lut() {
        let lut = generate_pawn_white_lut();
        helper(
            lut,
            TestSquare::A1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::A8,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ..x.....\n\
                      ..x.....\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::F7,
            ".....x..\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "........\n\
                      ........\n\
                      ........\n\
                      ...x....\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "........\n\
                      ........\n\
                      ........\n\
                      ....x...\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
    }

    #[test]
    fn test_pawn_black_lut() {
        let lut = generate_pawn_black_lut();
        helper(
            lut,
            TestSquare::A1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H1,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::A8,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::H8,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::C2,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ..x.....\n",
        );
        helper(
            lut,
            TestSquare::F7,
            "........\n\
                      ........\n\
                      .....x..\n\
                      .....x..\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::D4,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ...x....\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            TestSquare::E4,
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ....x...\n\
                      ........\n\
                      ........\n",
        );
    }

    fn helper(lut: BitBoardLUT, square: TestSquare, squares: &str) {
        let idx = square as usize;
        let actual = format!("{}", lut[idx]);
        assert_eq!(actual, squares, "{:?}", square);
    }
}
