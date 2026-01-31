//! Tailwind CSS class composition utilities.

/// Trait for types that can be converted to a CSS class string.
/// Used by the `tw!` macro to support various input types.
pub trait TwClass {
    /// Convert to an optional class string.
    /// Returns `None` if this value should not contribute a class.
    fn to_class_str(&self) -> Option<&str>;
}

impl TwClass for &str {
    fn to_class_str(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

impl TwClass for String {
    fn to_class_str(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(self.as_str())
        }
    }
}

impl TwClass for &String {
    fn to_class_str(&self) -> Option<&str> {
        if self.is_empty() {
            None
        } else {
            Some(self.as_str())
        }
    }
}

impl<T: TwClass> TwClass for Option<T> {
    fn to_class_str(&self) -> Option<&str> {
        self.as_ref().and_then(|v| v.to_class_str())
    }
}
