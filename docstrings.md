# Acacia Documentation Style Guide

This guide defines how to write documentation comments (docstrings) for the Acacia framework. Follow these conventions to ensure consistent, high-quality documentation across all crates.

## Core Principles

1. **Example-first**: Every public item should have at least one example
2. **Contextual**: Explain *when* and *why*, not just *what*
3. **Layered**: Start simple, add detail progressively
4. **Cross-linked**: Reference related items liberally
5. **Tested**: Examples should compile (use `cargo test --doc`)

## Basic Structure

### For Types (Structs, Enums)

```rust
/// Short one-line description ending with a period.
///
/// Longer explanation of what this type represents, when to use it,
/// and how it fits into the broader Acacia architecture. This can be
/// multiple paragraphs.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use acacia::prelude::*;
///
/// let fragment = html! {
///     <div>Hello</div>
/// };
/// ```
///
/// More complex example with context:
///
/// ```rust
/// use acacia::prelude::*;
///
/// #[fragment("/items/{id}")]
/// async fn item_row(Path(id): Path<i32>, db: Db) -> Fragment {
///     let item = db.get::<Item>(id).await?;
///     html! { <tr><td>{&item.name}</td></tr> }
/// }
/// ```
///
/// # See Also
///
/// - [`Page`] - For full page responses
/// - [`Response`] - For flexible response types
/// - [`html!`] - The macro used to create fragments
pub struct Fragment(pub(crate) String);
```

### For Functions and Methods

```rust
/// Short description of what this function does.
///
/// Longer explanation including:
/// - When to use this vs alternatives
/// - Important behavior notes
/// - Performance characteristics (if relevant)
///
/// # Arguments
///
/// * `name` - Description of the parameter
/// * `options` - Description, including valid values
///
/// # Returns
///
/// Description of return value, including error conditions.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the item doesn't exist.
/// Returns [`AppError::Forbidden`] if the user lacks permission.
///
/// # Panics
///
/// Panics if called outside of an async runtime. (Only if applicable)
///
/// # Examples
///
/// ```rust
/// use acacia::prelude::*;
///
/// let item = db.get::<Item>(42).await?;
/// ```
pub async fn get<T: Model>(&self, id: i32) -> Result<Option<T>, AppError> {
    // ...
}
```

### For Traits

```rust
/// Brief description of what this trait represents.
///
/// Explain the contract that implementors must fulfill and when
/// users would implement this trait themselves vs using provided
/// implementations.
///
/// # Implementing This Trait
///
/// ```rust
/// use acacia::prelude::*;
///
/// struct MyCache;
///
/// impl CacheBackend for MyCache {
///     async fn get(&self, key: &str) -> Option<Vec<u8>> {
///         // Your implementation
///         None
///     }
///     
///     async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) {
///         // Your implementation
///     }
/// }
/// ```
///
/// # Provided Implementations
///
/// - [`InMemoryCache`] - Default in-memory cache using moka
/// - [`RedisCache`] - Redis-backed cache (post-MVP)
///
/// # See Also
///
/// - [`CachePolicy`] - Configure caching behavior
/// - [`CacheConfig`] - Framework cache configuration
pub trait CacheBackend: Send + Sync {
    /// Retrieve a value from the cache.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    
    /// Store a value in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Data to store
    /// * `ttl` - Optional time-to-live; `None` means use default or no expiry
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>);
}
```

### For Macros

```rust
/// Creates an HTML fragment using RSX syntax.
///
/// This is the primary way to create HTML in Acacia. The syntax is similar
/// to JSX but with Rust expressions.
///
/// # Syntax
///
/// ## Basic Elements
///
/// ```rust
/// html! {
///     <div class="container">
///         <h1>Title</h1>
///         <p>Paragraph</p>
///     </div>
/// }
/// ```
///
/// ## Rust Expressions
///
/// Use `{expr}` to embed Rust expressions:
///
/// ```rust
/// let name = "World";
/// html! {
///     <h1>Hello, {name}!</h1>
///     <p>2 + 2 = {2 + 2}</p>
/// }
/// ```
///
/// ## Control Flow
///
/// Use `@if`, `@for`, `@match` for control flow:
///
/// ```rust
/// html! {
///     @if user.is_admin() {
///         <button>Delete</button>
///     }
///     
///     <ul>
///         @for item in &items {
///             <li>{&item.name}</li>
///         }
///     </ul>
///     
///     @match status {
///         Status::Active => <span class="green">Active</span>,
///         Status::Inactive => <span class="gray">Inactive</span>,
///     }
/// }
/// ```
///
/// ## Dynamic Attributes
///
/// ```rust
/// html! {
///     <div 
///         class={&dynamic_class}
///         id={format!("item-{}", id)}
///         disabled={is_disabled}
///     >
///         Content
///     </div>
/// }
/// ```
///
/// ## HATEOAS Actions
///
/// ```rust
/// html! {
///     <a {loads(item_detail_url(item.id))}>View</a>
///     <form {submits(create_item_url())}>...</form>
///     <button {removes(delete_item_url(item.id))}>Delete</button>
/// }
/// ```
///
/// # See Also
///
/// - [`tw!`] - Tailwind class helper
/// - [`Fragment`] - The output type
/// - [`component`] - Define reusable components
#[macro_export]
macro_rules! html {
    // ...
}
```

### For Attribute Macros

```rust
/// Marks a function as a page handler.
///
/// Pages are full HTML documents that include the layout. Use this for
/// top-level routes that users navigate to directly.
///
/// # Arguments
///
/// * `path` - The URL path, with optional parameters in `{braces}`
///
/// # Attributes
///
/// * `layout` - Override the default layout (optional)
///
/// # Generated Items
///
/// This macro generates:
/// - An Axum handler registered with the router
/// - A URL builder function: `{name}_url(...)`
///
/// # Examples
///
/// Basic page:
///
/// ```rust
/// #[page("/")]
/// async fn home() -> Page {
///     html! { <h1>Welcome</h1> }.into_page()
/// }
/// ```
///
/// With path parameters:
///
/// ```rust
/// #[page("/users/{id}")]
/// async fn user_profile(Path(id): Path<Uuid>) -> Page {
///     // ...
/// }
///
/// // Generated: user_profile_url(id: Uuid) -> Endpoint
/// ```
///
/// With custom layout:
///
/// ```rust
/// #[page("/admin", layout = admin_layout)]
/// async fn admin_dashboard() -> Page {
///     // ...
/// }
/// ```
///
/// # See Also
///
/// - [`fragment`] - For partial HTML (HTMX responses)
/// - [`action`] - For mutations (POST, PUT, DELETE)
/// - [`layout`] - Define layouts
#[proc_macro_attribute]
pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
    // ...
}
```

### For Derive Macros

```rust
/// Derives database model traits for a struct.
///
/// This macro generates:
/// - SeaORM entity implementation
/// - Query builder methods
/// - Migration metadata
/// - Relationship accessors
///
/// # Table Attributes
///
/// * `#[table("name")]` - Database table name (required)
/// * `#[cache(ttl = "5m")]` - Default cache policy
/// * `#[cache(none)]` - Disable caching
///
/// # Field Attributes
///
/// | Attribute | Description |
/// |-----------|-------------|
/// | `#[key]` | Primary key |
/// | `#[auto]` | Auto-increment/generated |
/// | `#[generated]` | Server-generated (excluded from forms) |
/// | `#[unique]` | Unique constraint |
/// | `#[index]` | Create index |
/// | `#[nullable]` | Allow NULL (use `Option<T>`) |
/// | `#[default(expr)]` | Default value |
/// | `#[max_length(n)]` | Max string length |
/// | `#[precision(p, s)]` | Decimal precision |
/// | `#[references(Model::field)]` | Foreign key |
///
/// # Relationship Attributes
///
/// | Attribute | Description |
/// |-----------|-------------|
/// | `#[belongs_to]` | Many-to-one relationship |
/// | `#[has_one]` | One-to-one relationship |
/// | `#[has_many]` | One-to-many relationship |
/// | `#[many_to_many(through = "table")]` | Many-to-many via junction |
///
/// # Examples
///
/// Basic model:
///
/// ```rust
/// #[derive(Model)]
/// #[table("users")]
/// pub struct User {
///     #[key]
///     #[auto]
///     pub id: i32,
///     
///     #[unique]
///     #[max_length(255)]
///     pub email: String,
///     
///     #[generated]
///     pub created_at: DateTime<Utc>,
/// }
/// ```
///
/// With relationships:
///
/// ```rust
/// #[derive(Model)]
/// #[table("posts")]
/// pub struct Post {
///     #[key]
///     pub id: i32,
///     
///     pub title: String,
///     
///     #[references(User::id)]
///     pub author_id: i32,
///     
///     #[belongs_to]
///     pub author: User,
///     
///     #[has_many]
///     pub comments: Vec<Comment>,
/// }
/// ```
///
/// # See Also
///
/// - [`Form`] - Derive form handling
/// - [`Db`] - Query the database
/// - [`MigratePolicy`] - Migration configuration
#[proc_macro_derive(Model, attributes(table, cache, key, auto, generated, ...))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    // ...
}
```

## Module Documentation

Each module should have a `//!` doc comment at the top:

```rust
//! # Authentication
//!
//! This module provides passkey-first authentication for Acacia applications.
//!
//! ## Quick Start
//!
//! Enable authentication in your app:
//!
//! ```rust
//! Acacia::new()
//!     .auth(AuthConfig::passkey())
//!     .build()
//! ```
//!
//! Protect routes with `#[authenticated]`:
//!
//! ```rust
//! #[page("/dashboard")]
//! #[authenticated]
//! async fn dashboard(user: CurrentUser) -> Page {
//!     html! { <h1>Hello, {&user.email}</h1> }.into_page()
//! }
//! ```
//!
//! ## Architecture
//!
//! Acacia uses WebAuthn for passwordless authentication:
//!
//! 1. User registers with email
//! 2. Browser creates a passkey (Face ID, Touch ID, etc.)
//! 3. Public key stored in database
//! 4. Login verifies signature against stored key
//!
//! ## Fallback Methods
//!
//! For browsers without passkey support:
//!
//! - **Magic Link**: Email a one-time login link
//! - **OTP**: Email a one-time code
//!
//! ## Modules
//!
//! - [`passkey`] - WebAuthn implementation
//! - [`session`] - Session management
//! - [`guards`] - Route protection

pub mod passkey;
pub mod session;
pub mod guards;
```

## Enum Documentation

Document each variant:

```rust
/// Application error type with automatic HTTP status mapping.
///
/// All handler errors should be convertible to `AppError`. The framework
/// automatically renders appropriate responses based on request context
/// (HTMX fragment, full page, or JSON).
///
/// # Automatic Conversion
///
/// Common error types implement `Into<AppError>`:
///
/// ```rust
/// // Database errors
/// let item = db.get::<Item>(id).await?; // DbErr -> AppError
///
/// // Option -> NotFound
/// let item = item.ok_or(AppError::NotFound)?;
/// ```
///
/// # Custom Errors
///
/// For application-specific errors:
///
/// ```rust
/// #[derive(Debug, thiserror::Error)]
/// pub enum MyError {
///     #[error("Insufficient funds")]
///     InsufficientFunds,
/// }
///
/// impl From<MyError> for AppError {
///     fn from(err: MyError) -> Self {
///         AppError::BadRequest(err.to_string())
///     }
/// }
/// ```
pub enum AppError {
    /// Resource not found (HTTP 404).
    ///
    /// Use when a requested item doesn't exist:
    ///
    /// ```rust
    /// let item = db.get::<Item>(id).await?
    ///     .ok_or(AppError::NotFound)?;
    /// ```
    NotFound,

    /// Invalid request data (HTTP 400).
    ///
    /// Use for malformed requests that aren't validation errors:
    ///
    /// ```rust
    /// if !is_valid_format(&input) {
    ///     return Err(AppError::BadRequest("Invalid format".into()));
    /// }
    /// ```
    BadRequest(String),

    /// Authentication required (HTTP 401).
    ///
    /// Returned automatically by `#[authenticated]` when no session exists.
    /// For HTMX requests, returns a redirect to login.
    Unauthorized,

    /// Permission denied (HTTP 403).
    ///
    /// User is authenticated but lacks permission:
    ///
    /// ```rust
    /// if !user.can_edit(&item) {
    ///     return Err(AppError::Forbidden);
    /// }
    /// ```
    Forbidden,

    /// Validation errors (HTTP 422).
    ///
    /// For form validation failures. Renders with field-level errors:
    ///
    /// ```rust
    /// let mut errors = ValidationErrors::new();
    /// if db.exists::<User>(User::email.eq(&form.email)).await? {
    ///     errors.add_field("email", "Already registered");
    /// }
    /// if !errors.is_empty() {
    ///     return Err(AppError::ValidationFailed(errors));
    /// }
    /// ```
    ValidationFailed(ValidationErrors),

    /// Internal server error (HTTP 500).
    ///
    /// For unexpected errors. Details are logged but not shown to users
    /// in production.
    ///
    /// Any error implementing `std::error::Error` can be converted:
    ///
    /// ```rust
    /// let data = serde_json::from_str(&input)?; // Converts to Internal
    /// ```
    Internal(anyhow::Error),

    /// Service temporarily unavailable (HTTP 503).
    ///
    /// Use during maintenance or when a dependency is down.
    ServiceUnavailable,
}
```

## Section Headers

Use these standard sections in order:

1. **Short description** (required) - One line, ends with period
2. **Long description** (if needed) - Multiple paragraphs
3. **# Syntax** - For macros, show syntax
4. **# Arguments** / **# Parameters** - For functions
5. **# Returns** - What it returns
6. **# Errors** - Error conditions
7. **# Panics** - Panic conditions (only if it can panic)
8. **# Safety** - For unsafe functions
9. **# Examples** (required for public items)
10. **# See Also** - Related items

## Code Examples

### Make Examples Compile

```rust
/// # Examples
///
/// ```rust
/// use acacia::prelude::*;
///
/// let fragment = html! { <div>Hello</div> };
/// assert!(!fragment.is_empty());
/// ```
```

### Hide Boilerplate

Use `#` to hide lines that are necessary for compilation but not instructive:

```rust
/// # Examples
///
/// ```rust
/// # use acacia::prelude::*;
/// # async fn example(db: Db) -> Result<(), AppError> {
/// let item = db.get::<Item>(42).await?;
/// # Ok(())
/// # }
/// ```
```

### Mark Non-Runnable Examples

```rust
/// ```rust,ignore
/// // This example requires a database connection
/// let items = db.all::<Item>().await?;
/// ```

/// ```rust,no_run
/// // This compiles but shouldn't be run in tests
/// Acacia::new().serve("0.0.0.0:3000").await;
/// ```

/// ```rust,compile_fail
/// // This intentionally doesn't compile (showing what NOT to do)
/// let x: i32 = "not a number";
/// ```
```

## Linking

### Link to Items

Use backticks with square brackets to link to other items. **Only link to items that actually exist in the crate.**

```rust
/// See [`Fragment`] for partial responses.
/// Use [`html!`] macro to create HTML.
/// Returns [`AppError::NotFound`] if missing.
/// Implements [`CacheBackend`] trait.
```

Rustdoc will warn about broken links during `cargo doc`. Always run `cargo doc --warn broken_intra_doc_links` to catch these.

### Qualified Paths

When linking to items in other modules or crates:

```rust
/// See [`crate::auth::CurrentUser`] for the user type.
/// Uses [`std::time::Duration`] for timeouts.
/// Returns [`sea_orm::DbErr`] on database failure.
```

### Link to Modules

```rust
/// See the [`auth`](crate::auth) module for authentication.
/// Check [`db::query`](crate::db::query) for query building.
```

### External Links

For items outside the crate, use regular markdown links:

```rust
/// See [HTMX documentation](https://htmx.org/docs/) for swap strategies.
/// Based on [WebAuthn spec](https://www.w3.org/TR/webauthn-2/).
```

### Validating Links

Before committing, always run:

```bash
# Check for broken intra-doc links
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Or just warnings without failing
cargo doc --no-deps 2>&1 | grep "warning: unresolved link"
```

## Common Patterns

### Builder Methods

```rust
impl QueryBuilder<T> {
    /// Filters results by a condition.
    ///
    /// Multiple `filter` calls are combined with AND:
    ///
    /// ```rust
    /// db.query::<Item>()
    ///     .filter(Item::price.gt(10.0))
    ///     .filter(Item::stock.gt(0))  // AND price > 10 AND stock > 0
    ///     .all()
    ///     .await?
    /// ```
    ///
    /// For OR conditions, use [`filter_or`](Self::filter_or).
    pub fn filter(self, condition: impl Into<Condition>) -> Self {
        // ...
    }
}
```

### Feature-Gated Items

```rust
/// Redis-backed cache implementation.
///
/// # Feature Flag
///
/// This requires the `redis` feature:
///
/// ```toml
/// acacia = { version = "0.1", features = ["redis"] }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use acacia::cache::RedisCache;
///
/// let cache = RedisCache::new("redis://localhost").await?;
/// ```
#[cfg(feature = "redis")]
pub struct RedisCache {
    // ...
}
```

### Async Functions

```rust
/// Fetches an item by ID.
///
/// # Examples
///
/// ```rust
/// # use acacia::prelude::*;
/// # async fn example(db: Db) -> Result<(), AppError> {
/// let item = db.get::<Item>(42).await?;
/// # Ok(())
/// # }
/// ```
///
/// Note: This is an async function and must be `.await`ed.
pub async fn get<T: Model>(&self, id: impl Into<T::Id>) -> Result<Option<T>, DbError> {
    // ...
}
```

## Checklist

Before submitting code, verify:

- [ ] Every public item has a doc comment
- [ ] Every doc comment has at least one example
- [ ] Examples compile (`cargo test --doc`)
- [ ] Complex items have "See Also" links
- [ ] All links resolve (`RUSTDOCFLAGS="-D warnings" cargo doc`)
- [ ] Errors and panics are documented
- [ ] Module has a `//!` header explaining its purpose

## Acacia Public API Reference

When writing "See Also" sections, only reference items that exist. Here are the public items in Acacia:

### acacia_core

| Item | Kind | Description |
|------|------|-------------|
| `Fragment` | struct | Partial HTML response |
| `Page` | struct | Full page response with layout |
| `Response` | enum | Flexible response type |
| `Endpoint` | struct | Type-safe URL reference |
| `Children` | struct | Component children |
| `Target` | enum | HTMX swap target (This, Parent, Selector) |
| `Swap` | enum | HTMX swap strategy |
| `AppError` | enum | Application error type |
| `ValidationErrors` | struct | Form validation errors |
| `Valid<T>` | struct | Validated form wrapper |

### acacia_macros

| Item | Kind | Description |
|------|------|-------------|
| `html!` | macro | Create HTML fragments |
| `tw!` | macro | Tailwind class helper |
| `component` | attr | Define a component |
| `page` | attr | Define a page handler |
| `fragment` | attr | Define a fragment handler |
| `action` | attr | Define an action handler |
| `layout` | attr | Define a layout |
| `Model` | derive | Database model traits |
| `Form` | derive | Form handling traits |

### acacia_db

| Item | Kind | Description |
|------|------|-------------|
| `Db` | struct | Database connection handle |
| `Query` | struct | Query builder |
| `MigratePolicy` | enum | Migration run policy |
| `Model` | trait | Database model trait |
| `CachePolicy` | enum | Caching behavior |

### acacia_auth

| Item | Kind | Description |
|------|------|-------------|
| `Auth` | struct | Authentication manager |
| `AuthConfig` | struct | Auth configuration |
| `AuthMethod` | enum | Authentication methods |
| `CurrentUser` | struct | Authenticated user extractor |
| `authenticated` | attr | Route protection attribute |
| `guard` | attr | Custom guard attribute |
| `Session` | struct | User session |
| `Credential` | struct | Passkey credential |

### acacia_server

| Item | Kind | Description |
|------|------|-------------|
| `Acacia` | struct | Framework builder |
| `Router` | struct | Route collection |
| `Middleware` | trait | Middleware trait |
| `LogConfig` | struct | Logging configuration |
| `LogFormat` | enum | Log output format |

### acacia_cache

| Item | Kind | Description |
|------|------|-------------|
| `CacheBackend` | trait | Cache implementation trait |
| `InMemoryCache` | struct | Default moka-based cache |
| `CacheConfig` | struct | Cache configuration |

### HATEOAS Functions

These are generated by handler macros and used in templates:

| Pattern | Description |
|---------|-------------|
| `loads(endpoint)` | GET and display |
| `submits(endpoint)` | POST form |
| `replaces(endpoint)` | PUT and swap |
| `removes(endpoint)` | DELETE and remove |
| `{handler}_url(...)` | Generated URL builder |

### Example See Also Sections

```rust
/// # See Also
///
/// - [`Page`] - For full page responses
/// - [`html!`] - The macro to create fragments
/// - [`component`] - Define reusable components
pub struct Fragment;

/// # See Also
///
/// - [`AppError::ValidationFailed`] - Wraps this type
/// - [`Valid`] - Extractor that produces these errors
/// - [`Form`] - Derive macro for form types
pub struct ValidationErrors;

/// # See Also
///
/// - [`CacheBackend`] - Trait this implements
/// - [`CacheConfig`] - Configure caching
/// - [`CachePolicy`] - Per-model cache settings
pub struct InMemoryCache;
```

## Anti-Patterns to Avoid

### Don't Just Repeat the Name

```rust
// Bad
/// The Fragment struct.
pub struct Fragment;

// Good
/// A partial HTML response for HTMX swaps.
pub struct Fragment;
```

### Don't Skip Error Documentation

```rust
// Bad
/// Gets an item.
pub async fn get(&self, id: i32) -> Result<Item, Error>;

// Good
/// Gets an item by ID.
///
/// # Errors
///
/// Returns [`DbError::NotFound`] if no item exists with this ID.
/// Returns [`DbError::Connection`] if the database is unavailable.
pub async fn get(&self, id: i32) -> Result<Item, DbError>;
```

### Don't Write Novels

```rust
// Bad - too verbose
/// This function is used to get an item from the database. It takes an ID
/// parameter which should be an integer representing the primary key of the
/// item you want to retrieve. The function will query the database and return
/// the item if it exists, or an error if it doesn't...

// Good - concise
/// Fetches an item by primary key.
///
/// Returns `None` if not found.
pub async fn get(&self, id: i32) -> Result<Option<Item>, DbError>;
```

### Don't Forget the User's Perspective

```rust
// Bad - implementation-focused
/// Calls SeaORM's find_by_id and maps the result.

// Good - user-focused
/// Fetches an item by ID with automatic caching.
///
/// Results are cached according to the model's `#[cache]` policy.
/// Use [`no_cache()`](Self::no_cache) to bypass.
```
