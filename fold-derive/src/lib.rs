extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Fields, ItemStruct};

#[proc_macro_derive(Fold, attributes(fold_family))]
pub fn derive_fold(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = fold_fields(&input.fields);

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
        impl#impl_generics Fold #generics for #name #ty_generics #where_clause {
            type Result = Self;

            fn fold_with(
                &self,
                folder: &mut dyn Folder #generics,
                binders: usize,
            ) -> ::chalk_engine::fallible::Fallible<Self::Result> {
                Ok(Self {
                    #(#fields),*
                })
            }
        }
    })
}

#[proc_macro_attribute]
pub fn fold_family(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn fold_fields(fields: &Fields) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = f.ident.as_ref().expect("Unnamed field in Foldable struct");
        quote! { #name: self.#name.fold_with(folder, binders)? }
    })
}
