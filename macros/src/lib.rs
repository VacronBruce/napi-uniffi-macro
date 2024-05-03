use proc_macro::TokenStream;
use quote::*;
use syn::Item;

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, items: TokenStream) -> TokenStream {
    let stream2: proc_macro2::TokenStream = items.into();
    let input = syn::parse2::<Item>(stream2).unwrap();
    let q = match input {
        Item::Struct(s) => {
            quote! {
                #[cfg(feature = "node")]
                #[napi]
                #s

                #[cfg(feature = "ffi")]
                #[uniffi::export]
                #s
            }
        }
        Item::Fn(f) => {
            quote! {
                #[cfg(feature = "node")]
                #[napi]
                #f

                #[cfg(feature = "ffi")]
                #[uniffi::export]
                #f
            }
        }
        _ => {
            quote! {
                //Nothing to do
            }
        }
    };
    
    q.into()
}