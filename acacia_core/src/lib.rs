//! Core types for the Acacia web framework.

use axum::response::{Html, IntoResponse};
use http::StatusCode;
use std::fmt;

pub mod hateoas;
pub mod route;

#[cfg(feature = "tailwind")]
pub mod tw;

pub use hateoas::*;
pub use route::*;

/// A raw HTML fragment that can be returned from actions and components.
#[derive(Clone, Debug, Default)]
pub struct Fragment(pub String);

impl Fragment {
    pub fn new(html: String) -> Self {
        Self(html)
    }

    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert this fragment into a full page with the default layout.
    pub fn into_page(self) -> Page {
        Page::new(self.0)
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoResponse for Fragment {
    fn into_response(self) -> axum::response::Response {
        Html(self.0).into_response()
    }
}

impl std::ops::Add for Fragment {
    type Output = Fragment;

    fn add(self, rhs: Fragment) -> Self::Output {
        Fragment(self.0 + &rhs.0)
    }
}

impl std::ops::AddAssign for Fragment {
    fn add_assign(&mut self, rhs: Fragment) {
        self.0.push_str(&rhs.0);
    }
}

impl std::iter::FromIterator<Fragment> for Fragment {
    fn from_iter<I: IntoIterator<Item = Fragment>>(iter: I) -> Self {
        let html: String = iter.into_iter().map(|f| f.0).collect();
        Fragment(html)
    }
}

/// A full HTML page with layout.
#[derive(Clone, Debug)]
pub struct Page(pub String);

impl Page {
    #[cfg(not(feature = "tailwind"))]
    pub fn new(content: String) -> Self {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Acacia App</title>
    <script src="/__acacia__/htmx.min.js"></script>
</head>
<body>
{content}
</body>
</html>"#
        );
        Self(html)
    }

    #[cfg(feature = "tailwind")]
    pub fn new(content: String) -> Self {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Acacia App</title>
    <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
    <script src="/__acacia__/htmx.min.js"></script>
</head>
<body>
{content}
</body>
</html>"#
        );
        Self(html)
    }

    #[cfg(not(feature = "tailwind"))]
    pub fn with_title(content: String, title: &str) -> Self {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <script src="/__acacia__/htmx.min.js"></script>
</head>
<body>
{content}
</body>
</html>"#
        );
        Self(html)
    }

    #[cfg(feature = "tailwind")]
    pub fn with_title(content: String, title: &str) -> Self {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
    <script src="/__acacia__/htmx.min.js"></script>
</head>
<body>
{content}
</body>
</html>"#
        );
        Self(html)
    }
}

impl IntoResponse for Page {
    fn into_response(self) -> axum::response::Response {
        Html(self.0).into_response()
    }
}

/// HTTP response wrapper with HTMX support.
#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl Response {
    pub fn empty() -> Self {
        Self {
            status: StatusCode::OK,
            body: String::new(),
            headers: vec![],
        }
    }

    pub fn html(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::OK,
            body: body.into(),
            headers: vec![],
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let mut response = (self.status, Html(self.body)).into_response();
        for (name, value) in self.headers {
            if let (Ok(name), Ok(value)) = (
                http::header::HeaderName::try_from(name),
                http::header::HeaderValue::try_from(value),
            ) {
                response.headers_mut().insert(name, value);
            }
        }
        response
    }
}

/// Validated form wrapper and extractor.
/// Use this instead of `axum::extract::Form` for cleaner handler signatures.
///
/// # Example
/// ```ignore
/// #[action("/tasks", method = "POST")]
/// async fn create_task(form: Valid<NewTask>, db: Db) -> Result<Fragment> {
///     let task = db.insert::<Task, _>(form.into_inner()).await?;
///     Ok(TaskItem(&task))
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Valid<T>(pub T);

impl<T> Valid<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Implement FromRequest to make Valid<T> work as an axum extractor
#[axum::async_trait]
impl<T, S> axum::extract::FromRequest<S> for Valid<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let axum::extract::Form(value) = axum::extract::Form::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(Valid(value))
    }
}

/// Children passed to a component.
#[derive(Clone, Debug, Default)]
pub struct Children(pub Fragment);

impl Children {
    pub fn new(fragment: Fragment) -> Self {
        Self(fragment)
    }

    pub fn into_fragment(self) -> Fragment {
        self.0
    }
}

/// Escape HTML special characters for safe rendering.
pub fn escape_html(s: &str) -> String {
    html_escape::encode_text(s).to_string()
}

/// Trait for rendering values as HTML.
/// Fragment renders as raw HTML, while other types are escaped.
pub trait RenderHtml {
    fn render_html(&self) -> String;
}

impl RenderHtml for Fragment {
    fn render_html(&self) -> String {
        self.0.clone() // Don't escape - already HTML
    }
}

impl RenderHtml for &Fragment {
    fn render_html(&self) -> String {
        self.0.clone()
    }
}

impl RenderHtml for String {
    fn render_html(&self) -> String {
        escape_html(self)
    }
}

impl RenderHtml for &String {
    fn render_html(&self) -> String {
        escape_html(self)
    }
}

impl RenderHtml for &str {
    fn render_html(&self) -> String {
        escape_html(self)
    }
}

impl RenderHtml for i32 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for i64 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for u32 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for u64 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for f32 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for f64 {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

impl RenderHtml for bool {
    fn render_html(&self) -> String {
        self.to_string()
    }
}

/// Application error type for handlers.
/// Handlers return `Result<T, AppError>` and use `?` for error propagation.
#[derive(Debug)]
pub enum AppError {
    // 4xx Client Errors
    NotFound,
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Conflict(String),

    // 5xx Server Errors
    Internal(String),
    Database(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) | AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::NotFound => "Not found".to_string(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Unauthorized => "Unauthorized".to_string(),
            AppError::Forbidden => "Forbidden".to_string(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::Internal(msg) => msg.clone(),
            AppError::Database(msg) => msg.clone(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = format!(
            r#"<div style="padding: 20px; color: #721c24; background: #f8d7da; border: 1px solid #f5c6cb; border-radius: 4px;">
                <strong>Error:</strong> {}
            </div>"#,
            self.message()
        );
        (status, Html(body)).into_response()
    }
}

/// Result type alias for handler operations.
pub type Result<T> = std::result::Result<T, AppError>;

/// Extension trait for Option to convert to AppError::NotFound
pub trait OptionExt<T> {
    fn or_not_found(self) -> std::result::Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_not_found(self) -> std::result::Result<T, AppError> {
        self.ok_or(AppError::NotFound)
    }
}

// Legacy Error type for internal use
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Database(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
