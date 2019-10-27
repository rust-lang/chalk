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

    let name = input.ident;
    let body = derive_fold_body(input.data);

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("has_type_family"))
    {
        // Hardcoded type-family:
        //
        // impl Fold<ChalkIr> for Type {
        //     type Result = Self;
        // }
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_type_family argument");

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #arg > for #name #ty_generics #where_clause_ref {
                type Result = Self;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < #arg >,
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
        // impl<T> Fold<<T as HasTypeFamily>::TypeFamily> for Binders<T>
        // where
        //     T: HasTypeFamily,
        //     T: Fold<<T as HasTypeFamily>::TypeFamily, Result = T>
        // {
        //     type Result = Binders<T>;
        // }

        let tf = quote! { <#param as HasTypeFamily>::TypeFamily };
        let mut where_clause = where_clause_ref.cloned();
        where_clause
            .get_or_insert(syn::parse2(quote![where]).unwrap())
            .predicates
            .push(syn::parse2(quote! { #param: Fold<#tf, Result = #param> }).unwrap());

        return TokenStream::from(quote! {
            impl #impl_generics Fold < #tf > for #name < #param >
                #where_clause
            {
                type Result = #name < #param >;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < #tf >,
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
    // impl<TF> Fold<TF> for Foo<TF>
    // where
    //     TF: HasTypeFamily,
    // {
    //     type Result = Foo<TF>;
    // }

    if let Some(tf) = is_type_family(&generic_param0) {
        return TokenStream::from(quote! {
            impl #impl_generics Fold < #tf > for #name < #tf >
                #where_clause_ref
            {
                type Result = #name < #tf >;

                fn fold_with(
                    &self,
                    folder: &mut dyn Folder < #tf >,
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
fn derive_fold_body(data: Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(s) => {
            let fields = s.fields.into_iter().map(|f| {
                let name = f.ident.as_ref().expect("Unnamed field in Foldable struct");
                quote! { #name: self.#name.fold_with(folder, binders)? }
            });
            quote! {
                Ok(Self {
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
                    Self::#variant( #(ref #names),* ) => {
                        Ok(Self::#variant( #(#names.fold_with(folder, binders)?),* ))
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
    match param {
        GenericParam::Type(ref t) => t.bounds.iter().find_map(|b| {
            if let TypeParamBound::Trait(trait_bound) = b {
                if trait_bound
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    == Some(String::from("HasTypeFamily"))
                {
                    return Some(&t.ident);
                }
            }
            None
        }),
        _ => None,
    }
}

/// Checks whether a generic parameter has a `: TypeFamily` bound
fn is_type_family(param: &GenericParam) -> Option<&Ident> {
    match param {
        GenericParam::Type(ref t) => t.bounds.iter().find_map(|b| {
            if let TypeParamBound::Trait(trait_bound) = b {
                if trait_bound
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    == Some(String::from("TypeFamily"))
                {
                    return Some(&t.ident);
                }
            }
            None
        }),
        _ => None,
    }
}
