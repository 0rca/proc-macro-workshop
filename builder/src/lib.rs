use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::fs;
use std::process::Command;
use syn::*;

#[proc_macro_derive(Builder, attributes(builder, milder, foobar))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // let input_ts = quote! {
    //     mod input {
    //         #input
    //     }
    // };

    let fields = match &input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    for f in fields.named.iter() {
        let foo: String = f
            .attrs
            .iter()
            .map(|attr| attr.into_token_stream().to_string())
            .fold("- ".to_string(), |acc, attr| format!("{}, {}", acc, attr));

        println!("{}", foo);
    }

    // pub struct CommandBuilder {
    //     executable: Option<String>,
    //     args: Option<Vec<String>>,
    //     env: Option<Vec<String>>,
    //     current_dir: Option<String>,
    // }
    let command_builder = {
        let recurse = fields.named.iter().map(|f| {
            let name = &f.ident;
            let ty = builder_field_type(f);
            quote! {
                #name: #ty
            }
        });
        quote! {
            pub struct #builder_name {
                #(#recurse), *
            }
        }
    };

    // impl CommandBuilder {
    //     fn executable(&mut self, executable: String) -> &mut Self {
    //         self.executable = Some(executable);
    //         self
    //     }
    //     fn args(&mut self, args: Vec<String>) -> &mut Self {
    //         self.args = Some(args);
    //         self
    //     }
    //     fn env(&mut self, env: Vec<String>) -> &mut Self {
    //         self.env = Some(env);
    //         self
    //     }
    //     fn current_dir(&mut self, current_dir: String) -> &mut Self {
    //         self.current_dir = Some(current_dir);
    //         self
    //     }
    //     pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
    //         Ok(Command {
    //             executable: self.executable.clone().unwrap(),
    //             args: self.args.clone().unwrap(),
    //             env: self.env.clone().unwrap(),
    //             current_dir: self.current_dir.clone(),
    //         })
    //     }
    // }
    let command_builder_impl = {
        let setters = fields.named.iter().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;

            let ty0 = if is_optional(f) {
                quote! {
                    #name: String
                }
            } else {
                quote! {
                    #name: #ty
                }
            };
            quote! {
                fn #name(&mut self, #ty0) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        });
        let field_constructors = fields.named.iter().map(|f| {
            let name = f.ident.as_ref();
            if is_optional(f) {
                quote! {
                    #name: self.#name.clone()
                }
            } else {
                quote! {
                    #name: self.#name.to_owned().unwrap()
                }
            }
        });

        quote! {
            impl #builder_name {
                #(#setters)*

                pub fn arg(&mut self, str: String) -> &mut Self {
                    self
                }

                pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                    Ok(#struct_name {
                        #(#field_constructors),*
                    })
                }
            }
        }
    };

    // impl Command {
    //     pub fn builder() -> CommandBuilder {
    //         CommandBuilder {
    //             executable: None,
    //             args: None,
    //             env: None,
    //             current_dir: None,
    //         }
    //     }
    // }
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

/// Returns true if the field is optional in the original struct
/// For now, it is hard-coded
fn is_optional(field: &Field) -> bool {
    match field.ty {
        Type::Path(ref path) => match path {
            TypePath { qself: None, path } => match path.segments.first() {
                Some(PathSegment { ident, .. }) => ident.to_string() == "Option",
                None => false,
            },
            _ => todo!(),
        },
        _ => todo!(),
    }
}

/// type of field that the builder will have.
fn builder_field_type(field: &Field) -> impl ToTokens {
    let ty = &field.ty;

    if is_optional(field) {
        quote! {
            #ty
        }
    } else {
        quote! {
            Option<#ty>
        }
    }
}
