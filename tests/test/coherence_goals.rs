//! Tests related to "coherence goals", which are the special goals we use to reflect
//! the coherence logic.

use super::*;

#[test]
fn local_and_upstream_types() {
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }
        }

        goal { IsLocal(Upstream) } yields { expect![["No possible solution"]] }
        goal { IsUpstream(Upstream) } yields { expect![["Unique"]] }

        goal { IsLocal(Local) } yields { expect![["Unique"]] }
        goal { IsUpstream(Local) } yields { expect![["No possible solution"]] }
    }

    test! {
        program {
            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        goal { forall<T> { IsLocal(Upstream<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Upstream<T>) } } yields { expect![["Unique"]] }

        goal { forall<T> { IsLocal(Local<T>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsUpstream(Local<T>) } } yields { expect![["No possible solution"]] }
    }
}

#[test]
fn is_fully_visible() {
    // Should be visible regardless of local, fundamental, or upstream
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }

            #[upstream]
            #[fundamental]
            struct Box<T> { }
        }

        goal { IsFullyVisible(Upstream) } yields { expect![["Unique"]] }
        goal { IsFullyVisible(Local) } yields { expect![["Unique"]] }
        goal { IsFullyVisible(Box<Local>) } yields { expect![["Unique"]] }
        goal { IsFullyVisible(Box<Upstream>) } yields { expect![["Unique"]] }
    }

    // Should be visible regardless of local, fundamental, or upstream
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }

            #[upstream] struct Upstream2<T> { }
            struct Local2<T> { }

            #[upstream]
            #[fundamental]
            struct Box<T> { }
        }

        // Unknown type parameters are not fully visible
        goal { forall<T> { IsFullyVisible(Box<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsFullyVisible(Upstream2<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsFullyVisible(Local2<T>) } } yields { expect![["No possible solution"]] }

        // Without any unknown type parameters, local and upstream should not matter
        goal { forall<T> { IsFullyVisible(Upstream2<Upstream>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Upstream2<Local>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Local2<Upstream>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Local2<Local>) } } yields { expect![["Unique"]] }

        // Fundamental anywhere should not change the outcome
        goal { forall<T> { IsFullyVisible(Box<Upstream2<Upstream>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Box<Upstream2<Local>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Box<Local2<Upstream>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Box<Local2<Local>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Upstream2<Box<Upstream>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Upstream2<Box<Local>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Local2<Box<Upstream>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsFullyVisible(Local2<Box<Local>>) } } yields { expect![["Unique"]] }
    }
}

#[test]
fn fundamental_types() {
    // NOTE: These tests need to have both Local and Upstream structs since chalk will attempt
    // to enumerate all of them.

    // This first test is a sanity check to make sure `Box` isn't a special case.
    // By testing this, we ensure that adding the #[fundamental] attribute does in fact
    // change behaviour
    test! {
        program {
            #[upstream] struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // Without fundamental, Box should behave like a regular upstream type
        goal { forall<T> { not { IsLocal(Box<T>) } } } yields { expect![["Unique"]] }
        goal { forall<T> { IsLocal(Box<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Box<T>) } } yields { expect![["Unique"]] }

        // Without fundamental, Box is upstream regardless of its inner type
        goal { IsLocal(Box<Upstream>) } yields { expect![["No possible solution"]] }
        goal { IsLocal(Box<Local>) } yields { expect![["No possible solution"]] }
        goal { IsUpstream(Box<Upstream>) } yields { expect![["Unique"]] }
        goal { IsUpstream(Box<Local>) } yields { expect![["Unique"]] }
    }

    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // With fundamental, Box can be local for certain types, so there is no unique solution
        // anymore for any of these
        goal { forall<T> { not { IsLocal(Box<T>) } } } yields { expect![["Ambiguous; no inference guidance"]] }
        goal { forall<T> { IsLocal(Box<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Box<T>) } } yields { expect![["No possible solution"]] }

        // With fundamental, some of these yield different results -- no longer depends on Box
        // itself
        goal { IsLocal(Box<Upstream>) } yields { expect![["No possible solution"]] }
        goal { IsLocal(Box<Local>) } yields { expect![["Unique"]] }
        goal { IsUpstream(Box<Upstream>) } yields { expect![["Unique"]] }
        goal { IsUpstream(Box<Local>) } yields { expect![["No possible solution"]] }
    }

    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }

            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        // Upstream is upstream no matter what, so this should not be local for any T
        goal { forall<T> { IsLocal(Box<Upstream<T>>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Box<Upstream<T>>) } } yields { expect![["Unique"]] }

        // A fundamental type inside an upstream type should not make a difference (i.e. the rules
        // for the outer, non-fundamental type should apply)
        goal { forall<T> { IsLocal(Upstream<Box<T>>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Upstream<Box<T>>) } } yields { expect![["Unique"]] }

        // Make sure internal types within an upstream type do not make a difference
        goal { forall<T> { IsLocal(Box<Upstream<Local<T>>>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Box<Upstream<Local<T>>>) } } yields { expect![["Unique"]] }

        // Local is local no matter what, so this should be local for any T
        goal { forall<T> { IsLocal(Box<Local<T>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsUpstream(Box<Local<T>>) } } yields { expect![["No possible solution"]] }

        // A fundamental type inside an internal type should not make a difference
        goal { forall<T> { IsLocal(Local<Box<T>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsUpstream(Local<Box<T>>) } } yields { expect![["No possible solution"]] }

        // Make sure upstream types within an internal type and vice versa do not make a difference
        goal { forall<T> { IsLocal(Box<Local<Upstream<T>>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { IsUpstream(Box<Upstream<Local<T>>>) } } yields { expect![["Unique"]] }
    }

    // Nested fundamental types should still be local if they can be recursively proven to be local
    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }
            // This type represents &T which is also fundamental
            #[upstream]
            #[fundamental]
            struct Ref<T> { }

            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        goal { forall<T> { IsLocal(Ref<Box<T>>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { IsUpstream(Ref<Box<T>>) } } yields { expect![["No possible solution"]] }

        goal { IsLocal(Ref<Box<Upstream2>>) } yields { expect![["No possible solution"]] }
        goal { IsUpstream(Ref<Box<Upstream2>>) } yields { expect![["Unique"]] }

        goal { IsLocal(Ref<Box<Internal2>>) } yields { expect![["Unique"]] }
        goal { IsUpstream(Ref<Box<Internal2>>) } yields { expect![["No possible solution"]] }
    }

    // If a type is not upstream, it is always local regardless of its parameters or #[fundamental]
    test! {
        program {
            // if we were compiling std, Box would never be upstream
            #[fundamental]
            struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        goal { forall<T> { IsLocal(Box<T>) } } yields { expect![["Unique"]] }
        goal { IsLocal(Box<Upstream>) } yields { expect![["Unique"]] }
        goal { IsLocal(Box<Local>) } yields { expect![["Unique"]] }
    }
}

#[test]
fn local_impl_allowed_for_traits() {
    test! {
        program {
            trait LocalTrait { }
            trait LocalTrait2<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // Local traits are always implementable
        goal { forall<T> { LocalImplAllowed(T: LocalTrait) } } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Local: LocalTrait) } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Upstream: LocalTrait) } yields { expect![["Unique"]] }
        goal { forall<T> { LocalImplAllowed(T: LocalTrait2<T>) } } yields { expect![["Unique"]] }
        goal { forall<T, U> { LocalImplAllowed(T: LocalTrait2<U>) } } yields { expect![["Unique"]] }
        goal { forall<T> { LocalImplAllowed(Local: LocalTrait2<T>) } } yields { expect![["Unique"]] }
        goal { forall<T> { LocalImplAllowed(Upstream: LocalTrait2<T>) } } yields { expect![["Unique"]] }
    }

    // Single-type parameter trait refs (Self only)
    test! {
        program {
            #[upstream] trait UpstreamTrait { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
            struct Local2<T> { }
        }

        // No local type
        goal { LocalImplAllowed(Upstream: UpstreamTrait) } yields { expect![["No possible solution"]] }
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait) } } yields { expect![["No possible solution"]] }

        // Local type, not preceded by anything
        // Notice that the types after the first local type do not matter at all
        goal { LocalImplAllowed(Local: UpstreamTrait) } yields { expect![["Unique"]] }
    }

    // Multi-type parameter trait refs (Self, T)
    test! {
        program {
            trait Clone { }
            #[upstream] trait UpstreamTrait2<T> where T: Clone { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
            struct Local2<T> { }
        }

        // No local type
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait2<T>) } } yields { expect![["No possible solution"]] }
        goal { forall<T, U> { LocalImplAllowed(T: UpstreamTrait2<U>) } } yields { expect![["No possible solution"]] }
        goal { forall<T> { LocalImplAllowed(Upstream: UpstreamTrait2<T>) } } yields { expect![["No possible solution"]] }

        // Local type, but preceded by a type parameter
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait2<Local>) } } yields { expect![["No possible solution"]] }

        // Local type, not preceded by anything
        // Notice that the types after the first local type do not matter at all
        goal { forall<T> { LocalImplAllowed(Local: UpstreamTrait2<T>) } } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Local: UpstreamTrait2<Upstream>) } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Local: UpstreamTrait2<Local>) } yields { expect![["Unique"]] }

        // Local type, but preceded by a fully visible type (i.e. no placeholder types)
        goal { LocalImplAllowed(Upstream: UpstreamTrait2<Local>) } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Upstream2<Local>: UpstreamTrait2<Local>) } yields { expect![["Unique"]] }
        goal { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait2<Local>) } yields { expect![["Unique"]] }

        // Type parameter covered by the local type
        goal { forall<T> { LocalImplAllowed(Upstream: UpstreamTrait2<Local2<T>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { LocalImplAllowed(Upstream2<Local>: UpstreamTrait2<Local2<T>>) } } yields { expect![["Unique"]] }
        goal { forall<T> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait2<Local2<T>>) } } yields { expect![["Unique"]] }

        // Type parameter covered by a deeply nested upstream type
        // Notice that it does not matter that the T is wrapped in a local type because the outer
        // type is still upstream
        goal { forall<T> { LocalImplAllowed(Upstream2<Local2<T>>: UpstreamTrait2<Local2<T>>) } } yields { expect![["No possible solution"]] }
        // Does not matter whether the covered type parameter is eventually covered or not by the
        // first actually local type found
        goal { forall<T, U> { LocalImplAllowed(Upstream2<Local2<T>>: UpstreamTrait2<Local2<U>>) } } yields { expect![["No possible solution"]] }
    }

    test! {
        program {
            trait Clone { }
            trait Eq { }
            // Lifetime is just included to show that it does not break anything.
            // Where clauses do not change the results at all.
            #[upstream] trait UpstreamTrait<'a, T, U, V> where T: Clone, U: Eq, V: Clone, V: Eq { }
            trait InternalTrait<'a, T, U, V> where T: Clone, U: Eq, V: Clone, V: Eq { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
        }

        // Local traits can be implemented regardless of the types involved
        goal { forall<Self, 'a, T, U, V> { LocalImplAllowed(Self: InternalTrait<'a, T, U, V>) } } yields { expect![["Unique"]] }

        // Upstream traits definitely cannot be implemented for all types
        goal { forall<Self, 'a, T, U, V> { LocalImplAllowed(Self: UpstreamTrait<'a, T, U, V>) } } yields { expect![["No possible solution"]] }

        // No local types
        goal { forall<'a> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait<'a, Upstream, Upstream, Upstream>) } } yields { expect![["No possible solution"]] }
        goal { forall<'a> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait<
            'a,
            Upstream2<Upstream>,
            Upstream2<Upstream2<Upstream2<Upstream>>>,
            Upstream2<Upstream2<Upstream>>
        >) } } yields { expect![["No possible solution"]] }

        // Local type, not preceded by anything -- types after the first local type do not matter
        goal { forall<'a, T, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, T, U, V>) } } yields { expect![["Unique"]] }
        goal { forall<'a, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, Local, U, V>) } } yields { expect![["Unique"]] }
        goal { forall<'a, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, Upstream, U, V>) } } yields { expect![["Unique"]] }
        goal { forall<'a> { LocalImplAllowed(Local: UpstreamTrait<'a, Upstream, Local, Local>) } } yields { expect![["Unique"]] }

        // Local type preceded by a type that is not fully visible
        goal { forall<'a, T> { LocalImplAllowed(T: UpstreamTrait<'a, Upstream, Upstream, Local>) } } yields { expect![["No possible solution"]] }
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, T, Upstream, Local>) } } yields { expect![["No possible solution"]] }
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, Upstream, T, Local>) } } yields { expect![["No possible solution"]] }

        // Once again, types after the first local do not matter
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, Upstream, Local, T>) } } yields { expect![["Unique"]] }
    }
}
