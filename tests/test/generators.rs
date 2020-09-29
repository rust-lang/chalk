use super::*;

#[test]
fn generator_test() {
    test! {
        program {
            #[auto] trait Send { }


            struct StructOne {}
            struct NotSend {}
            struct SendSameLifetime<'a, 'b, T> { val: &'a T, other: &'b T }
            impl<'a, T> Send for SendSameLifetime<'a, 'a, T> {}

            struct SendAnyLifetime<'a, 'b, T> { val: &'a u8, other: &'b u8, field: T }

            impl !Send for NotSend {}
            struct StructThree<'a> { val: &'a () }

            generator empty_gen<>[resume = (), yield = ()] {
                upvars []
                witnesses []
            }

            generator upvar_lifetime_restrict<T>[resume = (), yield = ()] {
                upvars [T; StructOne]
                witnesses exists<'a, 'b> [SendSameLifetime<'a, 'b, T>]
            }

            generator send_any_lifetime<T>[resume = (), yield = ()] {
                upvars []
                witnesses exists<'a, 'b> [SendAnyLifetime<'a, 'b, T>; u8]
            }


            generator not_send_resume_yield<>[resume = NotSend, yield = NotSend] {
                upvars []
                witnesses []
            }

        }

        goal {
            WellFormed(empty_gen)
        } yields {
            "Unique"
        }

        goal {
            empty_gen: Send
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                upvar_lifetime_restrict<T>: Send
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<T> {
                if (T: Send) {
                    upvar_lifetime_restrict<T>: Send
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([for<> FromEnv(!1_0: Send)]), goal: '!2_0: '!2_1 }, InEnvironment { environment: Env([for<> FromEnv(!1_0: Send)]), goal: '!2_1: '!2_0 }]"
        }

        goal {
            not_send_resume_yield: Send
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<T> {
                if (T: Send) {
                    send_any_lifetime<T>: Send
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
