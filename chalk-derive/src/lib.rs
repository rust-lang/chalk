extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::{parse_quote, DeriveInput, Ident, TypeParam, TypeParamBound};

use synstructure::decl_derive;

/// Checks whether a generic parameter has a `: HasInterner` bound
fn has_interner(param: &TypeParam) -> Option<&Ident> {
    bounded_by_trait(param, "HasInterner")
}

/// Checks whether a generic parameter has a `: Interner` bound
fn is_interner(param: &TypeParam) -> Option<&Ident> {
    bounded_by_trait(param, "Interner")
}

fn has_interner_attr(input: &DeriveInput) -> Option<TokenStream> {
    Some(
        input
            .attrs
            .iter()
            .find(|a| a.path().is_ident("has_interner"))?
            .parse_args::<TokenStream>()
            .expect("Expected has_interner argument"),
    )
}

fn bounded_by_trait<'p>(param: &'p TypeParam, name: &str) -> Option<&'p Ident> {
    let name = Some(String::from(name));
    param.bounds.iter().find_map(|b| {
        if let TypeParamBound::Trait(trait_bound) = b {
            if trait_bound
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                == name
            {
                return Some(&param.ident);
            }
        }
        None
    })
}

fn get_intern_param(input: &DeriveInput) -> Option<(DeriveKind, &Ident)> {
    let mut params = input.generics.type_params().filter_map(|param| {
        has_interner(param)
            .map(|ident| (DeriveKind::FromHasInterner, ident))
            .or_else(|| is_interner(param).map(|ident| (DeriveKind::FromInterner, ident)))
    });

    let param = params.next();
    assert!(params.next().is_none(), "deriving this trait only works with at most one type parameter that implements HasInterner or Interner");

    param
}

fn get_intern_param_name(input: &DeriveInput) -> &Ident {
    get_intern_param(input)
        .expect("deriving this trait requires a parameter that implements HasInterner or Interner")
        .1
}

fn try_find_interner(s: &mut synstructure::Structure) -> Option<(TokenStream, DeriveKind)> {
    let input = s.ast();

    if let Some(arg) = has_interner_attr(input) {
        // Hardcoded interner:
        //
        // #[has_interner(ChalkIr)]
        // struct S {
        //
        // }
        return Some((arg, DeriveKind::FromHasInternerAttr));
    }

    get_intern_param(input).map(|generic_param0| match generic_param0 {
        (DeriveKind::FromHasInterner, param) => {
            // HasInterner bound:
            //
            // Example:
            //
            // struct Binders<T: HasInterner> { }
            s.add_impl_generic(parse_quote! { _I });

            s.add_where_predicate(parse_quote! { _I: ::chalk_ir::interner::Interner });
            s.add_where_predicate(
                parse_quote! { #param: ::chalk_ir::interner::HasInterner<Interner = _I> },
            );

            (quote! { _I }, DeriveKind::FromHasInterner)
        }
        (DeriveKind::FromInterner, i) => {
            // Interner bound:
            //
            // Example:
            //
            // struct Foo<I: Interner> { }
            (quote! { #i }, DeriveKind::FromInterner)
        }
        _ => unreachable!(),
    })
}

fn find_interner(s: &mut synstructure::Structure) -> (TokenStream, DeriveKind) {
    try_find_interner(s)
        .expect("deriving this trait requires a `#[has_interner]` attr or a parameter that implements HasInterner or Interner")
}

#[derive(Copy, Clone, PartialEq)]
enum DeriveKind {
    FromHasInternerAttr,
    FromHasInterner,
    FromInterner,
}

decl_derive!([FallibleTypeFolder, attributes(has_interner)] => derive_fallible_type_folder);
decl_derive!([HasInterner, attributes(has_interner)] => derive_has_interner);
decl_derive!([TypeVisitable, attributes(has_interner)] => derive_type_visitable);
decl_derive!([TypeSuperVisitable, attributes(has_interner)] => derive_type_super_visitable);
decl_derive!([TypeFoldable, attributes(has_interner)] => derive_type_foldable);
decl_derive!([Zip, attributes(has_interner)] => derive_zip);

fn derive_has_interner(mut s: synstructure::Structure) -> TokenStream {
    s.underscore_const(true);
    let (interner, _) = find_interner(&mut s);

    s.add_bounds(synstructure::AddBounds::None);
    s.bound_impl(
        quote!(::chalk_ir::interner::HasInterner),
        quote! {
            type Interner = #interner;
        },
    )
}

/// Derives TypeVisitable for structs and enums for which one of the following is true:
/// - It has a `#[has_interner(TheInterner)]` attribute
/// - There is a single parameter `T: HasInterner` (does not have to be named `T`)
/// - There is a single parameter `I: Interner` (does not have to be named `I`)
fn derive_type_visitable(s: synstructure::Structure) -> TokenStream {
    derive_any_type_visitable(
        s,
        parse_quote! { TypeVisitable },
        parse_quote! { visit_with },
    )
}

/// Same as TypeVisitable, but derives TypeSuperVisitable instead
fn derive_type_super_visitable(s: synstructure::Structure) -> TokenStream {
    derive_any_type_visitable(
        s,
        parse_quote! { TypeSuperVisitable },
        parse_quote! { super_visit_with },
    )
}

fn derive_any_type_visitable(
    mut s: synstructure::Structure,
    trait_name: Ident,
    method_name: Ident,
) -> TokenStream {
    s.underscore_const(true);
    let input = s.ast();
    let (interner, kind) = find_interner(&mut s);

    let body = s.each(|bi| {
        quote! {
            ::chalk_ir::try_break!(::chalk_ir::visit::TypeVisitable::visit_with(#bi, visitor, outer_binder));
        }
    });

    if kind == DeriveKind::FromHasInterner {
        let param = get_intern_param_name(input);
        s.add_where_predicate(parse_quote! { #param: ::chalk_ir::visit::TypeVisitable<#interner> });
    }

    s.add_bounds(synstructure::AddBounds::None);
    s.bound_impl(
        quote!(::chalk_ir::visit:: #trait_name <#interner>),
        quote! {
            fn #method_name <B>(
                &self,
                visitor: &mut dyn ::chalk_ir::visit::TypeVisitor < #interner, BreakTy = B >,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> std::ops::ControlFlow<B> {
                match *self {
                    #body
                }
                std::ops::ControlFlow::Continue(())
            }
        },
    )
}

fn each_variant_pair<F, R>(
    a: &mut synstructure::Structure,
    b: &mut synstructure::Structure,
    mut f: F,
) -> TokenStream
where
    F: FnMut(&synstructure::VariantInfo<'_>, &synstructure::VariantInfo<'_>) -> R,
    R: ToTokens,
{
    let mut t = TokenStream::new();
    for (v_a, v_b) in a.variants_mut().iter_mut().zip(b.variants_mut().iter_mut()) {
        v_a.binding_name(|_, i| Ident::new(&format!("a_{}", i), Span::call_site()));
        v_b.binding_name(|_, i| Ident::new(&format!("b_{}", i), Span::call_site()));

        let pat_a = v_a.pat();
        let pat_b = v_b.pat();
        let body = f(v_a, v_b);

        quote!((#pat_a, #pat_b)  => {#body}).to_tokens(&mut t);
    }
    t
}

fn derive_zip(mut s: synstructure::Structure) -> TokenStream {
    s.underscore_const(true);
    let (interner, _) = find_interner(&mut s);

    let mut a = s.clone();
    let mut b = s.clone();

    let mut body = each_variant_pair(&mut a, &mut b, |v_a, v_b| {
        let mut t = TokenStream::new();
        for (b_a, b_b) in v_a.bindings().iter().zip(v_b.bindings().iter()) {
            quote!(chalk_ir::zip::Zip::zip_with(zipper, variance, #b_a, #b_b)?;).to_tokens(&mut t);
        }
        quote!(Ok(())).to_tokens(&mut t);
        t
    });

    // when the two variants are different
    quote!((_, _)  => Err(::chalk_ir::NoSolution)).to_tokens(&mut body);

    s.add_bounds(synstructure::AddBounds::None);
    s.bound_impl(
        quote!(::chalk_ir::zip::Zip<#interner>),
        quote! {

            fn zip_with<Z: ::chalk_ir::zip::Zipper<#interner>>(
                zipper: &mut Z,
                variance: ::chalk_ir::Variance,
                a: &Self,
                b: &Self,
            ) -> ::chalk_ir::Fallible<()> {
                    match (a, b) { #body }
                }
        },
    )
}

/// Derives TypeFoldable for structs and enums for which one of the following is true:
/// - It has a `#[has_interner(TheInterner)]` attribute
/// - There is a single parameter `T: HasInterner` (does not have to be named `T`)
/// - There is a single parameter `I: Interner` (does not have to be named `I`)
fn derive_type_foldable(mut s: synstructure::Structure) -> TokenStream {
    s.underscore_const(true);
    s.bind_with(|_| synstructure::BindStyle::Move);

    let (interner, kind) = find_interner(&mut s);

    let body = s.each_variant(|vi| {
        let bindings = vi.bindings();
        vi.construct(|_, index| {
            let bind = &bindings[index];
            quote! {
                ::chalk_ir::fold::TypeFoldable::try_fold_with(#bind, folder, outer_binder)?
            }
        })
    });

    let input = s.ast();

    if kind == DeriveKind::FromHasInterner {
        let param = get_intern_param_name(input);
        s.add_where_predicate(parse_quote! { #param: ::chalk_ir::fold::TypeFoldable<#interner> });
    };

    s.add_bounds(synstructure::AddBounds::None);
    s.bound_impl(
        quote!(::chalk_ir::fold::TypeFoldable<#interner>),
        quote! {
            fn try_fold_with<E>(
                self,
                folder: &mut dyn ::chalk_ir::fold::FallibleTypeFolder < #interner, Error = E >,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::std::result::Result<Self, E> {
                Ok(match self { #body })
            }
        },
    )
}

fn derive_fallible_type_folder(mut s: synstructure::Structure) -> TokenStream {
    let interner = try_find_interner(&mut s).map_or_else(
        || {
            s.add_impl_generic(parse_quote! { _I });
            s.add_where_predicate(parse_quote! { _I: ::chalk_ir::interner::Interner });
            quote! { _I }
        },
        |(interner, _)| interner,
    );
    s.underscore_const(true);
    s.unbound_impl(
        quote!(::chalk_ir::fold::FallibleTypeFolder<#interner>),
        quote! {
            type Error = ::core::convert::Infallible;

            fn as_dyn(&mut self) -> &mut dyn ::chalk_ir::fold::FallibleTypeFolder<#interner, Error = Self::Error> {
                self
            }

            fn try_fold_ty(
                &mut self,
                ty: ::chalk_ir::Ty<#interner>,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Ty<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_ty(self, ty, outer_binder))
            }

            fn try_fold_lifetime(
                &mut self,
                lifetime: ::chalk_ir::Lifetime<#interner>,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Lifetime<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_lifetime(self, lifetime, outer_binder))
            }

            fn try_fold_const(
                &mut self,
                constant: ::chalk_ir::Const<#interner>,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Const<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_const(self, constant, outer_binder))
            }

            fn try_fold_program_clause(
                &mut self,
                clause: ::chalk_ir::ProgramClause<#interner>,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::ProgramClause<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_program_clause(self, clause, outer_binder))
            }

            fn try_fold_goal(
                &mut self,
                goal: ::chalk_ir::Goal<#interner>,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Goal<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_goal(self, goal, outer_binder))
            }

            fn forbid_free_vars(&self) -> bool {
                ::chalk_ir::fold::TypeFolder::forbid_free_vars(self)
            }

            fn try_fold_free_var_ty(
                &mut self,
                bound_var: ::chalk_ir::BoundVar,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Ty<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_var_ty(self, bound_var, outer_binder))
            }

            fn try_fold_free_var_lifetime(
                &mut self,
                bound_var: ::chalk_ir::BoundVar,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Lifetime<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_var_lifetime(self, bound_var, outer_binder))
            }

            fn try_fold_free_var_const(
                &mut self,
                ty: ::chalk_ir::Ty<#interner>,
                bound_var: ::chalk_ir::BoundVar,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Const<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_var_const(self, ty, bound_var, outer_binder))
            }

            fn forbid_free_placeholders(&self) -> bool {
                ::chalk_ir::fold::TypeFolder::forbid_free_placeholders(self)
            }

            fn try_fold_free_placeholder_ty(
                &mut self,
                universe: ::chalk_ir::PlaceholderIndex,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Ty<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_placeholder_ty(self, universe, outer_binder))
            }

            fn try_fold_free_placeholder_lifetime(
                &mut self,
                universe: ::chalk_ir::PlaceholderIndex,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Lifetime<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_placeholder_lifetime(self, universe, outer_binder))
            }

            fn try_fold_free_placeholder_const(
                &mut self,
                ty: ::chalk_ir::Ty<#interner>,
                universe: ::chalk_ir::PlaceholderIndex,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Const<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_free_placeholder_const(self, ty, universe, outer_binder))
            }

            fn forbid_inference_vars(&self) -> bool {
                ::chalk_ir::fold::TypeFolder::forbid_inference_vars(self)
            }

            fn try_fold_inference_ty(
                &mut self,
                var: ::chalk_ir::InferenceVar,
                kind: ::chalk_ir::TyVariableKind,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Ty<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_inference_ty(self, var, kind, outer_binder))
            }

            fn try_fold_inference_lifetime(
                &mut self,
                var: ::chalk_ir::InferenceVar,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Lifetime<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_inference_lifetime(self, var, outer_binder))
            }

            fn try_fold_inference_const(
                &mut self,
                ty: ::chalk_ir::Ty<#interner>,
                var: ::chalk_ir::InferenceVar,
                outer_binder: ::chalk_ir::DebruijnIndex,
            ) -> ::core::result::Result<::chalk_ir::Const<#interner>, Self::Error> {
                ::core::result::Result::Ok(::chalk_ir::fold::TypeFolder::fold_inference_const(self, ty, var, outer_binder))
            }

            fn interner(&self) -> #interner {
                ::chalk_ir::fold::TypeFolder::interner(self)
            }
        },
    )
}
