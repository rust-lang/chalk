extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, GenericParam, Ident, TypeParamBound};

#[proc_macro_derive(Fold, attributes(fold_family))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let (_, ty_generics, where_clause_ref) = input.generics.split_for_impl();
    let body = derive_fold_body(input.data);

    // Allow custom Fold parameter using the `#[fold_family(Param)]` attribute
    let mut generics =
        if let Some(attr) = input.attrs.iter().find(|a| a.path.is_ident("fold_family")) {
            let arg = attr
                .parse_args::<proc_macro2::TokenStream>()
                .expect("Expected fold_family argument");
            quote! { < #arg > } // TODO extend instead of overriding?
        } else {
            ty_generics.to_token_stream()
        };

    let mut impl_generics = input.generics.clone();
    let mut where_clause = where_clause_ref.cloned();

    // if any impl_generics G : TypeFamily:
    // #impl_generics += TF: TypeFamily
    // #generics = TF
    // #where_clause += G: HasTypeFamily<TypeFamily = TF> + Fold<TF, Result = G>
    if let Some(param) = input
        .generics
        .params
        .iter()
        .find_map(|p| is_type_family_param(p))
    {
        impl_generics.params.push(GenericParam::Type(
            syn::parse(quote! { TF: TypeFamily }.into()).unwrap(),
        ));
        generics = quote! { <TF> };
        // TODO extend where clause instead of overriding
        where_clause = syn::parse(
            quote! { where #param: HasTypeFamily<TypeFamily = TF> + Fold<TF, Result = #param> }
                .into(),
        )
        .unwrap()
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

#[proc_macro_attribute]
pub fn fold_family(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
