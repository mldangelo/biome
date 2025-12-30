use biome_deserialize_macros::{Deserializable, Merge};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Options for the `useConsistentTailwindLineWrapping` rule.
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentTailwindLineWrappingOptions {
    /// Maximum line width before wrapping classes.
    /// If not specified, classes are always collapsed to a single line.
    /// If specified, classes will be wrapped to stay within this width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<u16>,

    /// List of attributes that should be considered class containers.
    /// Defaults to `["class", "className"]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Box<[Box<str>]>>,

    /// List of functions that should be considered class containers.
    /// Defaults to common Tailwind CSS helper functions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Box<[Box<str>]>>,

    /// Object keys whose values should be ignored (e.g., "defaultVariants" for cva).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_keys: Option<Box<[Box<str>]>>,

    /// A regex pattern to match attribute names that contain class strings.
    /// This allows matching attributes like `containerClassName`, `inputClass`, etc.
    /// Example: `"^(class(Name)?|.*[cC]lass(Name)?)$"` matches `class`, `className`,
    /// `containerClassName`, `inputClass`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_regex: Option<Box<str>>,
}

const DEFAULT_ATTRIBUTES: &[&str] = &["class", "className"];
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

impl UseConsistentTailwindLineWrappingOptions {
    /// Returns the configured print width, or None for no limit (always single line).
    pub fn print_width(&self) -> Option<u16> {
        self.print_width
    }

    /// Check if the given attribute name matches the configured attributes.
    pub fn has_attribute(&self, name: &str) -> bool {
        if let Some(attributes) = &self.attributes {
            attributes.iter().any(|a| a.as_ref() == name) || self.matches_class_regex(name)
        } else {
            DEFAULT_ATTRIBUTES.contains(&name) || self.matches_class_regex(name)
        }
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

    /// Check if the given function name matches the configured functions.
    pub fn has_function(&self, name: &str) -> bool {
        if let Some(functions) = &self.functions {
            functions.iter().any(|f| f.as_ref() == name)
        } else {
            DEFAULT_FUNCTIONS.contains(&name)
        }
    }

    /// Check if the function name matches (for call expressions).
    pub fn match_function(&self, name: &str) -> bool {
        self.has_function(name)
    }

    /// Check if an object key should be ignored.
    pub fn has_ignored_key(&self, name: &str) -> bool {
        self.ignored_keys
            .iter()
            .flatten()
            .any(|v| v.as_ref() == name)
    }
}
