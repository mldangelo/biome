use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Configuration for Tailwind CSS integration.
///
/// This configuration allows customizing how Biome's Tailwind CSS lint rules
/// work with your project. You can provide custom theme values that will be
/// merged with Tailwind's defaults, or point to a compiled preset file.
///
/// ## Tailwind v4 Support
///
/// For Tailwind v4 projects using CSS-based configuration, you can either:
/// - Let Biome auto-discover your CSS file (looks for `@import "tailwindcss"`)
/// - Explicitly set `cssPath` to your Tailwind CSS entry file
///
/// Biome will parse `@theme` blocks and extract custom theme values automatically.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Deserializable, Merge)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TailwindConfiguration {
    /// Path to a Tailwind v4 CSS file containing `@theme` blocks.
    ///
    /// When set, Biome will parse this CSS file to extract theme values
    /// defined with the `@theme` directive. If the CSS file contains a
    /// `@config` directive pointing to a JS config, that will also be loaded.
    ///
    /// If not set, Biome will attempt to auto-discover a CSS file containing
    /// `@import "tailwindcss"` in common locations.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "cssPath": "./src/app.css"
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_path: Option<String>,

    /// Path to a compiled Tailwind preset JSON file.
    ///
    /// Generate this file using `biome tailwind compile` to support
    /// custom Tailwind configurations. When provided, this preset
    /// takes precedence over the built-in defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_path: Option<String>,

    /// Additional spacing scale values to merge with Tailwind's defaults.
    ///
    /// Keys are the CSS values (e.g., "3.25rem"), and values are the
    /// Tailwind utility suffixes (e.g., "13").
    ///
    /// Example:
    /// ```json
    /// {
    ///   "spacing": {
    ///     "3.25rem": "13",
    ///     "4.5rem": "18"
    ///   }
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<BTreeMap<String, String>>,

    /// Additional opacity scale values to merge with Tailwind's defaults.
    ///
    /// Keys are the CSS values (e.g., "0.15"), and values are the
    /// Tailwind utility suffixes (e.g., "15").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<BTreeMap<String, String>>,

    /// Additional z-index scale values to merge with Tailwind's defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<BTreeMap<String, String>>,

    /// Additional font-size scale values to merge with Tailwind's defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<BTreeMap<String, String>>,

    /// Additional border-radius scale values to merge with Tailwind's defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<BTreeMap<String, String>>,

    /// Additional utility prefixes that support negative values.
    ///
    /// These will be merged with the built-in list of negatable utilities
    /// (m, p, top, left, etc.).
    ///
    /// Example:
    /// ```json
    /// {
    ///   "negatable": ["custom-margin", "custom-offset"]
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negatable: Option<Vec<String>>,

    /// Class patterns to ignore when linting.
    ///
    /// These patterns are matched against class names and can be used
    /// to skip custom classes that Biome doesn't recognize.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "ignoredClasses": ["^custom-", "^legacy-"]
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_classes: Option<Vec<String>>,
}

impl TailwindConfiguration {
    /// Returns the CSS path for Tailwind v4 configuration
    pub fn css_path(&self) -> Option<&str> {
        self.css_path.as_deref()
    }

    /// Returns the preset path if configured
    pub fn preset_path(&self) -> Option<&str> {
        self.preset_path.as_deref()
    }

    /// Returns custom spacing values
    pub fn spacing(&self) -> Option<&BTreeMap<String, String>> {
        self.spacing.as_ref()
    }

    /// Returns custom opacity values
    pub fn opacity(&self) -> Option<&BTreeMap<String, String>> {
        self.opacity.as_ref()
    }

    /// Returns custom z-index values
    pub fn z_index(&self) -> Option<&BTreeMap<String, String>> {
        self.z_index.as_ref()
    }

    /// Returns custom font-size values
    pub fn font_size(&self) -> Option<&BTreeMap<String, String>> {
        self.font_size.as_ref()
    }

    /// Returns custom border-radius values
    pub fn border_radius(&self) -> Option<&BTreeMap<String, String>> {
        self.border_radius.as_ref()
    }

    /// Returns additional negatable utilities
    pub fn negatable(&self) -> Option<&[String]> {
        self.negatable.as_deref()
    }

    /// Returns ignored class patterns
    pub fn ignored_classes(&self) -> Option<&[String]> {
        self.ignored_classes.as_deref()
    }
}
