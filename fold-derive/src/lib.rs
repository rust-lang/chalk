extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Fold, attributes(fold_family))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let body = match input.data {
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
    };

    // Allow custom Fold parameter using the `#[fold_family(Param)]` attribute
    let generics = if let Some(attr) = input.attrs.iter().find(|a| a.path.is_ident("fold_family")) {
        let arg = attr
            .parse_args::<proc_macro2::TokenStream>()
            .expect("Expected fold_family argument");
        quote! { < #arg > }
    } else {
        ty_generics.to_token_stream()
    };

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

#[proc_macro_attribute]
pub fn fold_family(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
