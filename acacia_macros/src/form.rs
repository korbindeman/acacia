//! Implementation of the #[form] attribute macro and #[derive(Form)].
//!
//! User writes:
//! ```ignore
//! #[form(Task)]
//! pub struct NewTask {
//!     pub title: String,
//! }
//! ```
//!
//! Macro generates Deserialize impl and IntoActiveModel<task::ActiveModel>.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

/// Attribute macro: #[form(ModelName)]
pub fn form_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let model_name = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as Ident))
    };

    let input = parse_macro_input!(item as DeriveInput);
    generate_form(&input, model_name)
}

/// Derive macro: #[derive(Form)] with optional #[for_model(ModelName)]
pub fn derive_form_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Find model name from #[for_model(...)] attribute
    let model_name = input.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("for_model") {
            attr.parse_args::<Ident>().ok()
        } else {
            None
        }
    });

    generate_form(&input, model_name)
}

fn generate_form(input: &DeriveInput, model_name: Option<Ident>) -> TokenStream {
    let name = &input.ident;
    let vis = &input.vis;

    // Get struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Form only supports structs with named fields"),
        },
        _ => panic!("Form only supports structs"),
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate IntoActiveModel implementation if model is specified
    let into_active_model_impl = model_name.map(|model_name| {
        // The entity module name is snake_case of the model name
        let mod_name = format_ident!("{}", to_snake_case(&model_name.to_string()));

        // Generate the field assignments for ActiveModel
        let field_assignments: Vec<_> = field_names
            .iter()
            .map(|name| {
                quote! {
                    #name: ::sea_orm::ActiveValue::Set(self.#name)
                }
            })
            .collect();

        quote! {
            impl ::sea_orm::IntoActiveModel<#mod_name::ActiveModel> for #name {
                fn into_active_model(self) -> #mod_name::ActiveModel {
                    #mod_name::ActiveModel {
                        #(#field_assignments,)*
                        ..Default::default()
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #vis struct #name {
            #(#vis #field_names: #field_types,)*
        }

        // Auto-generate Deserialize using serde (required for form parsing)
        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                struct Helper {
                    #(#field_names: #field_types,)*
                }

                let helper = Helper::deserialize(deserializer)?;
                Ok(Self {
                    #(#field_names: helper.#field_names,)*
                })
            }
        }

        // Form trait implementation
        impl ::acacia_db::Form for #name {}

        #into_active_model_impl
    };

    expanded.into()
}

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}
