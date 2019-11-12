extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, GenericParam, Ident, TypeParamBound};

/// Derives Fold for structs and enums for which one of the following is true:
/// - It has a `#[has_type_family(TheTypeFamily)]` attribute
/// - There is a single parameter `T: HasTypeFamily` (does not have to be named `T`)
/// - There is a single parameter `TF: TypeFamily` (does not have to be named `TF`)
#[proc_macro_derive(Fold, attributes(has_type_family))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause_ref) = input.generics.split_for_impl();

    let type_name = input.ident;
    let body = derive_fold_body(&type_name, input.data);

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("has_type_family"))
    {
        // Hardcoded type-family:
        //
        // impl Fold<ChalkIr, ChalkIr> for Type {
        //     type Result = Self;
        // }
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_type_family argument");

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #arg, #arg > for #type_name #ty_generics #where_clause_ref {
                type Result = Self;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < #arg, #arg >,
                    binders: usize,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                    #body
                }
            }
        });
    }

    match input.generics.params.len() {
        1 => {}

        0 => {
            panic!("Fold derive requires a single type parameter or a `#[has_type_family]` attr");
        }

        _ => {
            panic!("Fold derive only works with a single type parameter");
        }
    };

    let generic_param0 = &input.generics.params[0];

    if let Some(param) = has_type_family(&generic_param0) {
        // HasTypeFamily bound:
        //
        // Example:
        //
        // impl<T, _TF, _TTF, _U> Fold<_TF, _TTF> for Binders<T>
        // where
        //     T: HasTypeFamily<TypeFamily = _TF>,
        //     T: Fold<_TF, _TTF, Result = _U>,
        //     U: HasTypeFamily<TypeFamily = _TTF>,
        // {
        //     type Result = Binders<_U>;
        // }

        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![
            GenericParam::Type(syn::parse(quote! { _TF: TypeFamily }.into()).unwrap()),
            GenericParam::Type(syn::parse(quote! { _TTF: TypeFamily }.into()).unwrap()),
            GenericParam::Type(
                syn::parse(quote! { _U: HasTypeFamily<TypeFamily = _TTF> }.into()).unwrap(),
            ),
        ]);

        let mut where_clause = where_clause_ref
            .cloned()
            .unwrap_or_else(|| syn::parse2(quote![where]).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: HasTypeFamily<TypeFamily = _TF> }).unwrap());
        where_clause
            .predicates
            .push(syn::parse2(quote! { #param: Fold<_TF, _TTF, Result = _U> }).unwrap());

        return TokenStream::from(quote! {
            impl #impl_generics Fold < _TF, _TTF > for #type_name < #param >
                #where_clause
            {
                type Result = #type_name < _U >;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < _TF, _TTF >,
                    binders: usize,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                    #body
                }
            }
        });
    }

    // TypeFamily bound:
    //
    // Example:
    //
    // impl<TF, _TTF> Fold<TF, _TTF> for Foo<TF>
    // where
    //     TF: HasTypeFamily,
    //     _TTF: HasTypeFamily,
    // {
    //     type Result = Foo<_TTF>;
    // }

    if let Some(tf) = is_type_family(&generic_param0) {
        let mut impl_generics = input.generics.clone();
        impl_generics.params.extend(vec![GenericParam::Type(
            syn::parse(quote! { _TTF: TypeFamily }.into()).unwrap(),
        )]);

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #tf, _TTF > for #type_name < #tf >
                #where_clause_ref
            {
                type Result = #type_name < _TTF >;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < #tf, _TTF >,
                    binders: usize,
                ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                    #body
                }
            }
        });
    }

    panic!("derive(Fold) requires a parameter that implements HasTypeFamily or TypeFamily");
}

/// Generates the body of the Fold impl
fn derive_fold_body(type_name: &Ident, data: Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(s) => {
            let fields = s.fields.into_iter().map(|f| {
                let name = f.ident.as_ref().expect("Unnamed field in Foldable struct");
                quote! { #name: self.#name.fold_with(folder, binders)? }
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
                let names: Vec<_> = (0..v.fields.iter().count())
                    .map(|index| format_ident!("a{}", index))
                    .collect();
                quote! {
                    #type_name::#variant( #(ref #names),* ) => {
                        Ok(#type_name::#variant( #(#names.fold_with(folder, binders)?),* ))
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

/// Checks whether a generic parameter has a `: HasTypeFamily` bound
fn has_type_family(param: &GenericParam) -> Option<&Ident> {
    bounded_by_trait(param, "HasTypeFamily")
}

/// Checks whether a generic parameter has a `: TypeFamily` bound
fn is_type_family(param: &GenericParam) -> Option<&Ident> {
    bounded_by_trait(param, "TypeFamily")
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
