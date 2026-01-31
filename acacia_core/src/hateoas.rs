//! HATEOAS builder functions for HTMX attributes.

use std::borrow::Cow;
use std::fmt;

/// HTTP method for endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Patch => write!(f, "PATCH"),
            Method::Delete => write!(f, "DELETE"),
        }
    }
}

/// An endpoint with a path and method.
#[derive(Clone, Debug)]
pub struct Endpoint {
    pub path: Cow<'static, str>,
    pub method: Method,
}

impl Endpoint {
    pub fn new(path: impl Into<String>, method: Method) -> Self {
        Self {
            path: Cow::Owned(path.into()),
            method,
        }
    }

    pub fn get(path: impl Into<String>) -> Self {
        Self::new(path, Method::Get)
    }

    pub fn post(path: impl Into<String>) -> Self {
        Self::new(path, Method::Post)
    }

    pub fn delete(path: impl Into<String>) -> Self {
        Self::new(path, Method::Delete)
    }

    // Const constructors for use in const contexts
    pub const fn get_const(path: &'static str) -> Self {
        Self {
            path: Cow::Borrowed(path),
            method: Method::Get,
        }
    }

    pub const fn post_const(path: &'static str) -> Self {
        Self {
            path: Cow::Borrowed(path),
            method: Method::Post,
        }
    }

    pub const fn delete_const(path: &'static str) -> Self {
        Self {
            path: Cow::Borrowed(path),
            method: Method::Delete,
        }
    }
}

/// HTMX swap strategies.
#[derive(Clone, Copy, Debug, Default)]
pub enum Swap {
    #[default]
    InnerHtml,
    OuterHtml,
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
    Delete,
    None,
}

impl fmt::Display for Swap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Swap::InnerHtml => write!(f, "innerHTML"),
            Swap::OuterHtml => write!(f, "outerHTML"),
            Swap::BeforeBegin => write!(f, "beforebegin"),
            Swap::AfterBegin => write!(f, "afterbegin"),
            Swap::BeforeEnd => write!(f, "beforeend"),
            Swap::AfterEnd => write!(f, "afterend"),
            Swap::Delete => write!(f, "delete"),
            Swap::None => write!(f, "none"),
        }
    }
}

/// HTMX target specification.
#[derive(Clone, Debug)]
pub enum Target {
    This,
    Parent,
    Closest(String),
    Selector(String),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::This => write!(f, "this"),
            Target::Parent => write!(f, "closest li"),  // Common case: list items
            Target::Closest(selector) => write!(f, "closest {}", selector),
            Target::Selector(s) => write!(f, "{}", s),
        }
    }
}

/// Builder for HTMX attributes.
#[derive(Clone, Debug)]
pub struct HtmxAction {
    endpoint: Endpoint,
    target: Option<Target>,
    swap: Option<Swap>,
}

impl HtmxAction {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            target: None,
            swap: None,
        }
    }

    /// Set the target selector for the response.
    pub fn target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the target to a CSS selector and use innerHTML swap.
    pub fn into(mut self, selector: &str) -> Self {
        self.target = Some(Target::Selector(selector.to_string()));
        self
    }

    /// Set the swap strategy.
    pub fn swap(mut self, swap: Swap) -> Self {
        self.swap = Some(swap);
        self
    }

    /// Append to the target element (beforeend swap).
    pub fn append(mut self) -> Self {
        self.swap = Some(Swap::BeforeEnd);
        self
    }

    /// Prepend to the target element (afterbegin swap).
    pub fn prepend(mut self) -> Self {
        self.swap = Some(Swap::AfterBegin);
        self
    }

    /// Build the HTMX attributes as a string.
    pub fn build(&self) -> String {
        let mut attrs = Vec::new();

        // Add the appropriate hx-* attribute based on method
        let method_attr = match self.endpoint.method {
            Method::Get => "hx-get",
            Method::Post => "hx-post",
            Method::Put => "hx-put",
            Method::Patch => "hx-patch",
            Method::Delete => "hx-delete",
        };
        attrs.push(format!("{}=\"{}\"", method_attr, self.endpoint.path));

        // Add target if specified
        if let Some(ref target) = self.target {
            attrs.push(format!("hx-target=\"{}\"", target));
        }

        // Add swap if specified
        if let Some(ref swap) = self.swap {
            attrs.push(format!("hx-swap=\"{}\"", swap));
        }

        attrs.join(" ")
    }
}

impl fmt::Display for HtmxAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build())
    }
}

/// Create a GET request action (for loading content).
pub fn loads(endpoint: Endpoint) -> HtmxAction {
    HtmxAction::new(Endpoint::new(endpoint.path, Method::Get))
}

/// Create a POST request action (for form submissions).
pub fn submits(endpoint: Endpoint) -> HtmxAction {
    HtmxAction::new(endpoint)
}

/// Create a DELETE request action with delete swap.
pub fn removes(endpoint: Endpoint) -> HtmxAction {
    HtmxAction::new(endpoint).swap(Swap::OuterHtml).target(Target::Parent)
}
