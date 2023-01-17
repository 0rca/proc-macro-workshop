use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::process::Command;
use syn::{parse_macro_input, Data::*, DeriveInput, Fields::*};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let builder_name = format_ident!("{}Builder", struct_name);

    let fields = match &input.data {
        Struct(ref data) => match data.fields {
            Named(ref fields) => fields,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let command_builder = {
        let recurse = fields.named.iter().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! {
                #name: Option<#ty>
            }
        });
        quote! {
            pub struct #builder_name {
                #(#recurse), *
            }
        }
    };

    let command_builder_impl = {
        let setters = fields.named.iter().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        });
        let build_function = fields.named.iter().map(|f| {
            let name = &f.ident;
            quote! {
                let #name = self.#name.clone().unwrap();
            }
        });

        let field_names = fields.named.iter().map(|f| &f.ident);
        quote! {
            impl #builder_name {
                #(#setters)*
                pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                    #(#build_function)*
                    Ok(#struct_name {
                        #(#field_names: #field_names),*
                    })
                }
            }
        }
    };

    let command_impl = {
        let fields = fields.named.iter().map(|f| &f.ident);
        quote! {
            impl #struct_name {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        #(#fields: None),*
                    }
                }
            }
        }
    };

    let expanded = quote! {
        #command_builder
        #command_builder_impl
        #command_impl
    };

    let ts = proc_macro::TokenStream::from(expanded);
    save_and_format(&ts, "builder.rs");
    ts
}

/// saves the token stream into a file, and tries to reformat it
fn save_and_format(ts: &TokenStream, path: &str) {
    fs::write(path, ts.to_string()).unwrap();
    Command::new("rustfmt")
        .arg(path)
        .output()
        .expect("failed to format the generated code");
}
