//! Acacia - A modern Rust web framework for hypermedia-driven applications.
//!
//! # Example
//! ```ignore
//! use acacia::prelude::*;
//!
//! #[page("/")]
//! async fn home() -> Page {
//!     html! {
//!         <h1>Hello, Acacia!</h1>
//!     }.into_page()
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     Acacia::new()
//!         .serve("0.0.0.0:3000")
//!         .await;
//! }
//! ```

pub mod prelude {
    // Core types
    pub use acacia_core::{
        escape_html, loads, removes, submits, AppError, AppState, Children, Endpoint, Error,
        Fragment, HtmxAction, Method, OptionExt, Page, RenderHtml, Response, Result,
        RouteDefinition, Swap, Target, Valid,
    };

    // Macros
    #[cfg(feature = "tailwind")]
    pub use acacia_macros::tw;
    pub use acacia_macros::{action, component, form, html, model, page, Form};

    // Database
    pub use acacia_db::{Db, Form as FormTrait, MigratePolicy, Set};

    // SeaORM re-exports for entity definitions and queries
    pub use sea_orm::entity::prelude::*;
    pub use sea_orm::IntoActiveModel;

    // Server
    pub use acacia_server::Acacia;

    // Re-export axum extractors
    pub use axum::extract::Path;

    // Re-export serde for derive
    pub use serde::{Deserialize, Serialize};
}

// Re-export sub-crates
pub use acacia_core;
pub use acacia_db;
pub use acacia_macros;
pub use acacia_server;
