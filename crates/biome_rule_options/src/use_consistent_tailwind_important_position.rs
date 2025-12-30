use biome_deserialize_macros::{Deserializable, Merge};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Position of the important modifier
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ImportantPosition {
    /// Important modifier at the start: `!text-red-500` (Tailwind v4+ style)
    #[default]
    Start,
    /// Important modifier at the end: `text-red-500!`
    End,
}

impl ImportantPosition {
    pub fn is_start(&self) -> bool {
        matches!(self, ImportantPosition::Start)
    }

    pub fn is_end(&self) -> bool {
        matches!(self, ImportantPosition::End)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ImportantPosition::Start => "start",
            ImportantPosition::End => "end",
        }
    }
}

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
pub struct UseConsistentTailwindImportantPositionOptions {
    /// Where the important modifier should be placed.
    /// - "start" (default): `!text-red-500` (Tailwind v4+ style)
    /// - "end": `text-red-500!`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<ImportantPosition>,

    /// Additional attributes to check for Tailwind classes.
    /// Inherits from useSortedClasses options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Box<[Box<str>]>>,

    /// Additional functions to check for Tailwind classes.
    /// Inherits from useSortedClasses options.
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

impl UseConsistentTailwindImportantPositionOptions {
    /// Get the configured position, defaulting to Start
    pub fn position(&self) -> ImportantPosition {
        self.position.unwrap_or_default()
    }

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
}
