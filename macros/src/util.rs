use syn::{GenericArgument, ReturnType, Type};

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