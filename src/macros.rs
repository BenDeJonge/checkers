/// Implement the needed methods to compute saturating offsets of enum variants,
/// based on their usize indices.
#[macro_export]
macro_rules! impl_enum_index_math {
    ($name:ty) => {
        impl TryFrom<usize> for $name {
            type Error = OutOfBounds;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                <$name>::iter().nth(value).ok_or(OutOfBounds)
            }
        }
        impl From<$name> for usize {
            fn from(value: $name) -> usize {
                <$name>::iter().position(|v| v == value).unwrap()
            }
        }
        impl $name {
            /// Add two variants based on index, avoiding overflows.
            pub fn saturating_add(&self, rhs: &Self) -> Self {
                let max: usize = <$name>::iter().last().unwrap().into();
                let i: usize = max.min(usize::from(*self) + usize::from(*rhs));
                Self::try_from(i).unwrap()
            }
            /// Subtract two variants based on index, avoiding underflows.
            pub fn saturating_sub(&self, rhs: &Self) -> Self {
                let i: usize = usize::from(*self).saturating_sub(usize::from(*rhs));
                Self::try_from(i).unwrap()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::square::OutOfBounds;
    use strum::{EnumIter, IntoEnumIterator};

    #[derive(Clone, Copy, EnumIter, PartialEq, Debug)]
    enum Test {
        Zero,
        One,
        Two,
        Three,
    }

    impl_enum_index_math!(Test);

    #[test]
    fn test_saturating_add_enum() {
        assert_eq!(Test::Zero.saturating_add(&Test::Zero), Test::Zero);
        assert_eq!(Test::Zero.saturating_add(&Test::One), Test::One);
        assert_eq!(Test::One.saturating_add(&Test::One), Test::Two);
        assert_eq!(Test::One.saturating_add(&Test::Two), Test::Three);
        assert_eq!(Test::One.saturating_add(&Test::Three), Test::Three);
    }

    #[test]
    fn test_saturating_sub_enum() {
        assert_eq!(Test::Three.saturating_sub(&Test::Zero), Test::Three);
        assert_eq!(Test::Three.saturating_sub(&Test::One), Test::Two);
        assert_eq!(Test::Two.saturating_sub(&Test::One), Test::One);
        assert_eq!(Test::Two.saturating_sub(&Test::Two), Test::Zero);
        assert_eq!(Test::Two.saturating_sub(&Test::Three), Test::Zero);
    }
}
