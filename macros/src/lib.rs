use proc_macro::TokenStream;
use quote::*;
use syn::{parse::{Parse, ParseStream}, *};

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
            let mut napi_ii = im.clone();
            let mut uniffi_ii = im.clone();
            let mut napi_items: Vec<ImplItem> = vec![];
            let mut ffi_items: Vec<ImplItem> = vec![];
            for item in napi_ii.items {
                if let ImplItem::Fn(f) = item {
                    if contain_constructor(&f) {
                        napi_items.push(ImplItem::Fn(add_napi_constructor_fn(f.clone())));
                        ffi_items.push(ImplItem::Fn(add_ffi_constructor_fn(f)));
                    } else {
                        napi_items.push(ImplItem::Fn(f.clone()));
                        ffi_items.push(ImplItem::Fn(f));
                    }
                } else {
                    napi_items.push(item.clone());
                    ffi_items.push(item);
                }
            }

            napi_ii.items = napi_items;
            uniffi_ii.items = ffi_items;
            
            quote! {
                #[cfg(feature = "node")]
                #[napi]
                #napi_ii

                #[cfg(feature = "ffi")]
                #[uniffi::export]
                #uniffi_ii
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
            let napi_attr = generate_napi_attrs(args.identifiers);

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
            let mut modify = f.clone();
            if let Some(arg) = get_result_first_arg(modify.sig.output.clone()) {
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

fn add_tokio_async(f: syn::ItemFn) -> syn::ItemFn {
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

// Define a custom attribute struct
struct CustomAttributes {
    identifiers: Vec<Ident>,
    ident_lit_pairs: Vec<(Ident, Lit)>,
}

impl Parse for CustomAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut identifiers = Vec::new();
        let mut ident_lit_pairs: Vec<(Ident, Lit)> = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if input.peek(Token![=]) {
                //#[export(fed = "123")]
                let _ = input.parse::<Token![=]>();
                let lit: Lit = input.parse()?;
                ident_lit_pairs.push((ident, lit));
            } else {
                //#[export(object)]
                identifiers.push(ident);
            }

            if input.peek(Token![,]) {
                //consume comma
                let _ = input.parse::<Token![,]>();
            }
        }

        Ok(CustomAttributes { identifiers, ident_lit_pairs })
    }
}

fn contain_key(identifiers: Vec<Ident>, key: &str) -> bool {
    identifiers.iter().any(|ident| ident.to_string() == key)   
}

fn generate_napi_attrs(identifiers: Vec<Ident>) -> proc_macro2::TokenStream {
    if contain_key(identifiers, "object") {
        return quote! {#[napi(object)]};
    } else {
        return quote!{#[napi]};
    }
}

fn contain_constructor(f: &ImplItemFn) -> bool {
    let attr = f.attrs.first();
    if let Some(a) = attr {
        return quote!{#a}.to_string().contains("constructor");
    }
    return false;
}

fn add_ffi_constructor_fn(f: ImplItemFn) -> ImplItemFn {
    let mut modify = f.clone();
    let attr: Attribute = parse_quote!{#[uniffi::constructor]};
    modify.attrs = vec![attr];
    let body = f.block.clone();
    let q = quote! {
        {Arc::new(#body)}
    };
    let tt: Type = parse_quote! {
        Arc<Self>
    };
    modify.sig.output = ReturnType::Type(Default::default(), Box::new(tt));
    modify.block = syn::parse(q.into()).expect("Should parse success");
    modify
}

fn add_napi_constructor_fn(f: ImplItemFn) -> ImplItemFn {
    let mut modify = f.clone();
    let attr: Attribute = parse_quote!{#[napi(constructor)]};
    modify.attrs = vec![attr];
    modify
}