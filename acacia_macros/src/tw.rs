//! The `tw!` macro for composing Tailwind CSS classes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Token,
};

/// A single item in the tw! macro, which can be:
/// - A simple expression (string literal, variable, etc.)
/// - A conditional: "class" => condition
enum TwItem {
    /// Simple expression that evaluates to something implementing TwClass
    Simple(Expr),
    /// Conditional class: expr => condition
    Conditional { class: Expr, condition: Expr },
}

impl Parse for TwItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;

        // Check if this is a conditional (expr => condition)
        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let condition: Expr = input.parse()?;
            Ok(TwItem::Conditional {
                class: expr,
                condition,
            })
        } else {
            Ok(TwItem::Simple(expr))
        }
    }
}

/// The full tw! macro input: a comma-separated list of TwItems
struct TwInput {
    items: Vec<TwItem>,
}

impl Parse for TwInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            items.push(input.parse()?);

            // Consume optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(TwInput { items })
    }
}

pub fn tw_impl(input: TokenStream) -> TokenStream {
    let TwInput { items } = parse_macro_input!(input as TwInput);

    let parts: Vec<TokenStream2> = items
        .into_iter()
        .map(|item| match item {
            TwItem::Simple(expr) => {
                quote! {
                    acacia_core::tw::TwClass::to_class_str(&#expr)
                }
            }
            TwItem::Conditional { class, condition } => {
                quote! {
                    if #condition {
                        acacia_core::tw::TwClass::to_class_str(&#class)
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {
        {
            let parts: &[Option<&str>] = &[#(#parts),*];
            parts
                .iter()
                .filter_map(|p| *p)
                .collect::<Vec<_>>()
                .join(" ")
        }
    };

    expanded.into()
}
