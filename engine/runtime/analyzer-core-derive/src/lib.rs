//! Derive macro for the DataSize trait.
//!
//! This crate provides a derive macro for automatically implementing
//! the `DataSize` trait for structs and enums.
//!
//! # Example
//!
//! ```ignore
//! use reearth_flow_analyzer_core::DataSize;
//!
//! #[derive(DataSize)]
//! struct MyStruct {
//!     name: String,
//!     values: Vec<i32>,
//!     count: usize,
//! }
//! ```
//!
//! # Attributes
//!
//! - `#[data_size(skip)]` - Skip this field when calculating data size
//!
//! ```ignore
//! #[derive(DataSize)]
//! struct MyStruct {
//!     data: Vec<u8>,
//!     #[data_size(skip)]
//!     cached_size: usize, // Not counted in data size
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericParam, Generics};

/// Derive macro for the `DataSize` trait.
///
/// Automatically implements data size for structs and enums.
#[proc_macro_derive(DataSize, attributes(data_size))]
pub fn derive_data_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add DataSize bounds to all type parameters
    let where_clause = add_datasize_bounds(generics, where_clause);

    let data_size = match &input.data {
        Data::Struct(data_struct) => generate_struct_impl(&data_struct.fields),
        Data::Enum(data_enum) => generate_enum_impl(data_enum),
        Data::Union(_) => panic!("DataSize cannot be derived for unions"),
    };

    let expanded = quote! {
        impl #impl_generics reearth_flow_analyzer_core::DataSize for #name #ty_generics #where_clause {
            fn data_size(&self) -> usize {
                #data_size
            }
        }
    };

    TokenStream::from(expanded)
}

fn add_datasize_bounds(
    generics: &Generics,
    where_clause: Option<&syn::WhereClause>,
) -> syn::WhereClause {
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });

    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            where_clause
                .predicates
                .push(syn::parse_quote!(#ident: reearth_flow_analyzer_core::DataSize));
        }
    }

    where_clause
}

fn generate_struct_impl(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named) => {
            let field_names: Vec<_> = named
                .named
                .iter()
                .filter(|f| !should_skip_field(f))
                .map(|f| &f.ident)
                .collect();

            if field_names.is_empty() {
                return quote!(0);
            }

            let data_size = quote! {
                0 #(+ reearth_flow_analyzer_core::DataSize::data_size(&self.#field_names))*
            };

            data_size
        }
        Fields::Unnamed(unnamed) => {
            let indices: Vec<_> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .filter(|(_, f)| !should_skip_field(f))
                .map(|(i, _)| syn::Index::from(i))
                .collect();

            if indices.is_empty() {
                return quote!(0);
            }
            let data_size = quote! {
                0 #(+ reearth_flow_analyzer_core::DataSize::data_size(&self.#indices))*
            };

            data_size
        }
        Fields::Unit => quote!(0),
    }
}

fn generate_enum_impl(data_enum: &syn::DataEnum) -> proc_macro2::TokenStream {
    let variant_arms: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            match &variant.fields {
                Fields::Named(named) => {
                    let field_names: Vec<_> = named
                        .named
                        .iter()
                        .filter(|f| !should_skip_field(f))
                        .map(|f| &f.ident)
                        .collect();

                    let all_field_names: Vec<_> =
                        named.named.iter().map(|f| &f.ident).collect();

                    if field_names.is_empty() {
                        quote! {
                            Self::#variant_ident { .. } => 0
                        }
                    } else {
                        quote! {
                            Self::#variant_ident { #(#all_field_names),* } => {
                                0 #(+ reearth_flow_analyzer_core::DataSize::data_size(#field_names))*
                            }
                        }
                    }
                }
                Fields::Unnamed(unnamed) => {
                    let bindings: Vec<_> = (0..unnamed.unnamed.len())
                        .map(|i| quote::format_ident!("f{}", i))
                        .collect();

                    let active_bindings: Vec<_> = unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| !should_skip_field(f))
                        .map(|(i, _)| quote::format_ident!("f{}", i))
                        .collect();

                    if active_bindings.is_empty() {
                        quote! {
                            Self::#variant_ident(..) => 0
                        }
                    } else {
                        quote! {
                            Self::#variant_ident(#(#bindings),*) => {
                                0 #(+ reearth_flow_analyzer_core::DataSize::data_size(#active_bindings))*
                            }
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        Self::#variant_ident => 0
                    }
                }
            }
        })
        .collect();

    let data_size = if variant_arms.is_empty() {
        quote!(0)
    } else {
        quote! {
            match self {
                #(#variant_arms),*
            }
        }
    };

    data_size
}

fn should_skip_field(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("data_size") {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if tokens.contains("skip") {
                    return true;
                }
            }
        }
    }
    false
}
