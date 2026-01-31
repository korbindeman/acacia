//! Implementation of route macros (#[page] and #[action]).

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemFn, LitStr, Token};

struct PageArgs {
    path: LitStr,
}

impl Parse for PageArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        Ok(PageArgs { path })
    }
}

pub fn page_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PageArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &item_fn.sig.ident;
    let fn_vis = &item_fn.vis;
    let fn_block = &item_fn.block;
    let fn_inputs = &item_fn.sig.inputs;
    let fn_output = &item_fn.sig.output;
    let fn_asyncness = &item_fn.sig.asyncness;

    let path = &args.path;
    let handler_name = format_ident!("__acacia_handler_{}", fn_name);

    // Generate SCREAMING_CASE name for the endpoint constant/function
    let endpoint_name = format_ident!("{}", to_screaming_case(&fn_name.to_string()));

    // Extract path parameters from the path string (e.g., "/tasks/{id}" -> ["id"])
    let path_str = path.value();
    let path_params: Vec<String> = path_str
        .split('/')
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();

    // Generate endpoint constant or function based on whether there are path params
    let endpoint_def = if path_params.is_empty() {
        // No params: generate a constant
        quote! {
            #fn_vis const #endpoint_name: ::acacia_core::Endpoint = ::acacia_core::Endpoint::get_const(#path);
        }
    } else {
        // Has params: generate a function
        let url_fn_params: Vec<proc_macro2::TokenStream> = path_params
            .iter()
            .map(|p| {
                let ident = format_ident!("{}", p);
                quote! { #ident: impl std::fmt::Display }
            })
            .collect();

        let mut url_expr = quote! { let mut url = String::new(); };
        let parts: Vec<&str> = path_str.split('{').collect();

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                url_expr.extend(quote! { url.push_str(#part); });
            } else {
                let end_brace = part.find('}').unwrap();
                let param_name = &part[..end_brace];
                let rest = &part[end_brace + 1..];
                let param_ident = format_ident!("{}", param_name);
                url_expr.extend(quote! {
                    url.push_str(&#param_ident.to_string());
                    url.push_str(#rest);
                });
            }
        }
        url_expr.extend(quote! { url });

        quote! {
            #[allow(non_snake_case)]
            #fn_vis fn #endpoint_name(#(#url_fn_params),*) -> ::acacia_core::Endpoint {
                let path = { #url_expr };
                ::acacia_core::Endpoint::get(path)
            }
        }
    };

    let expanded = quote! {
        // The original handler function
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output #fn_block

        // Endpoint constant or function
        #endpoint_def

        // Route handler wrapper
        fn #handler_name() -> ::axum::routing::MethodRouter<::acacia_core::AppState> {
            ::axum::routing::get(#fn_name)
        }

        // Route registration
        ::inventory::submit! {
            ::acacia_core::RouteDefinition::new(
                #path,
                ::acacia_core::Method::Get,
                #handler_name,
            )
        }
    };

    expanded.into()
}

fn to_screaming_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

struct ActionArgs {
    path: LitStr,
    method: Option<String>,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        let mut method = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "method" {
                let value: LitStr = input.parse()?;
                method = Some(value.value());
            }
        }

        Ok(ActionArgs { path, method })
    }
}

pub fn action_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ActionArgs);
    let item_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &item_fn.sig.ident;
    let fn_vis = &item_fn.vis;
    let fn_block = &item_fn.block;
    let fn_inputs = &item_fn.sig.inputs;
    let fn_output = &item_fn.sig.output;
    let fn_asyncness = &item_fn.sig.asyncness;

    let path = &args.path;
    let method_str = args.method.as_deref().unwrap_or("POST");
    let method_upper = method_str.to_uppercase();

    let method_variant = match method_upper.as_str() {
        "GET" => quote! { ::acacia_core::Method::Get },
        "POST" => quote! { ::acacia_core::Method::Post },
        "PUT" => quote! { ::acacia_core::Method::Put },
        "PATCH" => quote! { ::acacia_core::Method::Patch },
        "DELETE" => quote! { ::acacia_core::Method::Delete },
        _ => quote! { ::acacia_core::Method::Post },
    };

    let axum_method = match method_upper.as_str() {
        "GET" => quote! { ::axum::routing::get },
        "POST" => quote! { ::axum::routing::post },
        "PUT" => quote! { ::axum::routing::put },
        "PATCH" => quote! { ::axum::routing::patch },
        "DELETE" => quote! { ::axum::routing::delete },
        _ => quote! { ::axum::routing::post },
    };

    let handler_name = format_ident!("__acacia_handler_{}", fn_name);

    // Generate SCREAMING_CASE name for the endpoint constant/function
    let endpoint_name = format_ident!("{}", to_screaming_case(&fn_name.to_string()));

    // Extract path parameters
    let path_str = path.value();
    let path_params: Vec<String> = path_str
        .split('/')
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();

    // Determine endpoint constructor based on method
    let endpoint_const_constructor = match method_upper.as_str() {
        "DELETE" => quote! { ::acacia_core::Endpoint::delete_const },
        "POST" => quote! { ::acacia_core::Endpoint::post_const },
        "GET" => quote! { ::acacia_core::Endpoint::get_const },
        _ => quote! { ::acacia_core::Endpoint::post_const },
    };

    let endpoint_fn_constructor = match method_upper.as_str() {
        "DELETE" => quote! { ::acacia_core::Endpoint::delete },
        "POST" => quote! { ::acacia_core::Endpoint::post },
        "GET" => quote! { ::acacia_core::Endpoint::get },
        _ => quote! { ::acacia_core::Endpoint::post },
    };

    // Generate endpoint constant or function based on whether there are path params
    let endpoint_def = if path_params.is_empty() {
        // No params: generate a constant
        quote! {
            #fn_vis const #endpoint_name: ::acacia_core::Endpoint = #endpoint_const_constructor(#path);
        }
    } else {
        // Has params: generate a function
        let url_fn_params: Vec<proc_macro2::TokenStream> = path_params
            .iter()
            .map(|p| {
                let ident = format_ident!("{}", p);
                quote! { #ident: impl std::fmt::Display }
            })
            .collect();

        let mut url_expr = quote! { let mut url = String::new(); };
        let parts: Vec<&str> = path_str.split('{').collect();

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                url_expr.extend(quote! { url.push_str(#part); });
            } else {
                let end_brace = part.find('}').unwrap();
                let param_name = &part[..end_brace];
                let rest = &part[end_brace + 1..];
                let param_ident = format_ident!("{}", param_name);
                url_expr.extend(quote! {
                    url.push_str(&#param_ident.to_string());
                    url.push_str(#rest);
                });
            }
        }
        url_expr.extend(quote! { url });

        quote! {
            #[allow(non_snake_case)]
            #fn_vis fn #endpoint_name(#(#url_fn_params),*) -> ::acacia_core::Endpoint {
                let path = { #url_expr };
                #endpoint_fn_constructor(path)
            }
        }
    };

    let expanded = quote! {
        // The original handler function
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output #fn_block

        // Endpoint constant or function
        #endpoint_def

        // Route handler wrapper
        fn #handler_name() -> ::axum::routing::MethodRouter<::acacia_core::AppState> {
            #axum_method(#fn_name)
        }

        // Route registration
        ::inventory::submit! {
            ::acacia_core::RouteDefinition::new(
                #path,
                #method_variant,
                #handler_name,
            )
        }
    };

    expanded.into()
}
