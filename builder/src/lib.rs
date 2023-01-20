use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::process::Command;
use syn::*;

fn get_option<'a>(t: &'a Type) -> Option<(&'a Type, &'a GenericArgument)> {
    match t {
        Type::Path(TypePath { qself: None, path }) => {
            let ps = path.segments.first()?;

            if "Option" == ps.ident.to_string().as_str() {
                let type_parameter = match &ps.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        assert!(args.len() == 1);
                        args.first().unwrap()
                    }
                    PathArguments::Parenthesized(_) | PathArguments::None => panic!(),
                };
                Some((t, type_parameter))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[proc_macro_derive(Builder, attributes(builder, milder, foobar))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    let fields = match &input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    // pub struct CommandBuilder {
    //     executable: Option<String>,
    //     args: Option<Vec<String>>,
    //     env: Option<Vec<String>>,
    //     current_dir: Option<String>,
    // }
    let command_builder = {
        let recurse = fields.named.iter().map(|f| {
            let name = &f.ident;
            if let Some((t, _)) = get_option(&f.ty) {
                quote! {
                    #name: #t
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #name: Option<#ty>
                }
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
    //     ...
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

            if let Some((_, t0)) = get_option(&f.ty) {
                quote! {
                    fn #name(&mut self, #name: #t0) -> &mut Self {
                        self.#name = Some(#name);
                        self
                    }
                }
            } else {
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = Some(#name);
                        self
                    }
                }
            }
        });
        let field_constructors = fields.named.iter().map(|f| {
            let name = f.ident.as_ref();

            if let Some(_) = get_option(&f.ty) {
                quote! {
                    #name: self.#name.to_owned()
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
