#![allow(dead_code)]
#![allow(unused_macros)]
use crate::*;

pub struct QId(pub Ident);
impl ToTokens for QId {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
macro_rules! q_id {
    ($t:ty) => {
        QId(format_ident!(stringify!($t)))
    };
    ($($t:tt)*) => {
        QId(format_ident!($($t)*))
    };
}

pub struct QType(pub TokenStream);
impl ToTokens for QType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
macro_rules! q_type {
    ($($t:tt)*) => {
        QType(quote!($($t)*))
    }
}

pub struct QVis(pub TokenStream);
impl ToTokens for QVis {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
macro_rules! q_vis {
    ($($t:tt)*) => {
        QVis(quote!($($t)*))
    }
}

pub struct QDoc(pub TokenStream);
impl ToTokens for QDoc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
macro_rules! q_doc {
    ($l:expr) => {{
        let doc = $l;
        QDoc(quote!(#doc))
    }}
}

pub struct QTypeDef {
    pub doc: QDoc,
    pub vis: QVis,
    pub name: QId,
    pub ty: QType,
}
impl ToTokens for QTypeDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let doc = &self.doc;
        let vis = &self.vis;
        let name = &self.name;
        let ty = &self.ty;
        tokens.append_all(quote! {
            #[doc = #doc]
            #vis type #name = #ty;
        });
    }
}
macro_rules! q_type_def {
    ($doc:expr, $vis:expr, $name:expr, $ty:expr,) => {
        QTypeDef {
            doc: $doc,
            vis: $vis,
            name: $name,
            ty: $ty,
        }
    }
}

pub struct QNamedField {
    pub doc: QDoc,
    pub vis: QVis,
    pub name: QId,
    pub ty: QType,
}
impl ToTokens for QNamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let doc = &self.doc;
        let vis = &self.vis;
        let name = &self.name;
        let ty = &self.ty;
        tokens.append_all(quote! {
            #[doc = #doc]
            #vis #name: #ty
        });
    }
}
macro_rules! q_named_field {
    ($doc:expr, $vis:expr, $name:expr, $ty:expr,) => {
        QNamedField {
            doc: $doc,
            vis: $vis,
            name: $name,
            ty: $ty,
        }
    }
}

pub struct QNamedStruct {
    pub doc: QDoc,
    pub vis: QVis,
    pub name: QId,
    pub flds: Vec<QNamedField>,
}
impl ToTokens for QNamedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let doc = &self.doc;
        let vis = &self.vis;
        let name = &self.name;
        let flds = &self.flds;
        tokens.append_all(quote! {
            #[doc = #doc]
            #vis struct #name {
                #( #flds ),*
            }
        })
    }
}
macro_rules! q_named_struct {
    ($doc:expr, $vis:expr, $name:expr, $( $fld:expr, )*) => {
        QNamedStruct {
            doc: $doc,
            vis: $vis,
            name: $name,
            flds: vec![ $( $fld, )* ],
        }
    }
}
