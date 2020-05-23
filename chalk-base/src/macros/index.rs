#[macro_export]
macro_rules! index_struct {
    ($(#[$m:meta])* $v:vis struct $n:ident {
        $vf:vis value: usize,
    }) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $(#[$m])*
        $v struct $n {
            $vf value: usize,
        }

        impl $n {
            // Not all index structs need this, so allow it to be dead
            // code.
            #[allow(dead_code)]
            $v fn get_and_increment(&mut self) -> Self {
                let old_value = *self;
                self.increment();
                old_value
            }

            #[allow(dead_code)]
            $v fn increment(&mut self) {
                self.value += 1;
            }

            // TODO: Once the Step trait is stabilized (https://github.com/rust-lang/rust/issues/42168), instead implement it and use the Iterator implementation of Range
            pub fn iterate_range(range: ::std::ops::Range<Self>) -> impl Iterator<Item = $n> {
                (range.start.value..range.end.value).into_iter().map(|i| Self { value: i })
            }
        }

        impl ::std::fmt::Debug for $n {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(fmt, "{}({})", stringify!($n), self.value)
            }
        }

        impl From<usize> for $n {
            fn from(value: usize) -> Self {
                Self { value }
            }
        }
    }
}
