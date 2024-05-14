use syn::{parse_quote, punctuated::Punctuated, FnArg, GenericArgument, ItemFn, Lit, Pat, PatType, ReturnType, Token, Type};
use quote::quote;

use crate::CustomAttributes;
pub fn parse_result_type(rt: ReturnType) -> Option<GenericArgument> {
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

pub fn string_to_ident(s: String) -> proc_macro2::Ident {
    syn::parse_str(&s).expect("Should parse success")
}

pub fn string_to_type(s: String) -> syn::Type {
    syn::parse_str(&s).expect("Should parse success")
}

pub fn extract_argument_types(args: &Punctuated<FnArg, syn::token::Comma>) -> String {
    let v: Vec<String> = args.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(t) => { let ty = &t.ty; Some(quote!(#ty).to_string()) },
            _ => None,
        }
    }).collect();

    format!("({})", v.join(","))
}

pub fn extract_argument_names(args: &Punctuated<FnArg, syn::token::Comma>) -> String {
    let v: Vec<String> = args.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(t) => { let p = &t.pat; Some(quote!(#p).to_string()) },
            _ => None,
        }
    }).collect();

    format!("({})", v.join(","))
}

fn get_object_args_list(args: CustomAttributes) -> Vec<String> {
    //example: object_params = "info, join, test"
    let lit = args.get_meta_value("object_params");
    if let Some(l) = lit {
        match l {
            Lit::Str(s) => {
                return s.value().split(",").map(|s| s.to_owned()).collect();
            }
            _ => {}
        };
    }
    vec![]
}

pub fn add_reference_to_ffi_inputs(args: CustomAttributes, f: ItemFn) -> ItemFn {
    let mut modify = f.clone();
    let need_modified_list = get_object_args_list(args);
    let inputs = modify.sig.inputs.clone();
    let mut clone_inputs: Vec<proc_macro2::TokenStream> = vec![];
    let modified_inputs = inputs.into_iter().map(|arg| {
        let mut arg = arg;
        if let FnArg::Typed(ref mut pat_type) = arg {
            if let Pat::Ident(ref mut pat_ident) = *pat_type.pat {
                if need_modified_list.contains(&pat_ident.ident.to_string()) {
                    let ty = *pat_type.ty.clone();
                    *pat_type.ty = parse_quote!(&#ty);
                    let ident = &pat_ident.ident;
                    clone_inputs.push(quote!{let #ident = #ident.clone();});
                }
            }
        }
        arg
    }).collect::<Punctuated<FnArg, Token![,]>>();

    let body_clone = clone_inputs.iter().fold(quote!{}, |acc, f| {
            quote!{ #acc #f}
    });

    
    let body = f.block.stmts.iter().fold(quote!{}, |acc, f| {
        quote!{#acc #f}
    });
    let q = quote! {
        {        
            #body_clone        
            #body
        }
    };
    modify.block = syn::parse(q.into()).expect("Should parse success");
    modify.sig.inputs = modified_inputs;
    modify
}