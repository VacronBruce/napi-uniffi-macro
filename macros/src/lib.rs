mod attr;
mod item_impl;
mod item_trait;
mod util;

use attr::*;
use item_impl::parse_impl;
use item_trait::parse_item_trait;
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
            let ident = e.ident.clone();
            quote! {   
                #[cfg(test)]
                #e
                
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
                #[cfg(test)]
                #e

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
            let mut ffi_construct: Option<proc_macro2::TokenStream> = None;
            if args.contain_key("object") {
                ffi_construct = Some(create_ffi_constructor(s.clone()));
            }
            quote! {
                #[cfg(test)]
                #s
                
                #[cfg(feature = "node")]
                #napi_attr
                #s

                #[cfg(feature = "ffi")]
                #[derive(uniffi::Object)]
                #s

                #ffi_construct
            }
        }
        Item::Fn(f) => {
            parse_item_fn(f, args)
        }
        Item::Trait(it) => {
            parse_item_trait(it)
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

fn parse_item_fn(f: ItemFn, args: CustomAttributes) -> proc_macro2::TokenStream {
    let mut modify = f.clone();
    if let Some(arg) = parse_result_type(modify.sig.output.clone()) {
        let tt: Type = parse_quote! {
            napi::Result<#arg>
        };
        modify.sig.output = ReturnType::Type(Default::default(), Box::new(tt))
    }

    let mut modify_ffi = add_reference_to_ffi_inputs(args, f.clone());
    if contain_async(modify_ffi.sig.clone()) {
        modify_ffi = add_tokio_async(modify_ffi);
    }
    
    quote! {
        #[cfg(test)]
        #f

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

fn is_builtin_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            // Assuming the first segment of the path is the type name
            if let Some(segment) = path.path.segments.first() {
                let ident = &segment.ident;
                // Check if the identifier matches a known built-in type
                match ident.to_string().as_str() {
                    "i8" | "i16" | "i32" | "i64" |
                    "u8" | "u16" | "u32" | "u64" |
                    "f32" | "f64" |
                    "bool" | "Vec<u8>" | "SystemTime" | "Duration"  => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn create_ffi_constructor(s: ItemStruct) -> proc_macro2::TokenStream {
    let ss = s.clone();
    let ident = ss.ident;
    let fields = ss.fields.clone();
    let v: Vec<(&Ident, &Type)> = fields.iter()
    .filter_map(|f| if let Some(t) = &f.ident {
        Some((t, &f.ty))
    } else {
        None
    }).collect();

    let getters = v.iter().fold(quote!{}, |acc, f| {
        let ident = f.0;
        let ty = f.1;
        let content = if is_builtin_type(ty) { quote!{self.#ident} } else {quote!{self.#ident.clone()}};
        quote! {
            #acc
            fn #ident (&self) -> #ty {
                #content
            }
        }
    });

    let args = v.iter().fold(quote!{}, |acc, f| 
        {
            let name = f.0; 
            let ty = f.1; 
            if is_builtin_type(ty) {
                quote!{#acc #name: #ty,}
            } else {
                quote!{#acc #name: &#ty,}
            }
        }
    );

    let types = v.iter().fold(quote!{}, |acc, f| {
        let name = f.0;
        if is_builtin_type(f.1) {
            quote!{#acc #name,}
        } else {
            quote!{#acc #name: #name.clone(),}
        }
    });

    quote! {
        #[cfg(feature = "ffi")]
        #[uniffi::export]
        impl #ident {
            #[uniffi::constructor]
            pub fn new(#args) -> Self {
                Self { #types }
            }
            #getters
        }
    }
} 