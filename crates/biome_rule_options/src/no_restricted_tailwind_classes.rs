use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

/// A restricted class pattern configuration
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct RestrictedClassPattern {
    /// The class name or regex pattern to forbid.
    /// When `regex` is true, this is treated as a regular expression.
    pub pattern: Box<str>,
    /// Whether to treat `pattern` as a regular expression.
    /// Defaults to false (exact match).
    #[serde(default, skip_serializing_if = "is_false")]
    pub regex: bool,
    /// Optional custom message to display when this class is found
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Box<str>>,
    /// Optional replacement class to use instead.
    /// If not specified, the class will be removed.
    /// Note: When using regex patterns, replacements are not supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<Box<str>>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoRestrictedTailwindClassesOptions {
    /// List of class patterns that are forbidden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Box<[RestrictedClassPattern]>>,
}
