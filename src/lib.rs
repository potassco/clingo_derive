extern crate proc_macro;

use proc_macro::TokenStream;
use inflector::Inflector;
use quote::quote;
use syn;
use syn::Data::Enum;
use syn::Data::Struct;
use syn::Data::Union;
use syn::Fields::*;
use syn::Type::*;

#[proc_macro_derive(Fact)]
pub fn derive_fact(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).expect("heeh");

    // Build the trait implementation
    impl_fact(&ast)
}

fn impl_fact(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen =
    match &ast.data {
        Struct(data) => {
            match_fields_struct(&data.fields, name)
        },
        Enum(data) => {
            let mut variants = quote! {
                _ => panic!("Unknown Variant"),
            };
            for variant in &data.variants {
                let ident = &variant.ident;
                let bla = match &variant.fields {
                    Named(named_fields) => {
                        println!("Named Fields");
                        let mut tokens = quote! {
                            let mut temp_vec =  vec![];
                        };
                        let mut field_idents =  quote! {};
                        for field in &named_fields.named {
                            let field_ident = field.ident.clone().expect("Expected Some(Ident). None found!");
                            if field_idents.is_empty() {
                                field_idents =  quote! {#field_ident};
                            } else {
                                field_idents =  quote! {#field_idents,#field_ident};
                            }
                            println!("IDENT:{:?}",field_ident);
                            tokens = match &field.ty {
                                BareFn(type_bare_fn) => {panic!("Found BareFn in enum 1.");},
                                Tuple(type_tuple) => {panic!("Found Tuple in enum not yet implemented! 2");},
                                Path(type_path) => {
                                    let segments = &type_path.path.segments;
                                    let typename = segments[0].ident.to_string();
                                    match typename.as_ref() {
                                        "String" => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(Symbol::create_string(&format!("{}", #field_ident))?);
                                            }
                                        },
                                        "bool" | "u8" | "i8" | "u16" | "i16" |"u32" | "i32"  => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(Symbol::create_number(*#field_ident as i32));
                                            }
                                        },
                                        "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                                        _ => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(#field_ident.symbol()?);
                                            }
                                        },
                                    }
                                },

                                Reference(type_reference) => {panic!("Found Reference in enum not yet implemented 1");},
                                _ => {panic!("Unexpected type annotation");}
                            };
                        }

                        let predicate_name = ident.to_string().to_snake_case();
                        quote! {
                            #ident{#field_idents} => {
                                #tokens
                                Symbol::create_function(#predicate_name,&temp_vec,true)
                            },
                        }
                    },
                    Unnamed(unnamed_fields) => {
                        println!("Unnamed Fields");
                        let mut tokens = quote! {
                            let mut temp_vec =  vec![];
                        };
                        let mut field_idents =  quote! {};
                        let predicate_name = ident.to_string().to_snake_case();
                        let mut field_count =1;
                        for field in &unnamed_fields.unnamed {
                            println!("unanmed field: {:?}",field);
                            let field_ident : syn::Ident  = syn::parse_str(&format!("x{}",field_count)).expect("Expected Ident");
                            if field_idents.is_empty() {
                                field_idents =  quote! {#field_ident};
                            } else {
                                field_idents =  quote! {#field_idents,#field_ident};
                            }
                            println!("IDENT NO");
                            tokens = match &field.ty {
                                BareFn(type_bare_fn) => {panic!("Found BareFn in enum.");},
                                Tuple(type_tuple) => {panic!("Found Tuple in enum not yet implemented! 2");},
                                Path(type_path) => {
                                    let segments = &type_path.path.segments;
                                    let typename = segments[0].ident.to_string();
                                    match typename.as_ref() {
                                        "String" => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(Symbol::create_string(&format!("{}", #field_ident))?);
                                            }
                                        },
                                        "bool" | "u8" | "i8" | "u16" | "i16" |"u32" | "i32"  => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(Symbol::create_number(*#field_ident as i32));
                                            }
                                        },
                                        "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                                        _ => {
                                            quote! {
                                                #tokens
                                                temp_vec.push(#field_ident.symbol()?);
                                            }
                                        },
                                    }
                                },

                                Reference(type_reference) => {panic!("Found Reference in enum not yet implemented!");},
                                _ => {panic!("Unexpected type annotation");}
                            };
                            field_count += 1;
                        }
                        quote! {
                            #ident(#field_idents) => {
                                #tokens
                                Symbol::create_function(#predicate_name,&temp_vec,true)    
                            },
                        }
                    },
                    Unit => {
                        println!("No Fields");
                        let predicate_name = ident.to_string().to_snake_case();
                        quote! {
                            #ident => {
                                Symbol::create_id(#predicate_name,true)
                            },
                        }
                    },
                };
                println!("{:?}",ident);
                println!("{:?}",bla.to_string());
                let predicate_name = ident.to_string().to_snake_case();
                variants = quote!{
                    #name::#bla
                    #variants
                }
            }
            let gen = quote! {
                use failure::*;
                impl Fact for #name {
                    fn symbol(&self) -> Result<Symbol, Error> {
                        match self {
                            #variants
                        }
                    }
                }
            };
            gen.into()
        },
        Union(data) => panic!("Cannot derive Fact for Unions!"),
    };
    println!("EXPANDED: \n{}",gen);
    gen
}

fn match_fields_struct(fields: &syn::Fields, name: &syn::Ident) -> TokenStream {
            match fields {
                Named(named_fields) => {
                    let mut tokens = quote! {
                        let mut temp_vec =  vec![];
                    };
                    for field in &named_fields.named {
                        let i = field.ident.clone().expect("Expected Some(Ident). None found!");
                        tokens = match &field.ty {
                            BareFn(type_bare_fn) => {panic!("Found BareFn in struct.");},
                            Tuple(type_tuple) => {panic!("Found Tuple in struct not yet implemented!");},
                            Path(type_path) => {
                                let segments = &type_path.path.segments;
                                let typename = segments[0].ident.to_string();
                                match typename.as_ref() {
                                    "String" => {
                                        quote! {
                                            #tokens
                                            temp_vec.push(Symbol::create_string(&format!("{}", self.#i))?);
                                        }
                                    },
                                    "bool" | "u8" | "i8" | "u16" | "i16" |"u32" | "i32"  => {
                                        quote! {
                                            #tokens
                                            temp_vec.push(Symbol::create_number(self.#i as i32));
                                        }
                                    },
                                    "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                                    _ => {
                                        quote! {
                                            #tokens
                                            temp_vec.push(self.#i.symbol()?);
                                        }
                                    },
                                }
                            },

                            Reference(type_reference) => {panic!("Found Reference in struct not yet implemented");},
                            _ => {panic!("Unexpected type annotation");}
                        };
                    }

                    let predicate_name = name.to_string().to_snake_case();
                    quote! {
                        use failure::*;
                        impl Fact for #name {
                            fn symbol(&self) -> Result<Symbol, Error> {
                                #tokens
                                Symbol::create_function(#predicate_name,&temp_vec,true)
                            }
                        }
                    }
                },
                Unnamed(unnamed_fields) => {
                    let predicate_name = name.to_string().to_snake_case();
                    quote! {
                        use failure::*;
                        impl Fact for #name {
                            fn symbol(&self) -> Result<Symbol, Error> {
                                Symbol::create_function(#predicate_name,&[self.0.symbol().unwrap()],true)
                            }
                        }
                    }

                },
                Unit => {
                    let predicate_name = name.to_string().to_snake_case();
                    quote! {
                        use failure::*;
                        impl Fact for #name {
                            fn symbol(&self) -> Result<Symbol, Error> {
                                Symbol::create_id(#predicate_name,true)
                            }
                        }
                    }
                },
            }.into()
}