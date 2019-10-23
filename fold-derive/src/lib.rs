extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct};

#[proc_macro_derive(Fold)]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = fold_fields(&input.fields);

    TokenStream::from(quote! {
        impl#impl_generics Fold #ty_generics for #name #ty_generics #where_clause {
            type Result = Self;

            fn fold_with(
                &self,
                folder: &mut dyn crate::fold::Folder #ty_generics,
                binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    })
}

fn fold_fields(fields: &Fields) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = f.ident.as_ref().expect("Unnamed field in Foldable struct");
        quote! { #name: self.#name.fold_with(folder, binders)? }
    })
}
