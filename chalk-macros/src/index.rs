#[macro_export]
macro_rules! index_struct {
    ($v:vis struct $n:ident {
        $vf:vis value: usize,
    }) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $v struct $n {
            $vf value: usize,
        }

        impl $n {
            // Not all index structs need this, so allow it to be dead
            // code.
            #[allow(dead_code)]
            $v fn get_and_increment(&mut self) -> Self {
                let old_value = *self;
                self.value += 1;
                old_value
            }

            #[allow(dead_code)]
            $v fn increment(&mut self) {
                self.value += 1;
            }
        }

        impl ::std::fmt::Debug for $n {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(fmt, "{}({})", stringify!($n), self.value)
            }
        }

        impl ::std::iter::Step for $n {
            fn steps_between(start: &Self, end: &Self) -> Option<usize> {
                usize::steps_between(&start.value, &end.value)
            }

            fn replace_one(&mut self) -> Self {
                Self {
                    value: usize::replace_one(&mut self.value),
                }
            }

            fn replace_zero(&mut self) -> Self {
                Self {
                    value: usize::replace_zero(&mut self.value),
                }
            }

            fn add_one(&self) -> Self {
                Self {
                    value: usize::add_one(&self.value),
                }
            }

            fn sub_one(&self) -> Self {
                Self {
                    value: usize::sub_one(&self.value),
                }
            }

            fn add_usize(&self, n: usize) -> Option<Self> {
                usize::add_usize(&self.value, n).map(|value| Self { value })
            }
        }

        impl From<usize> for $n {
            fn from(value: usize) -> Self {
                Self { value: value }
            }
        }
    }
}
