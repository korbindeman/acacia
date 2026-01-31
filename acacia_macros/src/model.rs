//! Implementation of the #[derive(Model)] macro.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, LitStr, Type};

pub fn derive_model_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Find table name from #[table("...")] attribute
    let table_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("table") {
                attr.parse_args::<LitStr>().ok()
            } else {
                None
            }
        })
        .map(|s| s.value())
        .unwrap_or_else(|| name.to_string().to_lowercase() + "s");

    // Get struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Model derive only supports structs with named fields"),
        },
        _ => panic!("Model derive only supports structs"),
    };

    // Find the key field
    let mut key_field: Option<(&Ident, &Type)> = None;
    let mut regular_fields: Vec<(&Ident, &Type)> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let is_key = field.attrs.iter().any(|attr| attr.path().is_ident("key"));

        if is_key {
            key_field = Some((field_name, field_type));
        } else {
            regular_fields.push((field_name, field_type));
        }
    }

    let (key_name, key_type) = key_field.expect("Model must have a #[key] field");

    // Generate field names and types
    let field_names: Vec<_> = regular_fields.iter().map(|(n, _)| *n).collect();
    let field_types: Vec<_> = regular_fields.iter().map(|(_, t)| *t).collect();

    // Active model name
    let active_model_name = format_ident!("{}ActiveModel", name);

    let expanded = quote! {
        // FromRow implementation using sea-orm's FromQueryResult pattern
        impl ::acacia_db::FromRow for #name {
            fn from_row(row: &::sea_orm::QueryResult) -> ::acacia_db::Result<Self> {
                use ::sea_orm::TryGetable;
                Ok(Self {
                    #key_name: row.try_get("", stringify!(#key_name))
                        .map_err(|e| ::acacia_db::DbError::Query(e.to_string()))?,
                    #(
                        #field_names: row.try_get("", stringify!(#field_names))
                            .map_err(|e| ::acacia_db::DbError::Query(e.to_string()))?,
                    )*
                })
            }
        }

        // Model trait implementation
        impl ::acacia_db::Model for #name {
            type Key = #key_type;
            type ActiveModel = #active_model_name;

            fn table_name() -> &'static str {
                #table_name
            }

            fn key(&self) -> Self::Key {
                self.#key_name.clone()
            }
        }

        // Active model for inserts/updates
        #[derive(Default, Clone)]
        pub struct #active_model_name {
            #(pub #field_names: Option<#field_types>,)*
        }

        impl #active_model_name {
            pub fn new() -> Self {
                Self::default()
            }
        }

        // Schema definition for migrations
        impl ::acacia_db::HasSchema for #name {
            fn schema() -> ::acacia_db::TableSchema {
                ::acacia_db::TableSchema {
                    name: #table_name.to_string(),
                    columns: vec![
                        ::acacia_db::ColumnSchema {
                            name: stringify!(#key_name).to_string(),
                            sql_type: <#key_type as ::acacia_db::SqlType>::sql_type(),
                            primary_key: true,
                            auto_increment: true,
                            nullable: false,
                            default: None,
                        },
                        #(
                            ::acacia_db::ColumnSchema {
                                name: stringify!(#field_names).to_string(),
                                sql_type: <#field_types as ::acacia_db::SqlType>::sql_type(),
                                primary_key: false,
                                auto_increment: false,
                                nullable: false,
                                default: <#field_types as ::acacia_db::SqlType>::default_value(),
                            },
                        )*
                    ],
                }
            }
        }

        // Register schema for auto-migration
        ::inventory::submit! {
            ::acacia_db::SchemaRegistration::new(|| <#name as ::acacia_db::HasSchema>::schema())
        }
    };

    expanded.into()
}
