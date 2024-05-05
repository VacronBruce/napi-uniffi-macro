use proc_macro::TokenStream;
use quote::*;
use syn::{parse_quote, Block, GenericArgument, Item, ReturnType, Signature, Type};

#[proc_macro_attribute]
pub fn export_error(_attr: TokenStream, items: TokenStream) -> TokenStream {
    let stream2: proc_macro2::TokenStream = items.into();
    let input = syn::parse2::<Item>(stream2).unwrap();
    
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
pub fn export(_attr: TokenStream, items: TokenStream) -> TokenStream {
    let stream2: proc_macro2::TokenStream = items.into();
    let input = syn::parse2::<Item>(stream2).unwrap();
    let q = match input {
        Item::Impl(im) => {
            quote! {
                #[cfg(feature = "node")]
                #[napi]
                #im

                #[cfg(feature = "ffi")]
                #[uniffi::export]
                #im
            }
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
            quote! {
                #[cfg(feature = "node")]
                #[napi]
                #s

                #[cfg(feature = "ffi")]
                #[derive(uniffi::Object)]
                #s
            }
        }
        Item::Fn(f) => {
            let mut modify = f.clone();
            if let Some(arg) = get_result_first_arg(modify.sig.output.clone()) {
                let tt: Type = parse_quote! {
                    napi::Result<#arg>
                };
                modify.sig.output = ReturnType::Type(Default::default(), Box::new(tt))
            }

            let mut modify_ffi = f.clone();
            if contain_async(modify_ffi.sig.clone()) {
                modify_ffi = modify_for_ffi_body(modify_ffi);
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
        _ => {
            quote! {
                //Nothing to do
            }
        }
    };
    
    q.into()
}

fn get_result_first_arg(rt: ReturnType) -> Option<GenericArgument> {
    let t = match rt {
        ReturnType::Default => return None,
        ReturnType::Type(_, t) => t,
    };

    let path = match *t {
        Type::Path(path) => {path},
        _ => return None,
    };

    let seg = if let Some(seg) = path.path.segments.last() { seg } else {return None;};
    if seg.ident.to_string() != "Result" {
        return None;
    }

    if let syn::PathArguments::AngleBracketed(ref angle_bracketed) = seg.arguments {
        if let Some(arg) = angle_bracketed.args.first() {
            if let syn::GenericArgument::Type(_) = arg {
                return Some(arg.clone());
            }
        }
    }
    
    None
}

fn contain_async(s: Signature) -> bool {
    s.asyncness.is_some()
}

fn modify_for_ffi_body(f: syn::ItemFn) -> syn::ItemFn {
    let mut modify = f.clone();
    let body = f.block.clone();
    let q = quote! {
        {        
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async 
                #body
            )
        }
    };
    modify.block = syn::parse(q.into()).expect("Should parse success");
    return modify;
}

fn print_type(t: Type) {
    match t.clone() {
        Type::Array(_) => println!("Array"),
        Type::BareFn(_) => println!("Bare function"),
        Type::Group(_) => println!("Group"),
        Type::ImplTrait(_) => println!("Impl Trait"),
        Type::Infer(_) => println!("Infer"),
        Type::Macro(_) => println!("Macro"),
        Type::Never(_) => println!("Never"),
        Type::Paren(_) => println!("Parenthesized"),
        Type::Path(path) => {
            println!("We got path");
            

        },
        Type::Ptr(_) => println!("Pointer"),
        Type::Reference(_) => println!("Reference"),
        Type::Slice(_) => println!("Slice"),
        Type::TraitObject(_) => println!("Trait Object"),
        Type::Tuple(_) => println!("Tuple"),
        _ => println!("Other"),
    }
    let type_string = quote::quote!(#t).to_string();
    println!("type -> {}", type_string);
}