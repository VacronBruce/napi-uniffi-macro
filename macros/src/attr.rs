
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Lit, Result, Token};

// Define a custom attribute struct
pub struct CustomAttributes {
    pub identifiers: Vec<Ident>,
    pub ident_lit_pairs: Vec<(Ident, Lit)>,
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

impl CustomAttributes {
    pub fn contain_key(&self, key: &str) -> bool {
        self.identifiers.iter().any(|ident| ident.to_string() == key)   
    }

    pub fn get_meta_value(&self, key: &str) -> Option<Lit> {
        for p in self.ident_lit_pairs.clone() {
            if p.0.to_string() == key {
                return Some(p.1);
            }
        }
        None
    }
}