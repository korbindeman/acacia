//! Implementation of the #[model] attribute macro.
//!
//! This macro generates SeaORM 2.0 entity definitions from a user-friendly struct syntax.
//!
//! User writes:
//! ```ignore
//! #[model("tasks")]
//! pub struct Task {
//!     #[key]
//!     pub id: i32,
//!     pub title: String,
//!     pub done: bool,
//! }
//! ```
//!
//! Macro generates a SeaORM entity module (`task`) with Entity, Model, ActiveModel, etc.
//! and re-exports `task::Entity` as `Task`.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

/// Attribute macro implementation for #[model("table_name")]
pub fn model_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the table name from the attribute
    let table_name = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as LitStr).value())
    };

    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;

    // Use provided table name or derive from struct name
    let table_name = table_name.unwrap_or_else(|| to_snake_case(&name.to_string()) + "s");

    // Get struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("model attribute only supports structs with named fields"),
        },
        _ => panic!("model attribute only supports structs"),
    };

    // Build the field definitions with SeaORM attributes
    let mut field_defs = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let is_key = field.attrs.iter().any(|attr| attr.path().is_ident("key"));
        let type_str = quote!(#field_type).to_string();

        if is_key {
            field_defs.push(quote! {
                #[sea_orm(primary_key)]
                pub #field_name: #field_type
            });
        } else if type_str == "bool" {
            // Bool fields default to false
            field_defs.push(quote! {
                #[sea_orm(default_value = false)]
                pub #field_name: #field_type
            });
        } else {
            field_defs.push(quote! {
                pub #field_name: #field_type
            });
        }
    }

    // Module name (snake_case of the struct name)
    let mod_name = format_ident!("{}", to_snake_case(&name.to_string()));

    // The attribute macro replaces the struct with a module + re-export
    let expanded = quote! {
        /// Generated SeaORM entity module
        #vis mod #mod_name {
            use sea_orm::entity::prelude::*;
            use serde::{Deserialize, Serialize};

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
            #[sea_orm(table_name = #table_name)]
            pub struct Model {
                #(#field_defs,)*
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}

            /// Create table statement for migrations
            pub fn __create_table_stmt(schema: &::sea_orm::Schema) -> ::sea_orm::sea_query::TableCreateStatement {
                schema.create_table_from_entity(Entity).if_not_exists().to_owned()
            }
        }

        // Register entity for auto-migration
        ::inventory::submit! {
            ::acacia_db::EntityRegistration::new(#mod_name::__create_table_stmt)
        }

        // Re-export the Model with the original name for ergonomic usage:
        // `db.all::<Task>()` and `fn component(task: &Task)`
        #vis use #mod_name::Model as #name;
    };

    expanded.into()
}

// Keep this for the deprecated derive macro
pub fn derive_model_impl(_input: TokenStream) -> TokenStream {
    quote! {
        compile_error!("Use #[model(\"table_name\")] attribute macro instead of #[derive(Model)]");
    }
    .into()
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
