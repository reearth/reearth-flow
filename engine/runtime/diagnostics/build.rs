use std::{env, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct RegistryFile {
    codes: Vec<RegistryEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryEntry {
    code: String,
    category: String,
    default_disposition: String,
    message: String,
    help: Option<String>,
}

const CATEGORIES: &[(&str, &str)] = &[
    ("io", "Io"),
    ("parse", "Parse"),
    ("validation", "Validation"),
    ("geometry", "Geometry"),
    ("schema", "Schema"),
    ("expression", "Expression"),
    ("config", "Config"),
    ("network", "Network"),
    ("resource", "Resource"),
    ("internal", "Internal"),
];

const DISPOSITIONS: &[(&str, &str)] = &[
    ("warn_drop", "WarnDrop"),
    ("reject", "Reject"),
    ("fatal", "Fatal"),
];

fn variant_ident(code: &str) -> proc_macro2::Ident {
    let pascal: String = code
        .split(['.', '_'])
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    format_ident!("{pascal}")
}

fn lookup(table: &[(&str, &str)], key: &str, what: &str, code: &str) -> proc_macro2::Ident {
    let (_, variant) = table
        .iter()
        .find(|(k, _)| *k == key)
        .unwrap_or_else(|| panic!("error-code registry: unknown {what} `{key}` for code `{code}`"));
    format_ident!("{variant}")
}

fn main() {
    let registry_dir = PathBuf::from("../../schema/error-codes");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", registry_dir.display());

    let mut entries: Vec<RegistryEntry> = Vec::new();
    let mut paths: Vec<PathBuf> = fs::read_dir(&registry_dir)
        .expect("error-code registry dir engine/schema/error-codes must exist")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    paths.sort();
    for path in paths {
        let raw = fs::read_to_string(&path).unwrap();
        let file: RegistryFile = toml::from_str(&raw)
            .unwrap_or_else(|e| panic!("invalid registry file {}: {e}", path.display()));
        entries.extend(file.codes);
    }

    entries.sort_by(|a, b| a.code.cmp(&b.code));
    for pair in entries.windows(2) {
        if pair[0].code == pair[1].code {
            panic!("duplicate error code `{}` in registry", pair[0].code);
        }
    }
    for entry in &entries {
        let parts: Vec<&str> = entry.code.split('.').collect();
        let valid = parts.len() == 2
            && parts.iter().all(|p| {
                !p.is_empty()
                    && p.chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            });
        if !valid {
            panic!(
                "error code `{}` must match `<domain>.<reason>` in snake_case",
                entry.code
            );
        }
    }

    let variants: Vec<_> = entries.iter().map(|e| variant_ident(&e.code)).collect();
    let strs: Vec<&String> = entries.iter().map(|e| &e.code).collect();
    let categories: Vec<_> = entries
        .iter()
        .map(|e| lookup(CATEGORIES, &e.category, "category", &e.code))
        .collect();
    let dispositions: Vec<_> = entries
        .iter()
        .map(|e| {
            lookup(
                DISPOSITIONS,
                &e.default_disposition,
                "default_disposition",
                &e.code,
            )
        })
        .collect();
    let messages: Vec<&String> = entries.iter().map(|e| &e.message).collect();
    let helps: Vec<TokenStream> = entries
        .iter()
        .map(|e| match &e.help {
            Some(h) => quote! { Some(#h) },
            None => quote! { None },
        })
        .collect();

    let generated = quote! {
        /// Stable, greppable identity, `<domain>.<reason>`. Generated from
        /// `engine/schema/error-codes/*.toml` — edit the registry, not this enum.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum ErrorCode {
            #(
                #[serde(rename = #strs)]
                #variants,
            )*
        }

        impl ErrorCode {
            pub const ALL: &'static [ErrorCode] = &[ #( ErrorCode::#variants, )* ];

            pub fn as_str(&self) -> &'static str {
                match self { #( ErrorCode::#variants => #strs, )* }
            }

            pub fn category(&self) -> crate::types::ErrorCategory {
                match self { #( ErrorCode::#variants => crate::types::ErrorCategory::#categories, )* }
            }

            pub fn default_disposition(&self) -> crate::types::Disposition {
                match self { #( ErrorCode::#variants => crate::types::Disposition::#dispositions, )* }
            }

            pub fn default_message(&self) -> &'static str {
                match self { #( ErrorCode::#variants => #messages, )* }
            }

            pub fn default_help(&self) -> Option<&'static str> {
                match self { #( ErrorCode::#variants => #helps, )* }
            }
        }

        impl std::fmt::Display for ErrorCode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("error_codes.rs");
    fs::write(&out, generated.to_string()).unwrap();
}
