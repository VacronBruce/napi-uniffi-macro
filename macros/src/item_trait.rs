use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, FnArg, Ident, ItemTrait, ReturnType, TraitItem, TraitItemFn, Type};

use crate::{string_to_ident, string_to_type};

pub fn parse_item_trait(t: ItemTrait) -> TokenStream {
    let name = t.ident.clone();
    let adapter = string_to_ident(format!("{}Adapter", t.ident.clone().to_string()));
    let v: Vec<(Ident, Type)> = t.items.iter().filter_map(|f| match f {
        TraitItem::Fn(f) => {
            Some(generate_tsfn_signature(f.clone()))
        },
        _ => None
    }).collect();

    let fields = v.iter().fold(quote!{}, |acc, f| 
        {
            let name = f.0.clone();
            let t = f.1.clone();
            quote!{ #acc #name: ThreadsafeFunction<#t>,}
        });
    let arg_names = v.iter().fold(quote!{}, |acc, f| {let name = f.0.clone(); quote!{ #acc #name,}});

    let impls = t.items.iter().filter_map(|f| match f {
        TraitItem::Fn(f) => {
            Some(generate_trait_impl(f.clone()))
        },
        _ => None
    }).collect::<Vec<TokenStream>>().iter().fold(quote!{}, |acc, f| 
        quote!{ 
            #acc 
            #f
        });


    quote! {
        #[cfg(feature = "node")]
        #t
        
        #[cfg(feature = "node")]
        pub struct #adapter {
            #fields
        }

        #[cfg(feature = "node")]
        impl #adapter {
            pub fn new(#fields) -> Self {
                Self {
                    #arg_names
                }
            }
        }

        #[cfg(feature = "node")]
        impl #name for #adapter {
            #impls
        }

        #[cfg(feature = "ffi")]
        #[uniffi::export(with_foreign)]
        #t
    }
}

fn generate_tsfn_signature(f: TraitItemFn) -> (Ident, Type) {
    let name = f.sig.ident.clone();
    let inputs = f.sig.inputs.clone();
    let tsfn_types = string_to_type(extract_argument_types(&inputs));
    let tsfn_name = string_to_ident(format!("{}_cb", name.to_string()));
    (tsfn_name, tsfn_types)
}

fn generate_trait_impl(f: TraitItemFn) -> TokenStream {
    let name = f.sig.ident.clone();
    let cb = string_to_ident(format!("{}_cb", name.to_string()));
    let sign = f.sig.clone();
    let output = match f.sig.output.clone() {
        ReturnType::Type(_, t) => Some(t),
        ReturnType::Default => None,
    };
    let inputs = f.sig.inputs.clone();
    let tsfn_names = string_to_type(extract_argument_names(&inputs));
    let body = if output.is_some() {
        quote!{
            let (tx, rx) = oneshot::channel::<#output>();
            self.#cb.call_with_return_value(
                Ok(#tsfn_names),
                ThreadsafeFunctionCallMode::NonBlocking,
                |t: #output| {
                    let _ = tx.send(t);
                    Ok(())
                },
            );
            rx.recv().unwrap_or_default().into()        
        }
    } else {
        quote! { self.#cb.call(Ok(#tsfn_names), ThreadsafeFunctionCallMode::NonBlocking)}
    };
    quote! {
        #sign {
            #body
        }
    }
}


fn extract_argument_types(args: &Punctuated<FnArg, syn::token::Comma>) -> String {
    let v: Vec<String> = args.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(t) => { let ty = &t.ty; Some(quote!(#ty).to_string()) },
            _ => None,
        }
    }).collect();

    format!("({})", v.join(","))
}

fn extract_argument_names(args: &Punctuated<FnArg, syn::token::Comma>) -> String {
    let v: Vec<String> = args.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(t) => { let p = &t.pat; Some(quote!(#p).to_string()) },
            _ => None,
        }
    }).collect();

    format!("({})", v.join(","))
}