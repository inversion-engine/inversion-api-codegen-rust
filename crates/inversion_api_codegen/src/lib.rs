#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
//! Tools for generating code related to inversion api implementations

use inversion_api_spec::*;
use proc_macro2::{TokenStream, Ident};
use inflector::Inflector;
use quote::*;

fn gen_one_type(top_tokens: &mut TokenStream, name: Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::Bool { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            quote! {
                #[doc = #doc]
                pub type #name = bool;
            }
        }
        Type::U32 { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            quote! {
                #[doc = #doc]
                pub type #name = u32;
            }
        }
        Type::String { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            quote! {
                #[doc = #doc]
                pub type #name = bool;
            }
        }
        Type::Tuple { doc, content } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            let mut content = (*content).clone();
            content.sort_unstable_by_key(|k| k.index);
            let type_names = content.iter().map(|i| {
                let name = format_ident!("{}{}", name, i.index);
                let res = gen_one_type(top_tokens, name.clone(), &i.content);
                top_tokens.extend(res);
                name
            });
            quote! {
                #[doc = #doc]
                pub type #name = (#(#type_names,)*);
            }
        }
        Type::Struct { doc, content } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            let mut content = (*content)
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<_>>();
            content.sort_unstable_by_key(|k| k.1.index);
            let types = content.iter().map(|i| {
                let tname = i.0;
                let name = format!("{}_{}", name.to_string().to_snake_case(), tname.to_snake_case());
                let name = format_ident!("{}", name.to_pascal_case());
                let res = gen_one_type(top_tokens, name.clone(), &i.1.content);
                top_tokens.extend(res);
                quote! {
                    #tname: #name
                }
            });
            quote! {
                #[doc = #doc]
                pub struct #name { #(#types,)* }
            }
        }
        _ => quote!(),
    }
}

/// Generate inversion api spec types
pub fn generate_types(doc: &IApiSpecDoc) -> TokenStream {
    let spec = &doc.inversion_api_spec;
    let mut top_tokens = TokenStream::new();
    for (name, ty) in spec.types.iter() {
        let name = format_ident!("{}", name.to_pascal_case());
        let res = gen_one_type(&mut top_tokens, name, ty);
        top_tokens.extend(res);
    }
    quote! {
        #top_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        const DATA: &[u8] = br#"{
  "inversionApiSpec": {
    "id": "kwSMYpO3kr5yLvTNR3KR4",
    "title": "Test Api",
    "revision": 0,
    "errorType": "error",
    "unique": true,
    "features": {
      "default": {
        "stablizedRevision": 0
      }
    },
    "unstableFeatures": {},
    "types": {
      "error": {
        "type": "string"
      },
      "callOneParams": {
        "type": "tuple",
        "content": [
            {
                "index": 0,
                "content": { "type": "bool" }
            },
            {
                "index": 1,
                "content": { "type": "u32" }
            }
        ]
      },
      "callTwoParams": {
        "type": "struct",
        "content": {
          "yay": { "index": 0, "content": { "type": "bool" } },
          "age": { "index": 0, "content": { "type": "u32" } }
        }
      }
    },
    "callsOut": {},
    "callsIn": {}
  }
}"#;
        let data = IApiSpecDoc::parse(DATA).unwrap();
        println!("#TESTING#\n{}", generate_types(&data).to_string());
    }
}
