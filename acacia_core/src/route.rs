//! Route registration for compile-time route collection.

use crate::Method;
use axum::routing::MethodRouter;

/// A registered route definition.
pub struct RouteDefinition {
    pub path: &'static str,
    pub method: Method,
    pub handler: fn() -> MethodRouter<crate::AppState>,
}

impl RouteDefinition {
    pub const fn new(
        path: &'static str,
        method: Method,
        handler: fn() -> MethodRouter<crate::AppState>,
    ) -> Self {
        Self {
            path,
            method,
            handler,
        }
    }
}

inventory::collect!(RouteDefinition);

/// Application state shared across all routes.
#[derive(Clone)]
pub struct AppState {
    pub db: Option<sea_orm::DatabaseConnection>,
}

impl AppState {
    pub fn new() -> Self {
        Self { db: None }
    }

    pub fn with_db(db: sea_orm::DatabaseConnection) -> Self {
        Self { db: Some(db) }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
