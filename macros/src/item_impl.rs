use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Attribute, ImplItemFn, ImplItem, ItemImpl, ReturnType, Type};
pub fn parse_impl(im: ItemImpl) -> TokenStream {
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

pub fn add_ffi_constructor_fn(f: ImplItemFn) -> ImplItemFn {
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

pub fn add_napi_constructor_fn(f: ImplItemFn) -> ImplItemFn {
    let mut modify = f.clone();
    let attr: Attribute = parse_quote!{#[napi(constructor)]};
    modify.attrs = vec![attr];
    modify
}

fn contain_constructor(f: &ImplItemFn) -> bool {
    let attr = f.attrs.first();
    if let Some(a) = attr {
        return quote!{#a}.to_string().contains("constructor");
    }
    return false;
}