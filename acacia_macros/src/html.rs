//! Implementation of the `html!` macro.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rstml::{
    node::{CustomNode, Node, NodeAttribute, NodeBlock, NodeElement, NodeName},
    Parser, ParserConfig,
};
use std::collections::HashSet;
use syn::Expr;

pub fn html_impl(input: TokenStream) -> TokenStream {
    let self_closed: HashSet<&'static str> = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ]
    .into_iter()
    .collect();

    let config = ParserConfig::new()
        .recover_block(true)
        .always_self_closed_elements(self_closed);
    let parser = Parser::new(config);
    let nodes = match parser.parse_simple(input) {
        Ok(nodes) => nodes,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let output = process_nodes(&nodes);

    let expanded = quote! {
        {
            let mut __html = String::new();
            #output
            ::acacia_core::Fragment::new(__html)
        }
    };

    expanded.into()
}

fn process_nodes<C: CustomNode>(nodes: &[Node<C>]) -> TokenStream2 {
    let mut output = TokenStream2::new();

    for node in nodes {
        let node_output = process_node(node);
        output.extend(node_output);
    }

    output
}

fn process_node<C: CustomNode>(node: &Node<C>) -> TokenStream2 {
    match node {
        Node::Element(element) => process_element(element),
        Node::Text(text) => {
            let value = &text.value;
            quote! {
                __html.push_str(#value);
            }
        }
        Node::RawText(raw) => {
            let value = raw.to_string_best();
            quote! {
                __html.push_str(#value);
            }
        }
        Node::Block(block) => process_block(block),
        Node::Comment(comment) => {
            let value = &comment.value;
            quote! {
                __html.push_str("<!--");
                __html.push_str(#value);
                __html.push_str("-->");
            }
        }
        Node::Doctype(doctype) => {
            let value = &doctype.value;
            quote! {
                __html.push_str("<!DOCTYPE ");
                __html.push_str(#value);
                __html.push_str(">");
            }
        }
        Node::Fragment(fragment) => process_nodes(&fragment.children),
        Node::Custom(_) => TokenStream2::new(),
    }
}

fn process_element<C: CustomNode>(element: &NodeElement<C>) -> TokenStream2 {
    let tag_name = element.open_tag.name.to_string();

    // Check if this is a component (starts with uppercase)
    if tag_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return process_component(element);
    }

    let mut output = TokenStream2::new();

    // Opening tag
    output.extend(quote! {
        __html.push_str("<");
        __html.push_str(#tag_name);
    });

    // Process attributes
    for attr in &element.open_tag.attributes {
        let attr_output = process_attribute(attr);
        output.extend(attr_output);
    }

    // Check for self-closing elements
    let self_closing = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];

    if self_closing.contains(&tag_name.as_str()) {
        output.extend(quote! {
            __html.push_str(" />");
        });
    } else {
        output.extend(quote! {
            __html.push_str(">");
        });

        // Process children
        let children_output = process_nodes(&element.children);
        output.extend(children_output);

        // Closing tag
        output.extend(quote! {
            __html.push_str("</");
            __html.push_str(#tag_name);
            __html.push_str(">");
        });
    }

    output
}

fn process_component<C: CustomNode>(element: &NodeElement<C>) -> TokenStream2 {
    let component_name = &element.open_tag.name;
    let component_ident = match component_name {
        NodeName::Path(path) => &path.path,
        _ => {
            return quote! {
                compile_error!("Invalid component name");
            }
        }
    };

    // Collect component props
    let mut props = Vec::new();
    for attr in &element.open_tag.attributes {
        match attr {
            NodeAttribute::Attribute(attr) => {
                let name = attr.key.to_string();
                if let Some(value) = &attr.value() {
                    let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                    props.push(quote! { #ident: #value });
                }
            }
            NodeAttribute::Block(block) => {
                // Spread attribute - for props passed as a block
                if let Some(expr) = block.try_block() {
                    props.push(quote! { #expr });
                }
            }
        }
    }

    // Call the component function
    quote! {
        {
            let __component_result = #component_ident(#(#props),*);
            __html.push_str(&__component_result.0);
        }
    }
}

fn process_attribute(attr: &NodeAttribute) -> TokenStream2 {
    match attr {
        NodeAttribute::Attribute(attr) => {
            let name = attr.key.to_string();

            if let Some(value) = &attr.value() {
                // Check for boolean attributes
                if name == "checked" || name == "disabled" || name == "selected" || name == "readonly" {
                    // Unwrap the block to get the inner expression for cleaner generated code
                    let condition = unwrap_block_expr(value);
                    return quote! {
                        if #condition {
                            __html.push_str(" ");
                            __html.push_str(#name);
                        }
                    };
                }

                // Regular attribute with value
                quote! {
                    __html.push_str(" ");
                    __html.push_str(#name);
                    __html.push_str("=\"");
                    __html.push_str(&::acacia_core::escape_html(&(#value).to_string()));
                    __html.push_str("\"");
                }
            } else {
                // Boolean attribute without value
                quote! {
                    __html.push_str(" ");
                    __html.push_str(#name);
                }
            }
        }
        NodeAttribute::Block(block) => {
            // Block attribute - this is for spreading HTMX actions like {submits(...)}
            if let Some(expr) = block.try_block() {
                quote! {
                    __html.push_str(" ");
                    __html.push_str(&(#expr).to_string());
                }
            } else {
                TokenStream2::new()
            }
        }
    }
}

/// Unwrap a block expression if it contains a single expression.
fn unwrap_block_expr(expr: &Expr) -> TokenStream2 {
    if let Expr::Block(block) = expr {
        if block.block.stmts.len() == 1 {
            if let syn::Stmt::Expr(inner, None) = &block.block.stmts[0] {
                return quote! { #inner };
            }
        }
    }
    quote! { #expr }
}

fn process_block(block: &NodeBlock) -> TokenStream2 {
    if let Some(valid) = block.try_block() {
        let stmts = &valid.stmts;

        // Check if this is a control flow block (@for, @if)
        if stmts.len() == 1 {
            if let syn::Stmt::Expr(expr, _) = &stmts[0] {
                // Check for @for loop syntax
                if let Expr::ForLoop(for_loop) = expr {
                    let pat = &for_loop.pat;
                    let iter = &for_loop.expr;
                    let body_nodes = parse_block_body(&for_loop.body);

                    return quote! {
                        for #pat in #iter {
                            #body_nodes
                        }
                    };
                }

                // Check for @if syntax
                if let Expr::If(if_expr) = expr {
                    let cond = &if_expr.cond;
                    let then_nodes = parse_block_body(&if_expr.then_branch);
                    let else_branch = if_expr.else_branch.as_ref().map(|(_, else_expr)| {
                        quote! { else {
                            let __else_result: ::acacia_core::Fragment = #else_expr;
                            __html.push_str(&__else_result.0);
                        }}
                    });

                    return quote! {
                        if #cond {
                            #then_nodes
                        }
                        #else_branch
                    };
                }
            }
        }

        // Regular expression block - use RenderHtml trait for proper escaping
        quote! {
            __html.push_str(&::acacia_core::RenderHtml::render_html(&(#(#stmts)*)));
        }
    } else {
        quote! {
            compile_error!("Invalid block in html! macro");
        }
    }
}

fn parse_block_body(block: &syn::Block) -> TokenStream2 {
    // For block bodies, we need to process the statements as html content
    let stmts = &block.stmts;

    // Check if this contains html! macro calls or raw expressions
    if stmts.len() == 1 {
        if let syn::Stmt::Expr(expr, _) = &stmts[0] {
            // If it's a macro call (likely html!), expand it
            if let Expr::Macro(mac) = expr {
                let mac_path = &mac.mac.path;
                let tokens = &mac.mac.tokens;
                return quote! {
                    let __nested = #mac_path!(#tokens);
                    __html.push_str(&__nested.0);
                };
            }

            // Otherwise treat as Fragment
            return quote! {
                let __nested: ::acacia_core::Fragment = #expr;
                __html.push_str(&__nested.0);
            };
        }
    }

    // Multiple statements - join them
    quote! {
        #(
            let __stmt_result: ::acacia_core::Fragment = { #stmts };
            __html.push_str(&__stmt_result.0);
        )*
    }
}
