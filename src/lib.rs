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
            match_fields_struct(&data.fields, name, &ast.generics)
        },
        Enum(data) => {
            let mut variants = quote! {
                _ => panic!("Unknown Variant"),
            };
            for variant in &data.variants {
                let ident = &variant.ident;
                let variant = match_fields_enum(&variant.fields, ident);
                variants = quote!{
                    #name::#variant
                    #variants
                }
            }
            let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
            let gen = quote! {
                use failure::*;
                impl #impl_generics Fact for #name #ty_generics #where_clause {
                    fn symbol(&self) -> Result<Symbol, Error> {
                        match self {
                            #variants
                        }
                    }
                }
            };
            gen.into()
        },
        Union(_) => panic!("Cannot derive Fact for Unions!"),
    };
    println!("EXPANDED: \n{}",gen);
    gen
}

fn match_fields_struct(fields: &syn::Fields, name: &syn::Ident, generics: &syn::Generics) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    match fields {
        Named(named_fields) => {
            let mut tokens = quote! {
                let mut temp_vec =  vec![];
            };
            for field in &named_fields.named {
                let i = field.ident.clone().expect("Expected Some(Ident). None found!");
                tokens = match_type_struct(&field.ty, &tokens, i);
            }
            let predicate_name = name.to_string().to_snake_case();
            quote! {
                use failure::*;
                impl #impl_generics Fact for #name #ty_generics #where_clause {
                    fn symbol(&self) -> Result<Symbol, Error> {
                        #tokens
                        Symbol::create_function(#predicate_name,&temp_vec,true)
                    }
                }
            }
        },
        Unnamed(unnamed_fields) => {
            let mut tokens = quote! {
                let mut temp_vec =  vec![];
            };
            let mut field_count = 0;
            for field in &unnamed_fields.unnamed {
                tokens = match_unamed_type_struct(&field.ty, &tokens, field_count);
                field_count += 1;
            }
            let predicate_name = name.to_string().to_snake_case();
            quote! {
                use failure::*;
                impl #impl_generics Fact for #name #ty_generics #where_clause {
                    fn symbol(&self) -> Result<Symbol, Error> {
                        #tokens
                        Symbol::create_function(#predicate_name,&temp_vec,true)
                    }
                }
            }
        },
        Unit => {
            let predicate_name = name.to_string().to_snake_case();
            quote! {
                use failure::*;
                impl #impl_generics Fact for #name #ty_generics #where_clause {
                    fn symbol(&self) -> Result<Symbol, Error> {
                        Symbol::create_id(#predicate_name,true)
                    }
                }
            }
        },
    }.into()
}

fn match_fields_enum(fields: &syn::Fields, ident: &syn::Ident) -> proc_macro2::TokenStream {
    match &fields {
        Named(named_fields) => {
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
                tokens = match_type_enum(&field.ty,&tokens,field_ident);
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
            let mut tokens = quote! {
                let mut temp_vec =  vec![];
            };
            let mut field_idents =  quote! {};
            let predicate_name = ident.to_string().to_snake_case();
            let mut field_count =1;
            for field in &unnamed_fields.unnamed {
                let field_ident : syn::Ident  = syn::parse_str(&format!("x{}",field_count)).expect("Expected Ident");
                if field_idents.is_empty() {
                    field_idents =  quote! {#field_ident};
                } else {
                    field_idents =  quote! {#field_idents,#field_ident};
                }
                tokens = match_type_enum(&field.ty,&tokens,field_ident);
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
            let predicate_name = ident.to_string().to_snake_case();
            quote! {
                #ident => {
                    Symbol::create_id(#predicate_name,true)
                },
            }
        },
    }
}

fn match_type_struct(ty: &syn::Type, tokens: &proc_macro2::TokenStream, i: syn::Ident) -> proc_macro2::TokenStream {
    let gen = match &ty {
        Tuple(_type_tuple) => {
            quote!{
                #tokens
                temp_vec.push(self.#i.symbol()?);
            }
        },
        Path(type_path) => {
            let segments = &type_path.path.segments;
            let typename = segments[0].ident.to_string();
            match typename.as_ref() {
                "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                _ => {
                    quote! {
                        #tokens
                        temp_vec.push(self.#i.symbol()?);
                    }
                },
            }
        },
        Reference(type_reference) => {
            match_type_struct(&type_reference.elem , tokens, i)
        },
        _ => {panic!("Unexpected type annotation");}
    };
    gen
}
fn match_unamed_type_struct(ty: &syn::Type, tokens: &proc_macro2::TokenStream, i: u32) -> proc_macro2::TokenStream {
    let gen = match &ty {
        Tuple(_type_tuple) => {
            quote!{
                #tokens
                temp_vec.push(self.#i.symbol()?);
            }
        },
        Path(type_path) => {
            let segments = &type_path.path.segments;
            let typename = segments[0].ident.to_string();
            match typename.as_ref() {
                "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                _ => {
                    quote! {
                        #tokens
                        temp_vec.push(self.#i.symbol()?);
                    }
                },
            }
        },
        Reference(type_reference) => {
            match_unamed_type_struct(&type_reference.elem , tokens, i)
        },
        _ => {panic!("Unexpected type annotation");}
    };
    gen
}

fn match_type_enum(ty: &syn::Type, tokens: &proc_macro2::TokenStream, i: syn::Ident) -> proc_macro2::TokenStream {
    let gen = match &ty {
        Tuple(_type_tuple) => {    
            quote!{
                #tokens
                temp_vec.push(#i.symbol()?);
            }
        },
        Path(type_path) => {
            let segments = &type_path.path.segments;
            let typename = segments[0].ident.to_string();
            match typename.as_ref() {
                "u64" | "i64" | "u128" | "i128" => panic!("Cannot derive_fact clingo library only support 32bit integers."),
                _ => {
                    quote! {
                        #tokens
                        temp_vec.push(#i.symbol()?);
                    }
                },
            }
        },
        Reference(type_reference) => {
            match_type_enum(&type_reference.elem , tokens, i)
        },
        _ => {panic!("Unexpected type annotation");}
    };
    gen
}