extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, GenericParam, Ident, TypeParamBound};

#[proc_macro_derive(Fold, attributes(has_type_family))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause_ref) = input.generics.split_for_impl();
    let body = derive_fold_body(input.data);

    // Allow custom Fold parameter using the `#[has_type_family(Param)]` attribute
    let mut generics = if let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("has_type_family"))
    {
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected has_type_family argument");
        quote! { < #arg > } // TODO extend instead of overriding?
    } else {
        ty_generics.to_token_stream()
    };

    let mut where_clause = where_clause_ref.cloned();

    // if any impl_generics G : HasTypeFamily:
    // #generics = TF
    // #where_clause += G: Fold<TF, Result = G>
    if let Some(param) = input
        .generics
        .params
        .iter()
        .find_map(|p| is_type_family_param(p))
    {
        let tf = quote! { <#param as HasTypeFamily>::TypeFamily };

        generics = quote! { <#tf> };
        // TODO extend where clause instead of overriding
        where_clause = syn::parse2(quote! { where #param: Fold<#tf, Result = #param> }).unwrap()
    }

    TokenStream::from(quote! {
        impl #impl_generics Fold #generics for #name #ty_generics #where_clause {
            type Result = Self;

            fn fold_with(
                &self,
                folder: &mut dyn Folder #generics,
                binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                #body
            }
        }
    })
}

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

fn is_type_family_param(param: &GenericParam) -> Option<&Ident> {
    match param {
        GenericParam::Type(ref t) => t.bounds.iter().find_map(|b| {
            if let TypeParamBound::Trait(trait_bound) = b {
                if trait_bound.path.get_ident().map(|i| i.to_string())
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
