use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnDecodeKind {
    Empty,
    Query,
    Form,
    Json,
    Proto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructAttributes {
    kind: EnDecodeKind,
}

fn parse_struct_attributes(ident: &str, attrs: &[syn::Attribute]) -> syn::Result<StructAttributes> {
    let mut struct_attributes = StructAttributes {
        kind: EnDecodeKind::Empty,
    };

    for attr in attrs {
        if !attr.path().is_ident(ident) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("empty") {
                struct_attributes.kind = EnDecodeKind::Empty;
                Ok(())
            } else if meta.path.is_ident("query") {
                struct_attributes.kind = EnDecodeKind::Query;
                Ok(())
            } else if meta.path.is_ident("form") {
                struct_attributes.kind = EnDecodeKind::Form;
                Ok(())
            } else if meta.path.is_ident("json") {
                struct_attributes.kind = EnDecodeKind::Json;
                Ok(())
            } else if meta.path.is_ident("proto") {
                struct_attributes.kind = EnDecodeKind::Proto;
                Ok(())
            } else {
                Err(meta.error(format!(
                    "Unknown struct-level argument to {} attribute. Supported args: empty, query, form, json, proto",
                    ident
                )))
            }
        })?;
    }

    Ok(struct_attributes)
}

#[proc_macro_derive(Encode, attributes(encode))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_encode_macro(&ast)
}

#[proc_macro_derive(Decode, attributes(decode))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_decode_macro(&ast)
}

fn impl_encode_macro(ast: &syn::DeriveInput) -> TokenStream {
    // generates an impl for Encode, from one of a few options:
    // - no-op
    // - Request.query()
    // - Request.form()
    // - Request.json()
    // - protobuf message encoded as base64 in single Request.multipart() form part
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let name = &ast.ident;
    let struct_attributes =
        parse_struct_attributes("encode", &ast.attrs).expect("error parsing encode struct attributes");

    let encode_impl = match struct_attributes.kind {
        EnDecodeKind::Empty => quote! { request },
        EnDecodeKind::Query => quote! { request.query(self) },
        EnDecodeKind::Form => quote! { request.form(self) },
        EnDecodeKind::Json => quote! { request.json(self) },
        EnDecodeKind::Proto => quote! {
            use ::prost::Message;
            use ::base64::Engine;
            let bytes = self.encode_to_vec();
            let encoded = ::base64::prelude::BASE64_STANDARD.encode(bytes);

            let form = ::reqwest::multipart::Form::new().text("input_protobuf_encoded", encoded);

            request.multipart(form)
        },
    };

    let generated = quote! {
        impl #generics ::transport::Encode for #name #generics #where_clause {
            fn encode(&self, request: ::reqwest_middleware::RequestBuilder) -> ::reqwest_middleware::RequestBuilder {
                #encode_impl
            }
        }
    };

    generated.into()
}

fn impl_decode_macro(ast: &syn::DeriveInput) -> TokenStream {
    // generates an impl for Encode, from one of a few options:
    // - no-op
    // - Response.form()
    // - Response.json()
    // - protobuf message decoded from response bytes
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let name = &ast.ident;
    let struct_attributes =
        parse_struct_attributes("decode", &ast.attrs).expect("error parsing decode struct attributes");

    let decode_impl = match struct_attributes.kind {
        EnDecodeKind::Empty => quote! { Ok(Self) },
        EnDecodeKind::Query => quote! { ::anyhow::anyhow!("unsupported kind Query") },
        EnDecodeKind::Form => quote! { ::anyhow::anyhow!("unsupported kind Form") },
        EnDecodeKind::Json => quote! {
            let result: Self = response.json().await?;
            Ok(result)
        },
        EnDecodeKind::Proto => quote! {
            let bytes = response.bytes().await?;
            let result: Self = ::prost::Message::decode(bytes)?;
            Ok(result)
        },
    };

    let generated = quote! {
        impl #generics ::transport::Decode for #name #generics #where_clause {
            async fn decode(response: ::reqwest::Response) -> ::anyhow::Result<Self> {
                #decode_impl
            }
        }
    };

    generated.into()
}
