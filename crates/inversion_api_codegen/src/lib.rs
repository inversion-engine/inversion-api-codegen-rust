#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
//! Tools for generating code related to inversion api implementations

use inflector::Inflector;
use inversion_api_spec::*;
use proc_macro2::*;
use quote::*;

/// If rustfmt is available on the path, will attempt to format a TokenStream.
/// Otherwise, just returns `TokenStream::to_string()`.
pub fn maybe_fmt(tokens: TokenStream) -> String {
    let tokens = tokens.to_string();
    let res = (|| {
        let mut rustfmt = std::process::Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        use std::io::Write;
        write!(rustfmt.stdin.take().unwrap(), "{}", &tokens)?;
        let output = rustfmt.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        std::io::Result::Ok(stdout)
    })();
    res.unwrap_or(tokens)
}

fn block_comment_open() -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend(vec![
        TokenTree::Punct(Punct::new('/', Spacing::Joint)),
        TokenTree::Punct(Punct::new('*', Spacing::Alone)),
    ]);
    ts
}

fn block_comment_close() -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend(vec![
        TokenTree::Punct(Punct::new('*', Spacing::Joint)),
        TokenTree::Punct(Punct::new('/', Spacing::Alone)),
    ]);
    ts
}

fn check_gen_primitive(ty: &Type) -> Option<TokenStream> {
    match ty {
        Type::Bool { .. } => Some(quote!(bool)),
        Type::U32 { .. } => Some(quote!(u32)),
        Type::String { .. } => Some(quote!(String)),
        _ => None,
    }
}

fn gen_one_type(name: Ident, ty: &Type) -> TokenStream {
    let mod_name = format_ident!("{}", name.to_string().to_snake_case());
    let mut mod_tokens = TokenStream::new();
    let mut item_tokens = TokenStream::new();
    match ty {
        Type::Bool { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            item_tokens.extend(quote! {
                #[doc = #doc]
                pub type #name = bool;
            });
        }
        Type::U32 { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            item_tokens.extend(quote! {
                #[doc = #doc]
                pub type #name = u32;
            });
        }
        Type::String { doc } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            item_tokens.extend(quote! {
                #[doc = #doc]
                pub type #name = bool;
            });
        }
        Type::Tuple { doc, content } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            let mut content = (*content).clone();
            content.sort_unstable_by_key(|k| k.index);
            let type_names = content.iter().map(|i| {
                let doc = i.content.doc().as_ref().map(|s| s.as_str()).unwrap_or("");
                let name = format_ident!("{}{}", name, i.index);
                let open = block_comment_open();
                let close = block_comment_close();
                if let Some(p) = check_gen_primitive(&i.content) {
                    quote! {
                        #open #doc #close
                        #p,
                    }
                } else {
                    let res = gen_one_type(name.clone(), &i.content);
                    mod_tokens.extend(res);
                    quote! {
                        #open #doc #close
                        #mod_name::#name,
                    }
                }
            });
            item_tokens.extend(quote! {
                #[doc = #doc]
                pub type #name = (#(#type_names)*);
            });
        }
        Type::Struct { doc, content } => {
            let doc = doc.as_ref().map(|s| s.as_str()).unwrap_or("");
            let mut content = (*content).iter().map(|v| v.clone()).collect::<Vec<_>>();
            content.sort_unstable_by_key(|k| k.1.index);
            struct D<'a> {
                doc: &'a str,
                tname: Ident,
                ty: TokenStream,
            }
            let types = content.iter().map(|i| {
                let doc = i.1.content.doc().as_ref().map(|s| s.as_str()).unwrap_or("");
                let tname = format_ident!("{}", i.0.to_snake_case());
                let name = format_ident!("{}", i.0.to_pascal_case());
                if let Some(p) = check_gen_primitive(&i.1.content) {
                    D { doc, tname, ty: p }
                } else {
                    let res = gen_one_type(name.clone(), &i.1.content);
                    mod_tokens.extend(res);
                    D { doc, tname, ty: quote!(#mod_name::#name) }
                }
            }).collect::<Vec<_>>();
            let types1 = types.iter().map(|d| {
                let D { doc, tname, ty } = d;
                quote! {
                    #[doc = #doc]
                    #tname: #ty,
                }
            });
            let types2 = types.iter().map(|d| {
                let D { ty, .. } = d;
                quote! {
                    &#ty,
                }
            });
            let types3 = types.iter().map(|d| {
                let D { tname, .. } = d;
                quote! {
                    &self.#tname,
                }
            });
            item_tokens.extend(quote! {
                #[doc = #doc]
                #[derive(Debug, Clone, PartialEq)]
                pub struct #name {
                    #(#types1)*
                }

                impl ::serde::Serialize for #name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: ::serde::Serializer,
                    {
                        let r: (#(#types2)*) = (#(#types3)*);
                        r.serialize(serializer)
                    }
                }
            });
        }
        _ => (),
    }
    let m = if mod_tokens.is_empty() {
        quote!()
    } else {
        quote! {
            pub mod #mod_name {
                #mod_tokens
            }
        }
    };
    quote! {
        #m
        #item_tokens
    }
}

/// Generate inversion api spec types
pub fn generate_types(doc: &IApiSpecDoc) -> TokenStream {
    let spec = &doc.inversion_api_spec;
    let mut tokens = TokenStream::new();
    for (name, ty) in spec.types.iter() {
        let name = format_ident!("{}", name.to_pascal_case());
        let res = gen_one_type(name, ty);
        tokens.extend(res);
    }
    quote! {
        #tokens
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
        "doc": "error type",
        "type": "string"
      },
      "callOne": {
        "doc": "a tuple type",
        "type": "tuple",
        "content": [
            {
                "index": 0,
                "content": { "doc": "first tuple item", "type": "bool" }
            },
            {
                "index": 1,
                "content": { "doc": "second", "type": "u32" }
            }
        ]
      },
      "callTwo": {
        "doc": "a struct type",
        "type": "struct",
        "content": {
          "yay": { "index": 0, "content": { "doc": "yay", "type": "bool" } },
          "age": { "index": 1, "content": { "doc": "age", "type": "u32" } },
          "sub": {
            "index": 2,
            "content": {
              "doc": "a sub struct",
              "type": "struct",
              "content": {
                "yay": { "index": 0, "content": { "doc": "yay", "type": "bool" } },
                "age": { "index": 1, "content": { "doc": "age", "type": "u32" } }
              }
            }
          }
        }
      }
    },
    "callsOut": {},
    "callsIn": {}
  }
}"#;
        let data = IApiSpecDoc::parse(DATA).unwrap();
        println!("{}", maybe_fmt(generate_types(&data)));
    }
}
