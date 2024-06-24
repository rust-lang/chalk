use super::*;

#[test]
fn scalar_in_tuple_trait_impl() {
    test! {
        program {
            trait Foo { }
            impl Foo for usize { }
            impl Foo for isize { }
            impl<T1, T2> Foo for (T1, T2) where T1: Foo, T2: Foo { }
            impl<T> Foo for (T,T,T) where T: Foo { }
        }

        goal {
            (usize, usize): Foo
        } yields {
            expect![["Unique"]]
        }

        goal {
            (usize, isize): Foo
        } yields {
            expect![["Unique"]]
        }

        goal {
            (usize, bool): Foo
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            (usize, usize, usize): Foo
        } yields {
            expect![["Unique"]]
        }

        goal {
            (char, u8, i8): Foo
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn scalar_trait_impl() {
    test! {
        program {
            trait Foo { }
            trait UnsignedFoo { }

            impl Foo for i8 { }
            impl Foo for i16 { }
            impl Foo for i32 { }
            impl Foo for i64 { }
            impl Foo for i128 { }
            impl Foo for isize { }
            impl Foo for u8 { }
            impl Foo for u16 { }
            impl Foo for u32 { }
            impl Foo for u64 { }
            impl Foo for u128 { }
            impl Foo for usize { }
            impl Foo for f16 { }
            impl Foo for f32 { }
            impl Foo for f64 { }
            impl Foo for f128 { }
            impl Foo for bool { }
            impl Foo for char { }

            impl UnsignedFoo for u8 { }
            impl UnsignedFoo for u16 { }
            impl UnsignedFoo for u32 { }
            impl UnsignedFoo for u64 { }
            impl UnsignedFoo for u128 { }
            impl UnsignedFoo for usize { }

        }

        goal { i8: Foo } yields { expect![["Unique"]] }
        goal { i16: Foo } yields { expect![["Unique"]] }
        goal { i32: Foo } yields { expect![["Unique"]] }
        goal { i64: Foo } yields { expect![["Unique"]] }
        goal { i128: Foo } yields { expect![["Unique"]] }
        goal { isize: Foo } yields { expect![["Unique"]] }
        goal { u8: Foo } yields { expect![["Unique"]] }
        goal { u16: Foo } yields { expect![["Unique"]] }
        goal { u32: Foo } yields { expect![["Unique"]] }
        goal { u64: Foo } yields { expect![["Unique"]] }
        goal { u128: Foo } yields { expect![["Unique"]] }
        goal { usize: Foo } yields { expect![["Unique"]] }
        goal { f16: Foo } yields { expect![["Unique"]] }
        goal { f32: Foo } yields { expect![["Unique"]] }
        goal { f64: Foo } yields { expect![["Unique"]] }
        goal { f128: Foo } yields { expect![["Unique"]] }
        goal { bool: Foo } yields { expect![["Unique"]] }
        goal { char: Foo } yields { expect![["Unique"]] }

        goal { i8: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { i16: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { i32: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { i64: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { i128: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { isize: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { u8: UnsignedFoo } yields { expect![["Unique"]] }
        goal { u16: UnsignedFoo } yields { expect![["Unique"]] }
        goal { u32: UnsignedFoo } yields { expect![["Unique"]] }
        goal { u64: UnsignedFoo } yields { expect![["Unique"]] }
        goal { u128: UnsignedFoo } yields { expect![["Unique"]] }
        goal { usize: UnsignedFoo } yields { expect![["Unique"]] }
        goal { f16: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { f32: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { f64: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { f128: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { bool: UnsignedFoo } yields { expect![["No possible solution"]] }
        goal { char: UnsignedFoo } yields { expect![["No possible solution"]] }

    }
}

#[test]
fn scalars_are_well_formed() {
    test! {
        goal { WellFormed(i8) } yields { expect![["Unique"]] }
        goal { WellFormed(i16) } yields { expect![["Unique"]] }
        goal { WellFormed(i32) } yields { expect![["Unique"]] }
        goal { WellFormed(i64) } yields { expect![["Unique"]] }
        goal { WellFormed(i128) } yields { expect![["Unique"]] }
        goal { WellFormed(isize) } yields { expect![["Unique"]] }
        goal { WellFormed(u8) } yields { expect![["Unique"]] }
        goal { WellFormed(u16) } yields { expect![["Unique"]] }
        goal { WellFormed(u32) } yields { expect![["Unique"]] }
        goal { WellFormed(u64) } yields { expect![["Unique"]] }
        goal { WellFormed(u128) } yields { expect![["Unique"]] }
        goal { WellFormed(usize) } yields { expect![["Unique"]] }
        goal { WellFormed(f16) } yields { expect![["Unique"]] }
        goal { WellFormed(f32) } yields { expect![["Unique"]] }
        goal { WellFormed(f64) } yields { expect![["Unique"]] }
        goal { WellFormed(f128) } yields { expect![["Unique"]] }
        goal { WellFormed(bool) } yields { expect![["Unique"]] }
        goal { WellFormed(char) } yields { expect![["Unique"]] }
    }
}

#[test]
fn scalars_are_sized() {
    test! {
        program {
            #[lang(sized)] trait Sized { }
        }

        goal { i8: Sized } yields { expect![["Unique"]] }
        goal { i16: Sized } yields { expect![["Unique"]] }
        goal { i32: Sized } yields { expect![["Unique"]] }
        goal { i64: Sized } yields { expect![["Unique"]] }
        goal { i128: Sized } yields { expect![["Unique"]] }
        goal { isize: Sized } yields { expect![["Unique"]] }
        goal { u8: Sized } yields { expect![["Unique"]] }
        goal { u16: Sized } yields { expect![["Unique"]] }
        goal { u32: Sized } yields { expect![["Unique"]] }
        goal { u64: Sized } yields { expect![["Unique"]] }
        goal { u128: Sized } yields { expect![["Unique"]] }
        goal { usize: Sized } yields { expect![["Unique"]] }
        goal { f16: Sized } yields { expect![["Unique"]] }
        goal { f32: Sized } yields { expect![["Unique"]] }
        goal { f64: Sized } yields { expect![["Unique"]] }
        goal { f128: Sized } yields { expect![["Unique"]] }
        goal { bool: Sized } yields { expect![["Unique"]] }
        goal { char: Sized } yields { expect![["Unique"]] }
    }
}
