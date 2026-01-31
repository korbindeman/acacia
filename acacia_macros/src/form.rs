//! Implementation of the #[derive(Form)] macro.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

pub fn derive_form_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Find model name from #[for_model(...)] attribute
    let model_name = input.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("for_model") {
            attr.parse_args::<Ident>().ok()
        } else {
            None
        }
    });

    // Get struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Form derive only supports structs with named fields"),
        },
        _ => panic!("Form derive only supports structs"),
    };

    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate InsertableFor implementation if #[for_model] is specified
    let insertable_impl = model_name.map(|model| {
        // Generate code that converts each field to SqlValue
        let field_conversions: Vec<_> = field_names
            .iter()
            .map(|name| {
                quote! {
                    ::acacia_db::SqlValue::from(self.#name)
                }
            })
            .collect();

        let field_name_strs: Vec<_> = field_names.iter().map(|name| name.to_string()).collect();

        quote! {
            impl ::acacia_db::InsertableFor<#model> for #name {
                fn columns_and_values(self) -> (Vec<&'static str>, Vec<::acacia_db::SqlValue>) {
                    (
                        vec![#(#field_name_strs),*],
                        vec![#(#field_conversions),*],
                    )
                }
            }
        }
    });

    let expanded = quote! {
        // Auto-generate Deserialize using serde (required for form parsing)
        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                // Use a helper struct with serde's derive
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

        #insertable_impl
    };

    expanded.into()
}
