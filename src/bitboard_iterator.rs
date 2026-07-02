use std::ops::Shl;

pub(crate) struct MaskedBitBoardIterator {
    board: u64,
    left: u64,
    right: u64,
}

impl MaskedBitBoardIterator {
    pub fn new(board: u64, mask: u64) -> Self {
        Self {
            board,
            left: mask.trailing_zeros().into(),
            right: mask.checked_ilog2().unwrap_or_default().into(),
        }
    }
}

impl Iterator for MaskedBitBoardIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.left <= self.right {
            let result = self.board & 1u64.shl(self.left) != 0;
            self.left += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub(crate) struct BitBoardIterator(MaskedBitBoardIterator);

impl BitBoardIterator {
    pub fn new(board: u64) -> Self {
        Self(MaskedBitBoardIterator::new(board, u64::MAX))
    }
}

impl Iterator for BitBoardIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, iter, ops::Shl};

    use crate::bitboard_iterator::{BitBoardIterator, MaskedBitBoardIterator};

    /// Place a single 1 in every position on the board.
    #[test]
    fn test_bitboard_iter() {
        for i in 0..64 {
            let actual = BitBoardIterator::new(1 << i);
            let expected = iter::repeat_n(false, i)
                .chain(iter::once(true))
                .chain(iter::repeat_n(false, 64 - i - 1));
            test_iter_helper(actual, expected);
        }
    }

    /// Mask a group of 8 1s in a board that is all 1s.
    #[test]
    fn test_masked_bitboard_iter() {
        for i in 0..(64 - 8) {
            let actual = MaskedBitBoardIterator::new(u64::MAX, 255u64.shl(i));
            let expected = iter::repeat_n(true, 8);
            test_iter_helper(actual, expected);
        }
    }

    fn test_iter_helper<T: Eq + Debug>(it1: impl Iterator<Item = T>, it2: impl Iterator<Item = T>) {
        assert_eq!(it1.collect::<Vec<T>>(), it2.collect::<Vec<T>>())
    }
}
