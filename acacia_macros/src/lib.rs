//! Procedural macros for the Acacia web framework.

use proc_macro::TokenStream;
use quote::quote;

mod form;
mod html;
mod model;
mod route;

#[cfg(feature = "tailwind")]
mod tw;

/// The `html!` macro for writing JSX-like HTML templates.
///
/// # Example
/// ```ignore
/// html! {
///     <div class="container">
///         <h1>{&title}</h1>
///         @for item in &items {
///             <p>{item}</p>
///         }
///     </div>
/// }
/// ```
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    html::html_impl(input)
}

/// Mark a function as a component that returns a Fragment.
///
/// # Example
/// ```ignore
/// #[component]
/// fn MyComponent(name: &str) -> Fragment {
///     html! { <div>Hello, {name}!</div> }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Add allow(non_snake_case) to permit PascalCase component names
    let item2: proc_macro2::TokenStream = item.clone().into();
    let expanded = quote! {
        #[allow(non_snake_case)]
        #item2
    };
    expanded.into()
}

/// Register a page route (GET request that returns a full page).
///
/// # Example
/// ```ignore
/// #[page("/")]
/// async fn home(db: Db) -> Page {
///     html! { <h1>Welcome</h1> }.into_page()
/// }
/// ```
#[proc_macro_attribute]
pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::page_impl(attr, item)
}

/// Register an action route (POST/PUT/DELETE requests).
///
/// # Example
/// ```ignore
/// #[action("/tasks", method = "POST")]
/// async fn create_task(form: Valid<NewTask>, db: Db) -> Fragment {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::action_impl(attr, item)
}

/// Derive the Model trait for database entities.
///
/// # Example
/// ```ignore
/// #[derive(Model)]
/// #[table("tasks")]
/// pub struct Task {
///     #[key]
///     pub id: i32,
///     pub title: String,
///     pub done: bool,
/// }
/// ```
#[proc_macro_derive(Model, attributes(table, key))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive_model_impl(input)
}

/// Derive the Form trait for form handling.
///
/// # Example
/// ```ignore
/// #[derive(Form)]
/// #[for_model(Task)]
/// pub struct NewTask {
///     pub title: String,
/// }
/// ```
#[proc_macro_derive(Form, attributes(for_model))]
pub fn derive_form(input: TokenStream) -> TokenStream {
    form::derive_form_impl(input)
}

/// The `tw!` macro for composing Tailwind CSS classes.
///
/// # Example
/// ```ignore
/// // Static classes
/// tw!["flex", "items-center", "gap-4"]
///
/// // Conditional classes
/// tw!("px-4 py-2", "bg-blue-500" => is_active, "opacity-50" => disabled)
///
/// // With Option<&str>
/// tw!["flex", some_class, None::<&str>]
/// ```
#[cfg(feature = "tailwind")]
#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    tw::tw_impl(input)
}
