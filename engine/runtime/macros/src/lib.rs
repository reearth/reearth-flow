#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use syn::DeriveInput;

fn debug_print_generated(ast: &DeriveInput, toks: &TokenStream) {
    let debug = env::var("FLOW_MACRO_DEBUG");
    if let Ok(s) = debug {
        if s == "1" {
            println!("{toks}");
        }

        if ast.ident == s {
            println!("{toks}");
        }
    }
}

#[proc_macro_derive(PropertySchema)]
pub fn property_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let gen = quote! {
        impl TryFrom<reearth_flow_workflow::graph::NodeProperty> for #name {
            type Error = anyhow::Error;

            fn try_from(node_property: reearth_flow_workflow::graph::NodeProperty) -> Result<Self, anyhow::Error> {
                serde_json::from_value(Value::Object(node_property)).map_err(|e| {
                    anyhow!(
                        "Failed to convert NodeProperty to PropertySchema with {}",
                        e
                    )
                })
            }
        }
    };
    debug_print_generated(&ast, &gen);
    gen.into()
}
