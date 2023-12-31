use super::*;

#[test]
fn coroutine_test() {
    test! {
        program {
            #[auto] trait Send { }

            #[lang(coroutine)]
            trait Coroutine<R> {
                type Yield;
                type Return;
            }

            struct StructOne {}
            struct NotSend {}
            struct SendSameLifetime<'a, 'b, T> { val: &'a T, other: &'b T }
            impl<'a, T> Send for SendSameLifetime<'a, 'a, T> {}

            struct SendAnyLifetime<'a, 'b, T> { val: &'a u8, other: &'b u8, field: T }

            impl !Send for NotSend {}
            struct StructThree<'a> { val: &'a () }

            coroutine empty_gen<>[resume = (), yield = ()] {
                upvars []
                witnesses []
            }

            coroutine upvar_lifetime_restrict<T>[resume = (), yield = ()] {
                upvars [T; StructOne]
                witnesses exists<'a, 'b> [SendSameLifetime<'a, 'b, T>]
            }

            coroutine send_any_lifetime<T>[resume = (), yield = ()] {
                upvars []
                witnesses exists<'a, 'b> [SendAnyLifetime<'a, 'b, T>; u8]
            }

            coroutine not_send_resume_yield<>[resume = NotSend, yield = NotSend] {
                upvars []
                witnesses []
            }

            coroutine gen_with_types<U>[resume = U, yield = StructOne] -> NotSend {
                upvars []
                witnesses []
            }
        }

        goal {
            WellFormed(empty_gen)
        } yields {
            expect![["Unique"]]
        }

        goal {
            empty_gen: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            empty_gen: Coroutine<()>
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                gen_with_types<T>: Coroutine<T>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                Normalize(<gen_with_types<T> as Coroutine<T>>::Yield -> StructOne)
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                Normalize(<gen_with_types<T> as Coroutine<T>>::Return -> NotSend)
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                upvar_lifetime_restrict<T>: Send
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> {
                if (T: Send) {
                    upvar_lifetime_restrict<T>: Send
                }
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0: '!2_1 }, InEnvironment { environment: Env([]), goal: '!2_1: '!2_0 }]"]]
        }

        goal {
            not_send_resume_yield: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Send) {
                    send_any_lifetime<T>: Send
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                send_any_lifetime<T>: Send
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}
