#[derive(Copy, Clone)]
pub struct Square {
    pub(crate) idx: usize,
    pub(crate) rank: usize,
    pub(crate) file: usize,
    pub(crate) board: u64,
}

impl Square {
    const fn new(i: usize) -> Self {
        Square {
            idx: i,
            rank: i / 8,
            file: i % 8,
            board: 1u64 << i,
        }
    }
}

#[derive(Debug)]
pub struct OutOfBounds;
impl TryFrom<usize> for Square {
    type Error = OutOfBounds;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 63 {
            Err(OutOfBounds)
        } else {
            Ok(Self::new(value))
        }
    }
}

pub static SQUARES: [Square; 64] = {
    let mut squares = [Square {
        idx: 0,
        rank: 0,
        file: 0,
        board: 1,
    }; 64];
    let mut i = 0;
    while i < 64 {
        squares[i] = Square::new(i);
        i += 1;
    }
    squares
};
