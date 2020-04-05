extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, GenericParam, Ident, TypeParamBound};

/// Derives Fold for structs and enums for which one of the following is true:
/// - It has a `#[has_interner(TheInterner)]` attribute
/// - There is a single parameter `T: HasInterner` (does not have to be named `T`)
/// - There is a single parameter `I: Interner` (does not have to be named `I`)
#[proc_macro_derive(Fold, attributes(has_interner))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause_ref) = input.generics.split_for_impl();

    let type_name = input.ident;
    let body = derive_fold_body(&type_name, input.data);

    if let Some(attr) = input.attrs.iter().find(|a| a.path.is_ident("has_interner")) {
        // Hardcoded interner:
        //
        // impl Fold<ChalkIr, ChalkIr> for Type {
        //     type Result = Self;
        // }
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_interner argument");

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #arg, #arg > for #type_name #ty_generics #where_clause_ref {
                type Result = Self;

                fn fold_with<'i>(
                    &self,
                    folder: &mut dyn Folder < 'i, #arg, #arg >,
                    outer_binder: DebruijnIndex,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                    #body
                }
            }
        });
    }

    match input.generics.params.len() {
        1 => {}

        0 => {
            panic!("Fold derive requires a single type parameter or a `#[has_interner]` attr");
        }

        _ => {
            panic!("Fold derive only works with a single type parameter");
        }
    };

    let generic_param0 = &input.generics.params[0];

    if let Some(param) = has_interner(&generic_param0) {
        // HasInterner bound:
        //
        // Example:
        //
        // impl<T, _I, _TI, _U> Fold<_I, _TI> for Binders<T>
        // where
        //     T: HasInterner<Interner = _I>,
        //     T: Fold<_I, _TI, Result = _U>,
        //     U: HasInterner<Interner = _TI>,
        // {
        //     type Result = Binders<_U>;
        // }

        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![
            GenericParam::Type(syn::parse(quote! { _I: Interner }.into()).unwrap()),
            GenericParam::Type(syn::parse(quote! { _TI: TargetInterner<_I> }.into()).unwrap()),
            GenericParam::Type(
                syn::parse(quote! { _U: HasInterner<Interner = _TI> }.into()).unwrap(),
            ),
        ]);

        let mut where_clause = where_clause_ref
            .cloned()
            .unwrap_or_else(|| syn::parse2(quote![where]).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: HasInterner<Interner = _I> }).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: Fold<_I, _TI, Result = _U> }).unwrap());

        return TokenStream::from(quote! {
            impl #impl_generics Fold < _I, _TI > for #type_name < #param >
                #where_clause
            {
                type Result = #type_name < _U >;

                fn fold_with<'i>(
                    &self,
                    folder: &mut dyn Folder < 'i, _I, _TI >,
                    outer_binder: DebruijnIndex,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result>
                where
                    _I: 'i,
                    _TI: 'i,
                {
                    #body
                }
            }
        });
    }

    // Interner bound:
    //
    // Example:
    //
    // impl<I, _TI> Fold<I, _TI> for Foo<I>
    // where
    //     I: HasInterner,
    //     _TI: HasInterner,
    // {
    //     type Result = Foo<_TI>;
    // }

    if let Some(i) = is_interner(&generic_param0) {
        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![GenericParam::Type(
            syn::parse(quote! { _TI: TargetInterner<#i> }.into()).unwrap(),
        )]);

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #i, _TI > for #type_name < #i >
                #where_clause_ref
            {
                type Result = #type_name < _TI >;

                fn fold_with<'i>(
                    &self,
                    folder: &mut dyn Folder < 'i, #i, _TI >,
                    outer_binder: DebruijnIndex,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result>
                where
                    #i: 'i,
                    _TI: 'i,
                {
                    #body
                }
            }
        });
    }

    panic!("derive(Fold) requires a parameter that implements HasInterner or Interner");
}

/// Generates the body of the Fold impl
fn derive_fold_body(type_name: &Ident, data: Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(s) => {
            let fields = s.fields.into_iter().map(|f| {
                let name = f.ident.as_ref().expect("Unnamed field in Foldable struct");
                quote! { #name: self.#name.fold_with(folder, outer_binder)? }
            });
            quote! {
                Ok(#type_name {
                    #(#fields),*
                })
            }
        }
        Data::Enum(e) => {
            let matches = e.variants.into_iter().map(|v| {
                let variant = v.ident;
                match &v.fields {
                    syn::Fields::Named(fields) => {
                        let fnames: &Vec<_> = &fields.named.iter().map(|f| &f.ident).collect();
                        let fnames1: &Vec<_> = fnames;
                        quote! {
                            #type_name :: #variant { #(#fnames),* } => {
                                Ok(#type_name :: #variant {
                                    #(#fnames: #fnames1),*
                                })
                            }
                        }
                    }

                    syn::Fields::Unnamed(_fields) => {
                        let names: Vec<_> = (0..v.fields.iter().count())
                            .map(|index| format_ident!("a{}", index))
                            .collect();
                        quote! {
                            #type_name::#variant( #(ref #names),* ) => {
                                Ok(#type_name::#variant( #(#names.fold_with(folder, outer_binder)?),* ))
                            }
                        }
                    }

                    syn::Fields::Unit => {
                        quote! {
                            #type_name::#variant => {
                                Ok(#type_name::#variant)
                            }
                        }
                    }
                }
            });
            quote! {
                match *self {
                    #(#matches)*
                }
            }
        }
        Data::Union(..) => panic!("Fold can not be derived for unions"),
    }
}

#[proc_macro_derive(HasInterner, attributes(has_interner))]
pub fn derive_has_interner(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause_ref) = input.generics.split_for_impl();

    let type_name = input.ident;

    if let Some(attr) = input.attrs.iter().find(|a| a.path.is_ident("has_interner")) {
        // Hardcoded interner:
        //
        // impl HasInterner for Type {
        //     type Result = XXX;
        // }
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_interner argument");

        return TokenStream::from(quote! {
            impl #impl_generics HasInterner for #type_name #ty_generics #where_clause_ref {
                type Interner = #arg;
            }
        });
    }

    match input.generics.params.len() {
        1 => {}

        0 => {
            panic!("Interner derive requires a single type parameter or a `#[has_interner]` attr");
        }

        _ => {
            panic!("Interner derive only works with a single type parameter");
        }
    };

    let generic_param0 = &input.generics.params[0];

    if let Some(param) = has_interner(&generic_param0) {
        // HasInterner bound:
        //
        // Example:
        //
        // impl<T, _I> HasInterner for Binders<T>
        // where
        //     T: HasInterner<Interner = _I>,
        //     _I: Interner,
        // {
        //     type Result = _I;
        // }

        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![GenericParam::Type(
            syn::parse(quote! { _I: Interner }.into()).unwrap(),
        )]);

        let mut where_clause = where_clause_ref
            .cloned()
            .unwrap_or_else(|| syn::parse2(quote![where]).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: HasInterner<Interner = _I> }).unwrap());

        return TokenStream::from(quote! {
            impl #impl_generics HasInterner for #type_name < #param >
                #where_clause
            {
                type Interner = _I;
            }
        });
    }

    // Interner bound:
    //
    // Example:
    //
    // impl<I> HasInterner for Foo<I>
    // where
    //     I: Interner,
    // {
    //     type Interner = I;
    // }

    if let Some(i) = is_interner(&generic_param0) {
        let impl_generics = &input.generics;

        return TokenStream::from(quote! {
            impl #impl_generics HasInterner for #type_name < #i >
                #where_clause_ref
            {
                type Interner = #i;
            }
        });
    }

    panic!("derive(Interner) requires a parameter that implements HasInterner or Interner");
}

/// Checks whether a generic parameter has a `: HasInterner` bound
fn has_interner(param: &GenericParam) -> Option<&Ident> {
    bounded_by_trait(param, "HasInterner")
}

/// Checks whether a generic parameter has a `: Interner` bound
fn is_interner(param: &GenericParam) -> Option<&Ident> {
    bounded_by_trait(param, "Interner")
}

fn bounded_by_trait<'p>(param: &'p GenericParam, name: &str) -> Option<&'p Ident> {
    let name = Some(String::from(name));
    match param {
        GenericParam::Type(ref t) => t.bounds.iter().find_map(|b| {
            if let TypeParamBound::Trait(trait_bound) = b {
                if trait_bound
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    == name
                {
                    return Some(&t.ident);
                }
            }
            None
        }),
        _ => None,
    }
}

/// Derives Visit for structs and enums for which one of the following is true:
/// - It has a `#[has_interner(TheInterner)]` attribute
/// - There is a single parameter `T: HasInterner` (does not have to be named `T`)
/// - There is a single parameter `I: Interner` (does not have to be named `I`)
#[proc_macro_derive(Visit, attributes(has_interner))]
pub fn derive_visit(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause_ref) = input.generics.split_for_impl();

    let type_name = input.ident;
    let body = derive_visit_body(&type_name, input.data);

    if let Some(attr) = input.attrs.iter().find(|a| a.path.is_ident("has_interner")) {
        // Hardcoded interner:
        //
        // impl Visit<ChalkIr> for Type {
        //
        // }
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_interner argument");

        return TokenStream::from(quote! {
            impl #impl_generics Visit < #arg > for #type_name #ty_generics #where_clause_ref {
                fn visit_with<'i, R: VisitResult>(
                    &self,
                    visitor: &mut dyn Visitor < 'i, #arg, Result = R >,
                    outer_binder: DebruijnIndex,
                ) -> R
                where
                    I: 'i
                {
                    #body
                }
            }
        });
    }

    match input.generics.params.len() {
        1 => {}

        0 => {
            panic!("Visit derive requires a single type parameter or a `#[has_interner]` attr");
        }

        _ => {
            panic!("Visit derive only works with a single type parameter");
        }
    };

    let generic_param0 = &input.generics.params[0];

    if let Some(param) = has_interner(&generic_param0) {
        // HasInterner bound:
        //
        // Example:
        //
        // impl<T, _I> Visit<_I> for Binders<T>
        // where
        //     T: HasInterner<Interner = _I>,
        // {
        // }

        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![GenericParam::Type(
            syn::parse(quote! { _I: Interner }.into()).unwrap(),
        )]);

        let mut where_clause = where_clause_ref
            .cloned()
            .unwrap_or_else(|| syn::parse2(quote![where]).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: HasInterner<Interner = _I> }).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: Visit<_I> }).unwrap());

        return TokenStream::from(quote! {
            impl #impl_generics Visit < _I > for #type_name < #param >
                #where_clause
            {
                fn visit_with<'i, R: VisitResult>(
                    &self,
                    visitor: &mut dyn Visitor < 'i, _I, Result = R >,
                    outer_binder: DebruijnIndex,
                ) -> R
                where
                    _I: 'i
                {
                    #body
                }
            }
        });
    }

    // Interner bound:
    //
    // Example:
    //
    // impl<I> Visit<I> for Foo<I>
    // where
    //     I: Interner,
    // {
    // }

    if let Some(i) = is_interner(&generic_param0) {
        let impl_generics = &input.generics;

        return TokenStream::from(quote! {
            impl #impl_generics Visit < #i > for #type_name < #i >
                #where_clause_ref
            {
                fn visit_with<'i, R: VisitResult>(
                    &self,
                    visitor: &mut dyn Visitor < 'i, #i, Result = R >,
                    outer_binder: DebruijnIndex,
                ) -> R
                where
                    I: 'i
                {
                    #body
                }
            }
        });
    }

    panic!("derive(Visit) requires a parameter that implements HasInterner or Interner");
}

/// Generates the body of the Visit impl
fn derive_visit_body(type_name: &Ident, data: Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(s) => {
            let fields = s.fields.into_iter().map(|f| {
                let name = f.ident.as_ref().expect("Unnamed field in a struct");
                quote! { .and_then(|| self.#name.visit_with(visitor, outer_binder)) }
            });
            quote! {
                R::new()
                  #(#fields)*
            }
        }
        Data::Enum(e) => {
            let matches = e.variants.into_iter().map(|v| {
                let variant = v.ident;
                match &v.fields {
                    syn::Fields::Named(fields) => {
                        let fnames: &Vec<_> = &fields.named.iter().map(|f| &f.ident).collect();
                        quote! {
                            #type_name :: #variant { #(#fnames),* } => {
                                R::new()
                                    #(.and_then(|| #fnames.visit_with(visitor, outer_binder)))*
                            }
                        }
                    }

                    syn::Fields::Unnamed(_fields) => {
                        let names: Vec<_> = (0..v.fields.iter().count())
                            .map(|index| format_ident!("a{}", index))
                            .collect();
                        quote! {
                            #type_name::#variant( #(ref #names),* ) => {
                                R::new()
                                  #(.and_then(|| #names.visit_with(visitor, outer_binder)))*
                            }
                        }
                    }

                    syn::Fields::Unit => {
                        quote! {
                            #type_name::#variant => R::new(),
                        }
                    }
                }
            });
            quote! {
                match *self {
                    #(#matches)*
                }
            }
        }
        Data::Union(..) => panic!("Visit can not be derived for unions"),
    }
}
