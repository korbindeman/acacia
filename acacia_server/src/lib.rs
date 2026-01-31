//! Server module for Acacia, providing the main application builder.

use acacia_core::{AppState, RouteDefinition};
use acacia_db::{Db, MigratePolicy};
use axum::{
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::Database;
use std::net::SocketAddr;

/// HTMX library content (minified).
const HTMX_JS: &str = include_str!("htmx.min.js");

/// The main Acacia application builder.
pub struct Acacia {
    database_url: Option<String>,
    migrate_policy: MigratePolicy,
}

impl Acacia {
    /// Create a new Acacia application.
    pub fn new() -> Self {
        Self {
            database_url: None,
            migrate_policy: MigratePolicy::Auto,
        }
    }

    /// Set the database connection URL.
    pub fn database(mut self, url: &str) -> Self {
        self.database_url = Some(url.to_string());
        self
    }

    /// Set the migration policy.
    pub fn migrate(mut self, policy: MigratePolicy) -> Self {
        self.migrate_policy = policy;
        self
    }

    /// Start serving the application.
    pub async fn serve(self, addr: &str) {
        // Connect to database if configured
        let db_conn = if let Some(url) = &self.database_url {
            let conn = Database::connect(url)
                .await
                .expect("Failed to connect to database");

            // Run migrations if auto
            if matches!(self.migrate_policy, MigratePolicy::Auto) {
                let db = Db::new(conn.clone());
                db.migrate().await.expect("Failed to run migrations");
            }

            Some(conn)
        } else {
            None
        };

        // Build the router with all registered routes
        let mut router = Router::new();

        // Add HTMX serving route
        router = router.route("/__acacia__/htmx.min.js", get(serve_htmx));

        // Add all registered routes
        for route_def in inventory::iter::<RouteDefinition> {
            let handler = (route_def.handler)();
            // Convert Acacia path format {param} to Axum format :param
            let axum_path = route_def.path.replace('{', ":").replace('}', "");
            router = router.route(&axum_path, handler);
        }

        // Create app state
        let state = if let Some(conn) = db_conn {
            AppState::with_db(conn)
        } else {
            AppState::new()
        };

        let app = router.with_state(state);

        // Parse address and serve
        let socket_addr: SocketAddr = addr.parse().expect("Invalid address");
        println!("🌿 Acacia server running at http://{}", socket_addr);

        let listener = tokio::net::TcpListener::bind(socket_addr)
            .await
            .expect("Failed to bind address");

        axum::serve(listener, app)
            .await
            .expect("Server error");
    }
}

impl Default for Acacia {
    fn default() -> Self {
        Self::new()
    }
}

/// Serve the HTMX library.
async fn serve_htmx() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        HTMX_JS,
    )
}
