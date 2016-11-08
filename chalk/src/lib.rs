#![allow(dead_code)] // temporary
#![feature(conservative_impl_trait)]
#![feature(question_mark)]
#![feature(static_in_const)]

extern crate chalk_parse;
extern crate lalrpop_intern;
extern crate ena;

/// Create a deref impl. We do this a lot.
macro_rules! deref_to {
    ($source:ident<$($param:ident),*>.$field:ident => $target:ty) => {
        impl<$($param),*> ::std::ops::Deref for $source<$($param),*> {
            type Target = $target;

            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    };

    ($source:ident.$field:ident => $target:ty) => {
        impl ::std::ops::Deref for $source {
            type Target = $target;

            fn deref(&self) -> &$target {
                &self.$field
            }
        }
    };
}

#[macro_use]
mod formula;

mod infer;
mod solve;
mod subst;

