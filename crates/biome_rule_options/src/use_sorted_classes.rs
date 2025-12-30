use biome_deserialize::{
    Deserializable, DeserializableTypes, DeserializableValue, DeserializationContext,
    DeserializationDiagnostic, DeserializationVisitor, TextRange,
};
use biome_rowan::Text;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseSortedClassesOptions {
    /// Additional attributes that will be sorted.
    /// By default, only `class` and `className` are checked.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub attributes: Option<Box<[Box<str>]>>,
    /// Names of functions and tagged templates that will be sorted.
    /// This covers both call expressions like `clsx(...)` and tagged
    /// template literals like `tw\`...\``.
    ///
    /// Defaults include: classnames, clsx, cva, tv, twMerge, cn, tw, etc.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub functions: Option<Box<[Box<str>]>>,
    /// Object keys whose values should be ignored (e.g., "defaultVariants" for cva).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub ignored_keys: Option<Box<[Box<str>]>>,
    /// A regex pattern to match attribute names that contain class strings.
    /// This allows matching attributes like `containerClassName`, `inputClass`, etc.
    /// Example: `"^(class(Name)?|.*[cC]lass(Name)?)$"` matches `class`, `className`,
    /// `containerClassName`, `inputClass`, etc.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub class_regex: Option<Box<str>>,
}
impl biome_deserialize::Merge for UseSortedClassesOptions {
    fn merge_with(&mut self, other: Self) {
        if let Some(attributes) = other.attributes {
            self.attributes = Some(attributes);
        }
        if let Some(functions) = other.functions {
            self.functions = Some(functions);
        }
        if let Some(ignored_keys) = other.ignored_keys {
            self.ignored_keys = Some(ignored_keys);
        }
        if let Some(class_regex) = other.class_regex {
            self.class_regex = Some(class_regex);
        }
    }
}

impl UseSortedClassesOptions {
    pub fn has_function(&self, name: &str) -> bool {
        // Check default functions first
        if DEFAULT_FUNCTIONS.contains(&name) {
            return true;
        }
        // Then check user-provided functions
        self.functions.iter().flatten().any(|v| v.as_ref() == name)
    }

    pub fn match_function(&self, name: &str) -> bool {
        // Check default functions first (exact match for defaults)
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

    pub fn has_ignored_key(&self, name: &str) -> bool {
        self.ignored_keys
            .iter()
            .flatten()
            .any(|v| v.as_ref() == name)
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

const ALLOWED_OPTIONS: &[&str] = &["attributes", "functions", "ignoredKeys", "classRegex"];

impl Deserializable for UseSortedClassesOptions {
    fn deserialize(
        ctx: &mut impl DeserializationContext,
        value: &impl DeserializableValue,
        name: &str,
    ) -> Option<Self> {
        value.deserialize(ctx, UtilityClassSortingOptionsVisitor, name)
    }
}

struct UtilityClassSortingOptionsVisitor;
impl DeserializationVisitor for UtilityClassSortingOptionsVisitor {
    type Output = UseSortedClassesOptions;

    const EXPECTED_TYPE: DeserializableTypes = DeserializableTypes::MAP;

    fn visit_map(
        self,
        ctx: &mut impl DeserializationContext,
        members: impl Iterator<Item = Option<(impl DeserializableValue, impl DeserializableValue)>>,
        _range: TextRange,
        _name: &str,
    ) -> Option<Self::Output> {
        let mut result = UseSortedClassesOptions::default();

        let mut attributes = Vec::new();
        for (key, value) in members.flatten() {
            let Some(key_text) = Text::deserialize(ctx, &key, "") else {
                continue;
            };
            match key_text.text() {
                "attributes" => {
                    if let Some(attributes_option) =
                        Deserializable::deserialize(ctx, &value, &key_text)
                    {
                        attributes.extend::<Vec<Box<str>>>(attributes_option);
                    }
                }
                "functions" => {
                    result.functions = Deserializable::deserialize(ctx, &value, &key_text)
                }
                "ignoredKeys" => {
                    result.ignored_keys = Deserializable::deserialize(ctx, &value, &key_text)
                }
                "classRegex" => {
                    result.class_regex = Deserializable::deserialize(ctx, &value, &key_text)
                }
                unknown_key => ctx.report(DeserializationDiagnostic::new_unknown_key(
                    unknown_key,
                    key.range(),
                    ALLOWED_OPTIONS,
                )),
            }
        }
        result.attributes = if attributes.is_empty() {
            None
        } else {
            Some(attributes.into_boxed_slice())
        };
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_regex_matching() {
        // Test with a regex that matches class/className and any attribute ending with Class or ClassName
        let options = UseSortedClassesOptions {
            class_regex: Some("^(class(Name)?|.*[cC]lass(Name)?)$".into()),
            ..Default::default()
        };

        // Default attributes should match
        assert!(options.has_attribute("class"));
        assert!(options.has_attribute("className"));

        // Regex matches
        assert!(options.has_attribute("containerClassName"));
        assert!(options.has_attribute("inputClass"));
        assert!(options.has_attribute("wrapperClass"));
        assert!(options.has_attribute("buttonClassName"));

        // Non-matching attributes
        assert!(!options.has_attribute("style"));
        assert!(!options.has_attribute("id"));
        assert!(!options.has_attribute("classPrefix")); // doesn't end with Class/ClassName
    }

    #[test]
    fn test_class_regex_no_regex() {
        // Without classRegex, only default and explicit attributes should match
        let options = UseSortedClassesOptions {
            attributes: Some(Box::new(["customAttr".into()])),
            ..Default::default()
        };

        assert!(options.has_attribute("class"));
        assert!(options.has_attribute("className"));
        assert!(options.has_attribute("customAttr"));
        assert!(!options.has_attribute("containerClassName"));
    }

    #[test]
    fn test_class_regex_invalid_regex() {
        // Invalid regex should not cause panic, just not match
        let options = UseSortedClassesOptions {
            class_regex: Some("[invalid".into()), // unclosed bracket
            ..Default::default()
        };

        // Default attributes still work
        assert!(options.has_attribute("class"));
        // Invalid regex doesn't match anything extra
        assert!(!options.has_attribute("containerClassName"));
    }
}
