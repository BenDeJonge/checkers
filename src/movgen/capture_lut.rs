use crate::{
    movgen::{
        bitboard::{BitBoard, File, Rank},
        move_lut::{BitBoardLUT, MoveLUT, PieceLUT},
    },
    square::SQUARES,
};

struct CaptureLUT<'a> {
    move_lut: &'a PieceLUT,
    pawn_white_lut: MoveLUT,
    pawn_black_lut: MoveLUT,
    rook: MoveLUT,
    bishop: MoveLUT,
    queen: MoveLUT,
}

fn generate_pawn_captures(
    next_rank: impl Fn(Rank) -> Rank,
    is_at_end: impl Fn(Rank) -> bool,
) -> BitBoardLUT {
    let mut boards = [BitBoard::from(0); 64];
    for square in SQUARES.iter() {
        let rank = Rank::try_from(square.rank).unwrap();
        let next = (!is_at_end(rank) as u64 * u64::MAX) & u64::from(next_rank(rank));

        let file = File::try_from(square.file).unwrap();
        let left = ((square.file > 0) as u64 * u64::MAX) & u64::from(file.saturating_sub(1));
        let right = ((square.file < 7) as u64 * u64::MAX) & u64::from(file.saturating_add(1));

        boards[square.idx] = BitBoard::new(next & (left | right));
    }
    boards
}

fn generate_pawn_white_captures() -> BitBoardLUT {
    // In the used matrix notation, white pawns move to lower ranks when moving up.
    generate_pawn_captures(|rank| rank.saturating_sub(1), |rank| rank == Rank::Eight)
}

fn generate_pawn_black_captures() -> BitBoardLUT {
    // In the used matrix notation, black pawns move to higher ranks when moving down.
    generate_pawn_captures(|rank| rank.saturating_add(1), |rank| rank == Rank::One)
}

#[cfg(test)]
mod tests {
    use crate::{
        movgen::{
            capture_lut::{generate_pawn_black_captures, generate_pawn_white_captures},
            move_lut::BitBoardLUT,
        },
        square::get_square_from_name,
    };

    fn helper(lut: BitBoardLUT, name: &str, squares: &str) {
        let square = get_square_from_name(name).unwrap();
        let actual = format!("{}", lut[square.idx]);
        assert_eq!(actual, squares, "{} {:?}", name, square);
    }

    #[test]
    fn test_white_pawn_captures() {
        let lut = generate_pawn_white_captures();
        helper(
            lut,
            "a2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      .x......\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "a3",
            "........\n\
            ........\n\
            ........\n\
            ........\n\
            .x......\n\
            ........\n\
            ........\n\
            ........\n",
        );
        helper(
            lut,
            "a7",
            ".x......\n\
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
            "a8",
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
            "h2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ......x.\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "h3",
            "........\n\
            ........\n\
            ........\n\
            ........\n\
            ......x.\n\
            ........\n\
            ........\n\
            ........\n",
        );
        helper(
            lut,
            "h7",
            "......x.\n\
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
            "h8",
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
            "b2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      x.x.....\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "b3",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      x.x.....\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "b7",
            "x.x.....\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
    }

    #[test]
    fn test_black_pawn_captures() {
        let lut = generate_pawn_black_captures();
        helper(
            lut,
            "a2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      .x......\n",
        );
        helper(
            lut,
            "a3",
            "........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            .x......\n\
            ........\n",
        );
        helper(
            lut,
            "a7",
            "........\n\
                      ........\n\
                      .x......\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "a8",
            "........\n\
                      .x......\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "h2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ......x.\n",
        );
        helper(
            lut,
            "h3",
            "........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ......x.\n\
            ........\n",
        );
        helper(
            lut,
            "h7",
            "........\n\
                      ........\n\
                      ......x.\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
        helper(
            lut,
            "h8",
            "........\n\
                      ......x.\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );

        helper(
            lut,
            "b2",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      x.x.....\n",
        );
        helper(
            lut,
            "b3",
            "........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      x.x.....\n\
                      ........\n",
        );
        helper(
            lut,
            "b7",
            "........\n\
                      ........\n\
                      x.x.....\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n\
                      ........\n",
        );
    }
}
