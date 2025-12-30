use biome_deserialize_macros::{Deserializable, Merge};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Attributes that are always targets.
const CLASS_ATTRIBUTES: [&str; 2] = ["class", "className"];

/// Default functions that are always checked for Tailwind classes.
/// Matches eslint-plugin-tailwindcss defaults.
const DEFAULT_FUNCTIONS: &[&str] = &[
    "classnames",
    "classNames",
    "clsx",
    "ctl",
    "cva",
    "tv",
    "twMerge",
    "twJoin",
    "cn",
    "tw",
];

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoUnregisteredTailwindClassesOptions {
    /// Additional attributes to check for Tailwind classes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Box<[Box<str>]>>,

    /// Additional functions to check for Tailwind classes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Box<[Box<str>]>>,

    /// Object keys whose values should be ignored (e.g., "defaultVariants" for cva).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_keys: Option<Box<[Box<str>]>>,

    /// Custom class names or patterns to treat as valid.
    /// Supports exact matches and glob-like patterns with `*` wildcard.
    /// Examples: "custom-class", "my-*", "*-special"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whitelist: Option<Box<[Box<str>]>>,

    /// A regex pattern to match attribute names that contain class strings.
    /// This allows matching attributes like `containerClassName`, `inputClass`, etc.
    /// Example: `"^(class(Name)?|.*[cC]lass(Name)?)$"` matches `class`, `className`,
    /// `containerClassName`, `inputClass`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_regex: Option<Box<str>>,
}

impl NoUnregisteredTailwindClassesOptions {
    /// Check if a function name matches the configured functions
    pub fn has_function(&self, name: &str) -> bool {
        // Check default functions first
        if DEFAULT_FUNCTIONS.contains(&name) {
            return true;
        }
        // Then check user-provided functions
        self.functions.iter().flatten().any(|v| v.as_ref() == name)
    }

    /// Check if a function name matches the configured functions (with wildcard support)
    pub fn match_function(&self, name: &str) -> bool {
        // Check default functions first
        if DEFAULT_FUNCTIONS.contains(&name) {
            return true;
        }
        // Then check user-provided functions with wildcard support
        self.functions.iter().flatten().any(|matcher| {
            let mut matcher_parts = matcher.split('.');
            let mut name_parts = name.split('.');

            let all_parts_match = matcher_parts
                .by_ref()
                .zip(name_parts.by_ref())
                .all(|(m, p)| m == "*" || m == p);

            all_parts_match && matcher_parts.next().is_none() && name_parts.next().is_none()
        })
    }

    /// Check if an attribute name matches the configured attributes
    pub fn has_attribute(&self, name: &str) -> bool {
        CLASS_ATTRIBUTES.contains(&name)
            || self.attributes.iter().flatten().any(|v| v.as_ref() == name)
            || self.matches_class_regex(name)
    }

    /// Check if an attribute name matches the class_regex pattern.
    pub fn matches_class_regex(&self, name: &str) -> bool {
        if let Some(ref pattern) = self.class_regex
            && let Ok(regex) = Regex::new(pattern)
        {
            return regex.is_match(name);
        }
        false
    }

    /// Check if an object key should be ignored
    pub fn has_ignored_key(&self, name: &str) -> bool {
        self.ignored_keys
            .iter()
            .flatten()
            .any(|v| v.as_ref() == name)
    }

    /// Check if a class name is whitelisted
    pub fn is_whitelisted(&self, class_name: &str) -> bool {
        self.whitelist.iter().flatten().any(|pattern| {
            let pattern = pattern.as_ref();
            if pattern.contains('*') {
                // Simple glob matching
                match_glob_pattern(pattern, class_name)
            } else {
                // Exact match
                pattern == class_name
            }
        })
    }
}

/// Simple glob pattern matching supporting `*` as wildcard
fn match_glob_pattern(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcard, exact match
        return pattern == text;
    }

    let mut remaining = text;

    // Check prefix (part before first *)
    if let Some(prefix) = parts.first()
        && !prefix.is_empty()
    {
        if !remaining.starts_with(*prefix) {
            return false;
        }
        remaining = &remaining[prefix.len()..];
    }

    // Check suffix (part after last *)
    if let Some(suffix) = parts.last()
        && !suffix.is_empty()
    {
        if !remaining.ends_with(*suffix) {
            return false;
        }
        remaining = &remaining[..remaining.len() - suffix.len()];
    }

    // Check middle parts
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        if let Some(pos) = remaining.find(*part) {
            remaining = &remaining[pos + part.len()..];
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(match_glob_pattern("my-*", "my-class"));
        assert!(match_glob_pattern("my-*", "my-other"));
        assert!(!match_glob_pattern("my-*", "your-class"));

        assert!(match_glob_pattern("*-class", "my-class"));
        assert!(match_glob_pattern("*-class", "custom-class"));
        assert!(!match_glob_pattern("*-class", "my-button"));

        assert!(match_glob_pattern("*", "anything"));
        assert!(match_glob_pattern("my-*-class", "my-custom-class"));
    }

    #[test]
    fn test_is_whitelisted() {
        let options = NoUnregisteredTailwindClassesOptions {
            whitelist: Some(Box::new([
                "exact-match".into(),
                "prefix-*".into(),
                "*-suffix".into(),
            ])),
            ..Default::default()
        };

        assert!(options.is_whitelisted("exact-match"));
        assert!(options.is_whitelisted("prefix-anything"));
        assert!(options.is_whitelisted("anything-suffix"));
        assert!(!options.is_whitelisted("no-match"));
    }
}
