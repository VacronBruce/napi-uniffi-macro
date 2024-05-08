mod attr;
mod item_impl;
mod util;

use attr::*;
use item_impl::parse_impl;
use util::*;

use proc_macro::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_attribute]
pub fn export_error(_attr: TokenStream, items: TokenStream) -> TokenStream {
    let stream: proc_macro2::TokenStream = items.into();
    let input = syn::parse2::<Item>(stream).unwrap();
    let q = match input {
        Item::Enum(e) => {
            // println!("enum ident is -> {:?}", e.ident);
            let ident = e.ident.clone();
            quote! {   
                #[cfg(feature = "node")]
                #e

                #[cfg(feature = "node")]
                impl From<#ident> for napi::Error {
                    fn from(error: #ident) -> Self {
                        match error {
                            _ => napi::Error::new(napi::Status::GenericFailure, error.to_string()),
                        }
                    }
                }

                #[cfg(feature = "ffi")]
                #[derive(uniffi::Error)]
                #[uniffi(flat_error)]
                #e   
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

#[proc_macro_attribute]
pub fn export(attr: TokenStream, items: TokenStream) -> TokenStream {
    let stream: proc_macro2::TokenStream = items.into();
    let input = syn::parse2::<Item>(stream).unwrap();
    let args = parse_macro_input!(attr as CustomAttributes);
    let q = match input {
        Item::Impl(im) => {
            parse_impl(im)
        }
        Item::Enum(e) => {
            quote! {       
                #[cfg(feature = "node")]
                #[napi]
                #e

                #[cfg(feature = "ffi")]
                #[derive(uniffi::Enum)]
                #e
            }
        }
        Item::Struct(s) => {
            let napi_attr = generate_napi_attrs(&args);
            quote! {
                #[cfg(feature = "node")]
                #napi_attr
                #s

                #[cfg(feature = "ffi")]
                #[derive(uniffi::Object)]
                #s
            }
        }
        Item::Fn(f) => {
            parse_item_fn(f)
        }
        _ => {
            quote! {
                //Nothing to do
            }
        }
    };
    
    q.into()
}

fn generate_napi_attrs(args: &CustomAttributes) -> proc_macro2::TokenStream {
    if args.contain_key("object") {
        return quote!{#[napi(object)]};
    } else {
        return quote!{#[napi]};
    }
}

fn parse_item_fn(f: ItemFn) -> proc_macro2::TokenStream {
    let mut modify = f.clone();
    if let Some(arg) = parse_result_type(modify.sig.output.clone()) {
        let tt: Type = parse_quote! {
            napi::Result<#arg>
        };
        modify.sig.output = ReturnType::Type(Default::default(), Box::new(tt))
    }

    let mut modify_ffi = f.clone();
    if contain_async(modify_ffi.sig.clone()) {
        modify_ffi = add_tokio_async(modify_ffi);
    }
    
    quote! {
        #[cfg(feature = "node")]
        #[napi]
        #modify

        #[cfg(feature = "ffi")]
        #[uniffi::export]
        #modify_ffi
    }
}

fn contain_async(s: Signature) -> bool {
    s.asyncness.is_some()
}

fn add_tokio_async(f: syn::ItemFn) -> syn::ItemFn {
    let mut modify = f.clone();
    let body = f.block.clone();
    let q = quote! {
        {        
            tokio::runtime::Builder::new_current_thread()
            .enable_all().build().unwrap().block_on(async 
                #body
            )
        }
    };
    modify.block = syn::parse(q.into()).expect("Should parse success");
    return modify;
}
