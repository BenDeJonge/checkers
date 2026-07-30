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
            pub fn saturating_add(&self, rhs: usize) -> Self {
                let max: usize = <$name>::iter().last().unwrap().into();
                let i: usize = max.min(usize::from(*self) + rhs);
                Self::try_from(i).unwrap()
            }
            /// Subtract two variants based on index, avoiding underflows.
            pub fn saturating_sub(&self, rhs: usize) -> Self {
                let i: usize = usize::from(*self).saturating_sub(usize::from(rhs));
                Self::try_from(i).unwrap()
            }
        }
    };
}

#[doc(hidden)]
pub const fn __compare_str_as_bytes(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let bytes_a = a.as_bytes();
    let bytes_b = b.as_bytes();

    let mut i = 0;
    while i < bytes_a.len() {
        if bytes_a[i] != bytes_b[i] {
            return false;
        }
        i += 1;
    }

    true
}

/// Create a `const fn` that matches `&str` to a `usize` index.
#[macro_export]
macro_rules! make_str_lut {
    ($fn_name:ident, {
        $($name:literal => $idx:expr),* $(,)?
    }) => {
        pub const fn $fn_name(s: &str) -> Option<usize> {
            $(
                if $crate::macros::__compare_str_as_bytes(s, $name) {
                    return Some($idx);
                }
            )*
            None
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
        assert_eq!(Test::Zero.saturating_add(0), Test::Zero);
        assert_eq!(Test::Zero.saturating_add(1), Test::One);
        assert_eq!(Test::One.saturating_add(1), Test::Two);
        assert_eq!(Test::One.saturating_add(2), Test::Three);
        assert_eq!(Test::One.saturating_add(3), Test::Three);
    }

    #[test]
    fn test_saturating_sub_enum() {
        assert_eq!(Test::Three.saturating_sub(0), Test::Three);
        assert_eq!(Test::Three.saturating_sub(1), Test::Two);
        assert_eq!(Test::Two.saturating_sub(1), Test::One);
        assert_eq!(Test::Two.saturating_sub(2), Test::Zero);
        assert_eq!(Test::Two.saturating_sub(3), Test::Zero);
    }

    make_str_lut!(string_lut, {
        "zero" => 0,
        "one" => 1,
        "two" => 2,
    });

    #[test]
    fn test_string_lut() {
        assert_eq!(string_lut("zero"), Some(0));
        assert_eq!(string_lut("one"), Some(1));
        assert_eq!(string_lut("two"), Some(2));
        assert_eq!(string_lut("three"), None);
        assert_eq!(string_lut("Zero"), None);
        assert_eq!(string_lut("ZERO"), None);
    }
}
