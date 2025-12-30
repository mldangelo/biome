//! Tailwind CSS v4 theme extraction from CSS `@theme` blocks.
//!
//! This module extracts theme values from Tailwind v4 CSS files that use the
//! `@theme` directive to define design tokens as CSS custom properties.
//!
//! ## Example CSS
//!
//! ```css
//! @import "tailwindcss";
//!
//! @theme {
//!   --color-primary: #3b82f6;
//!   --spacing-18: 4.5rem;
//!   --font-size-2xs: 0.625rem;
//! }
//! ```
//!
//! ## Variable Naming Convention
//!
//! Tailwind v4 uses consistent CSS variable naming:
//! - `--color-*` → colors
//! - `--spacing-*` → spacing
//! - `--font-size-*` → font_size
//! - `--font-*` (non-size) → font_family
//! - `--radius-*`, `--border-radius-*` → border_radius
//! - `--z-*`, `--z-index-*` → z_index
//! - `--opacity-*` → opacity
//! - `--animate-*` → animations
//! - `--shadow-*` → box_shadow
//! - `--breakpoint-*` → breakpoints
//! - `--tracking-*` → letter_spacing
//! - `--leading-*` → line_height
//! - `--transition-*` → transition_duration
//! - `--ease-*` → easing

use biome_css_parser::{CssParserOptions, parse_css};
use biome_css_syntax::{
    AnyCssAtRule, AnyCssDeclarationName, AnyCssDeclarationOrRuleBlock, AnyCssProperty, AnyCssRule,
    CssDeclarationWithSemicolon, CssGenericProperty, CssImportAtRule, CssRoot, TwConfigAtRule,
    TwThemeAtRule,
};
use biome_rowan::{AstNode, SyntaxNodeCast};
use std::collections::BTreeMap;

/// Extracted theme values from a Tailwind v4 CSS file.
#[derive(Debug, Default, Clone)]
pub struct TailwindThemeValues {
    /// Color values (e.g., `--color-primary: #3b82f6` → `primary: #3b82f6`)
    pub colors: BTreeMap<String, String>,
    /// Spacing values (e.g., `--spacing-18: 4.5rem` → `4.5rem: 18`)
    pub spacing: BTreeMap<String, String>,
    /// Font size values (e.g., `--font-size-2xs: 0.625rem` → `0.625rem: 2xs`)
    pub font_size: BTreeMap<String, String>,
    /// Font family values (e.g., `--font-sans: ui-sans-serif` → `sans: ui-sans-serif`)
    pub font_family: BTreeMap<String, String>,
    /// Border radius values (e.g., `--radius-xl: 1rem` → `1rem: xl`)
    pub border_radius: BTreeMap<String, String>,
    /// Z-index values (e.g., `--z-modal: 100` → `100: modal`)
    pub z_index: BTreeMap<String, String>,
    /// Opacity values (e.g., `--opacity-muted: 0.5` → `0.5: muted`)
    pub opacity: BTreeMap<String, String>,
    /// Animation values (e.g., `--animate-spin: spin 1s linear infinite` → `spin: ...`)
    pub animations: BTreeMap<String, String>,
    /// Box shadow values (e.g., `--shadow-lg: 0 10px 15px ...` → `lg: ...`)
    pub box_shadow: BTreeMap<String, String>,
    /// Breakpoint values (e.g., `--breakpoint-sm: 640px` → `sm: 640px`)
    pub breakpoints: BTreeMap<String, String>,
    /// Letter spacing values (e.g., `--tracking-wide: 0.025em` → `wide: 0.025em`)
    pub letter_spacing: BTreeMap<String, String>,
    /// Line height values (e.g., `--leading-relaxed: 1.625` → `relaxed: 1.625`)
    pub line_height: BTreeMap<String, String>,
    /// Transition duration values (e.g., `--transition-fast: 150ms` → `fast: 150ms`)
    pub transition_duration: BTreeMap<String, String>,
    /// Easing function values (e.g., `--ease-in-out: cubic-bezier(...)` → `in-out: ...`)
    pub easing: BTreeMap<String, String>,
    /// Path to a `@config` JS file if present
    pub config_path: Option<String>,
    /// Local CSS file paths from `@import` statements (relative paths only, no URLs)
    pub import_paths: Vec<String>,
}

impl TailwindThemeValues {
    /// Check if any theme values were extracted.
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
            && self.spacing.is_empty()
            && self.font_size.is_empty()
            && self.font_family.is_empty()
            && self.border_radius.is_empty()
            && self.z_index.is_empty()
            && self.opacity.is_empty()
            && self.animations.is_empty()
            && self.box_shadow.is_empty()
            && self.breakpoints.is_empty()
            && self.letter_spacing.is_empty()
            && self.line_height.is_empty()
            && self.transition_duration.is_empty()
            && self.easing.is_empty()
            && self.config_path.is_none()
            && self.import_paths.is_empty()
    }

    /// Merge another TailwindThemeValues into this one.
    /// Values from `other` are added if they don't already exist.
    pub fn merge(&mut self, other: TailwindThemeValues) {
        // Merge maps (existing values take precedence)
        for (k, v) in other.colors {
            self.colors.entry(k).or_insert(v);
        }
        for (k, v) in other.spacing {
            self.spacing.entry(k).or_insert(v);
        }
        for (k, v) in other.font_size {
            self.font_size.entry(k).or_insert(v);
        }
        for (k, v) in other.font_family {
            self.font_family.entry(k).or_insert(v);
        }
        for (k, v) in other.border_radius {
            self.border_radius.entry(k).or_insert(v);
        }
        for (k, v) in other.z_index {
            self.z_index.entry(k).or_insert(v);
        }
        for (k, v) in other.opacity {
            self.opacity.entry(k).or_insert(v);
        }
        for (k, v) in other.animations {
            self.animations.entry(k).or_insert(v);
        }
        for (k, v) in other.box_shadow {
            self.box_shadow.entry(k).or_insert(v);
        }
        for (k, v) in other.breakpoints {
            self.breakpoints.entry(k).or_insert(v);
        }
        for (k, v) in other.letter_spacing {
            self.letter_spacing.entry(k).or_insert(v);
        }
        for (k, v) in other.line_height {
            self.line_height.entry(k).or_insert(v);
        }
        for (k, v) in other.transition_duration {
            self.transition_duration.entry(k).or_insert(v);
        }
        for (k, v) in other.easing {
            self.easing.entry(k).or_insert(v);
        }
        // Keep first config_path
        if self.config_path.is_none() {
            self.config_path = other.config_path;
        }
        // Don't merge import_paths - they're for resolution only
    }
}

/// Extract theme values from CSS content.
///
/// Parses the CSS and extracts values from all `@theme` blocks,
/// mapping CSS custom properties to their respective categories.
pub fn extract_theme_from_css(css_content: &str) -> TailwindThemeValues {
    let options = CssParserOptions {
        tailwind_directives: true,
        ..Default::default()
    };

    let parsed = parse_css(css_content, options);
    let root = parsed.tree();

    extract_theme_from_root(&root)
}

/// Extract theme values from a parsed CSS root.
pub fn extract_theme_from_root(root: &CssRoot) -> TailwindThemeValues {
    let mut values = TailwindThemeValues::default();

    for rule in root.rules() {
        process_rule(&rule, &mut values);
    }

    values
}

/// Process a CSS rule, looking for @theme, @config, and @import directives.
fn process_rule(rule: &AnyCssRule, values: &mut TailwindThemeValues) {
    if let AnyCssRule::CssAtRule(at_rule) = rule
        && let Ok(inner) = at_rule.rule()
    {
        match inner {
            AnyCssAtRule::TwThemeAtRule(theme_rule) => {
                extract_from_theme_rule(&theme_rule, values);
            }
            AnyCssAtRule::TwConfigAtRule(config_rule) => {
                extract_config_path(&config_rule, values);
            }
            AnyCssAtRule::CssImportAtRule(import_rule) => {
                extract_import_path(&import_rule, values);
            }
            _ => {}
        }
    }
}

/// Extract import path from an @import directive.
/// Only extracts local paths (starting with "./" or "../"), not URLs or npm packages.
fn extract_import_path(import_rule: &CssImportAtRule, values: &mut TailwindThemeValues) {
    if let Ok(url) = import_rule.url()
        && let Some(path_str) = get_import_path_string(&url)
    {
        // Only process local paths (relative paths)
        if path_str.starts_with("./") || path_str.starts_with("../") {
            values.import_paths.push(path_str);
        }
    }
}

/// Extract the string path from an import URL.
fn get_import_path_string(url: &biome_css_syntax::AnyCssImportUrl) -> Option<String> {
    match url {
        biome_css_syntax::AnyCssImportUrl::CssUrlFunction(url_func) => {
            // url() syntax: @import url("./file.css")
            if let Some(value) = url_func.value() {
                match value {
                    biome_css_syntax::AnyCssUrlValue::CssString(s) => {
                        if let Ok(token) = s.value_token() {
                            let text = token.text_trimmed();
                            let clean = text.trim_matches('"').trim_matches('\'').to_string();
                            if !clean.is_empty() {
                                return Some(clean);
                            }
                        }
                    }
                    biome_css_syntax::AnyCssUrlValue::CssUrlValueRaw(raw) => {
                        if let Ok(token) = raw.value_token() {
                            let text = token.text_trimmed().to_string();
                            if !text.is_empty() {
                                return Some(text);
                            }
                        }
                    }
                }
            }
            None
        }
        biome_css_syntax::AnyCssImportUrl::CssString(s) => {
            // String syntax: @import "./file.css"
            if let Ok(token) = s.value_token() {
                let text = token.text_trimmed();
                // Remove quotes
                let clean = text.trim_matches('"').trim_matches('\'').to_string();
                if !clean.is_empty() {
                    return Some(clean);
                }
            }
            None
        }
    }
}

/// Extract the config path from a @config directive.
fn extract_config_path(config_rule: &TwConfigAtRule, values: &mut TailwindThemeValues) {
    if let Ok(path) = config_rule.path()
        && let Ok(token) = path.value_token()
    {
        let path_str = token.text_trimmed();
        // Remove quotes from the path
        let clean_path = path_str
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .to_string();
        values.config_path = Some(clean_path);
    }
}

/// Extract theme values from a @theme block.
fn extract_from_theme_rule(theme_rule: &TwThemeAtRule, values: &mut TailwindThemeValues) {
    let Ok(block) = theme_rule.block() else {
        return;
    };

    let AnyCssDeclarationOrRuleBlock::CssDeclarationOrRuleBlock(decl_block) = block else {
        return;
    };

    for item in decl_block.items() {
        if let Some(decl_with_semi) = item.syntax().clone().cast::<CssDeclarationWithSemicolon>()
            && let Ok(decl) = decl_with_semi.declaration()
            && let Ok(AnyCssProperty::CssGenericProperty(prop)) = decl.property()
        {
            extract_property(&prop, values);
        }
    }
}

/// Extract a single property and categorize it.
fn extract_property(prop: &CssGenericProperty, values: &mut TailwindThemeValues) {
    let name = match prop.name() {
        Ok(AnyCssDeclarationName::CssDashedIdentifier(ident)) => {
            if let Ok(token) = ident.value_token() {
                token.text_trimmed().to_string()
            } else {
                return;
            }
        }
        _ => return,
    };

    // Get the value as a string
    let value = prop
        .value()
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    if value.is_empty() {
        return;
    }

    // Categorize based on variable name prefix
    // Note: For Tailwind utilities, we store value → key (e.g., "4.5rem" → "18")
    // because that's how tailwind_utils looks up values
    if let Some(suffix) = name.strip_prefix("--color-") {
        values.colors.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--spacing-") {
        // Store as value → key for lookup
        values.spacing.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--font-size-") {
        values.font_size.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--font-") {
        // Font family (non-size) - e.g., --font-sans, --font-serif
        values.font_family.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--radius-") {
        values.border_radius.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--border-radius-") {
        values.border_radius.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--z-index-") {
        values.z_index.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--z-") {
        values.z_index.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--opacity-") {
        values.opacity.insert(value, suffix.to_string());
    } else if let Some(suffix) = name.strip_prefix("--animate-") {
        // Animation values - e.g., --animate-spin, --animate-bounce
        values.animations.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--shadow-") {
        // Box shadow values - e.g., --shadow-sm, --shadow-lg
        values.box_shadow.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--breakpoint-") {
        // Breakpoint values - e.g., --breakpoint-sm, --breakpoint-md
        values.breakpoints.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--tracking-") {
        // Letter spacing values - e.g., --tracking-tight, --tracking-wide
        values.letter_spacing.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--leading-") {
        // Line height values - e.g., --leading-tight, --leading-relaxed
        values.line_height.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--transition-") {
        // Transition duration values - e.g., --transition-fast
        values.transition_duration.insert(suffix.to_string(), value);
    } else if let Some(suffix) = name.strip_prefix("--ease-") {
        // Easing function values - e.g., --ease-in, --ease-out
        values.easing.insert(suffix.to_string(), value);
    }
}

/// Check if CSS content contains Tailwind v4 imports.
///
/// Looks for `@import "tailwindcss"` or `@import 'tailwindcss'`.
pub fn is_tailwind_v4_css(css_content: &str) -> bool {
    css_content.contains("@import \"tailwindcss\"")
        || css_content.contains("@import 'tailwindcss'")
        || css_content.contains("@import\"tailwindcss\"")
        || css_content.contains("@import'tailwindcss'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_colors() {
        let css = r#"
            @theme {
                --color-primary: #3b82f6;
                --color-secondary: #10b981;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.colors.get("primary"), Some(&"#3b82f6".to_string()));
        assert_eq!(values.colors.get("secondary"), Some(&"#10b981".to_string()));
    }

    #[test]
    fn test_extract_spacing() {
        let css = r#"
            @theme {
                --spacing-18: 4.5rem;
                --spacing-13: 3.25rem;
            }
        "#;

        let values = extract_theme_from_css(css);
        // Spacing is stored as value → key
        assert_eq!(values.spacing.get("4.5rem"), Some(&"18".to_string()));
        assert_eq!(values.spacing.get("3.25rem"), Some(&"13".to_string()));
    }

    #[test]
    fn test_extract_font_size() {
        let css = r#"
            @theme {
                --font-size-2xs: 0.625rem;
                --font-size-lg: 1.125rem;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.font_size.get("0.625rem"), Some(&"2xs".to_string()));
        assert_eq!(values.font_size.get("1.125rem"), Some(&"lg".to_string()));
    }

    #[test]
    fn test_extract_border_radius() {
        let css = r#"
            @theme {
                --radius-xl: 1rem;
                --border-radius-2xl: 1.5rem;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.border_radius.get("1rem"), Some(&"xl".to_string()));
        assert_eq!(values.border_radius.get("1.5rem"), Some(&"2xl".to_string()));
    }

    #[test]
    fn test_extract_z_index() {
        let css = r#"
            @theme {
                --z-modal: 100;
                --z-index-tooltip: 1000;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.z_index.get("100"), Some(&"modal".to_string()));
        assert_eq!(values.z_index.get("1000"), Some(&"tooltip".to_string()));
    }

    #[test]
    fn test_extract_opacity() {
        let css = r#"
            @theme {
                --opacity-muted: 0.5;
                --opacity-disabled: 0.3;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.opacity.get("0.5"), Some(&"muted".to_string()));
        assert_eq!(values.opacity.get("0.3"), Some(&"disabled".to_string()));
    }

    #[test]
    fn test_extract_config_path() {
        let css = r#"
            @config "./tailwind.config.js";
            @theme {
                --color-brand: #ff5722;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.config_path, Some("./tailwind.config.js".to_string()));
    }

    #[test]
    fn test_multiple_theme_blocks() {
        let css = r#"
            @theme {
                --color-primary: #3b82f6;
            }

            @theme {
                --color-secondary: #10b981;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.colors.get("primary"), Some(&"#3b82f6".to_string()));
        assert_eq!(values.colors.get("secondary"), Some(&"#10b981".to_string()));
    }

    #[test]
    fn test_is_tailwind_v4_css() {
        assert!(is_tailwind_v4_css("@import \"tailwindcss\";"));
        assert!(is_tailwind_v4_css("@import 'tailwindcss';"));
        assert!(is_tailwind_v4_css("@import\"tailwindcss\";"));
        assert!(!is_tailwind_v4_css("@import \"./styles.css\";"));
        assert!(!is_tailwind_v4_css("body { color: red; }"));
    }

    #[test]
    fn test_empty_theme() {
        let css = r#"
            @theme {
            }
        "#;

        let values = extract_theme_from_css(css);
        assert!(values.is_empty());
    }

    #[test]
    fn test_complex_values() {
        let css = r#"
            @theme {
                --color-gradient: linear-gradient(to right, #3b82f6, #10b981);
                --spacing-custom: calc(1rem + 2px);
            }
        "#;

        let values = extract_theme_from_css(css);
        // Complex values are extracted as-is
        assert!(values.colors.contains_key("gradient"));
        assert!(values.spacing.contains_key("calc(1rem + 2px)"));
    }

    #[test]
    fn test_extract_animations() {
        let css = r#"
            @theme {
                --animate-spin: spin 1s linear infinite;
                --animate-bounce: bounce 1s ease-in-out infinite;
            }
        "#;

        let values = extract_theme_from_css(css);
        // Values may have extra whitespace from CSS parsing - just check key exists
        assert!(values.animations.contains_key("spin"));
        assert!(values.animations.contains_key("bounce"));
        // Verify the extracted values contain the expected parts
        let spin_val = &values.animations["spin"];
        assert!(
            spin_val.contains("spin") && spin_val.contains("1s") && spin_val.contains("linear")
        );
    }

    #[test]
    fn test_extract_font_family() {
        let css = r#"
            @theme {
                --font-sans: ui-sans-serif, system-ui, sans-serif;
                --font-serif: ui-serif, Georgia, serif;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert!(values.font_family.contains_key("sans"));
        assert!(values.font_family.contains_key("serif"));
    }

    #[test]
    fn test_extract_box_shadow() {
        let css = r#"
            @theme {
                --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
                --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
            }
        "#;

        let values = extract_theme_from_css(css);
        assert!(values.box_shadow.contains_key("sm"));
        assert!(values.box_shadow.contains_key("lg"));
    }

    #[test]
    fn test_extract_breakpoints() {
        let css = r#"
            @theme {
                --breakpoint-sm: 640px;
                --breakpoint-md: 768px;
                --breakpoint-lg: 1024px;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.breakpoints.get("sm"), Some(&"640px".to_string()));
        assert_eq!(values.breakpoints.get("md"), Some(&"768px".to_string()));
        assert_eq!(values.breakpoints.get("lg"), Some(&"1024px".to_string()));
    }

    #[test]
    fn test_extract_letter_spacing() {
        let css = r#"
            @theme {
                --tracking-tight: -0.025em;
                --tracking-wide: 0.025em;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(
            values.letter_spacing.get("tight"),
            Some(&"-0.025em".to_string())
        );
        assert_eq!(
            values.letter_spacing.get("wide"),
            Some(&"0.025em".to_string())
        );
    }

    #[test]
    fn test_extract_line_height() {
        let css = r#"
            @theme {
                --leading-tight: 1.25;
                --leading-relaxed: 1.625;
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(values.line_height.get("tight"), Some(&"1.25".to_string()));
        assert_eq!(
            values.line_height.get("relaxed"),
            Some(&"1.625".to_string())
        );
    }

    #[test]
    fn test_extract_transitions_and_easing() {
        let css = r#"
            @theme {
                --transition-fast: 150ms;
                --transition-slow: 500ms;
                --ease-in: cubic-bezier(0.4, 0, 1, 1);
                --ease-out: cubic-bezier(0, 0, 0.2, 1);
            }
        "#;

        let values = extract_theme_from_css(css);
        assert_eq!(
            values.transition_duration.get("fast"),
            Some(&"150ms".to_string())
        );
        assert_eq!(
            values.transition_duration.get("slow"),
            Some(&"500ms".to_string())
        );
        assert!(values.easing.contains_key("in"));
        assert!(values.easing.contains_key("out"));
    }

    #[test]
    fn test_font_size_vs_font_family() {
        // Ensure --font-size-* and --font-* are correctly categorized
        let css = r#"
            @theme {
                --font-size-lg: 1.125rem;
                --font-mono: ui-monospace, monospace;
            }
        "#;

        let values = extract_theme_from_css(css);
        // --font-size-lg should go to font_size (value -> suffix mapping)
        // Key is "1.125rem", value is "lg"
        assert!(values.font_size.contains_key("1.125rem"));
        assert_eq!(values.font_size.get("1.125rem"), Some(&"lg".to_string()));
        assert!(!values.font_family.contains_key("lg"));
        // --font-mono should go to font_family (suffix -> value mapping)
        // Key is "mono", value is the font stack
        assert!(values.font_family.contains_key("mono"));
        // font_size should NOT contain the font-mono value as a key
        assert!(!values.font_size.values().any(|v| v == "mono"));
    }

    #[test]
    fn test_extract_import_paths() {
        let css = r#"
            @import "tailwindcss";
            @import "./custom-theme.css";
            @import "../shared/base.css";
            @import "some-package";

            @theme {
                --color-primary: #3b82f6;
            }
        "#;

        let values = extract_theme_from_css(css);
        // Only relative paths are extracted
        assert_eq!(values.import_paths.len(), 2);
        assert!(values.import_paths.contains(&"./custom-theme.css".to_string()));
        assert!(values.import_paths.contains(&"../shared/base.css".to_string()));
        // Non-relative paths are not included
        assert!(!values.import_paths.iter().any(|p| p == "tailwindcss"));
        assert!(!values.import_paths.iter().any(|p| p == "some-package"));
    }

    #[test]
    fn test_merge_theme_values() {
        let mut base = TailwindThemeValues::default();
        base.colors.insert("primary".to_string(), "#3b82f6".to_string());
        base.spacing.insert("4.5rem".to_string(), "18".to_string());

        let mut other = TailwindThemeValues::default();
        other.colors.insert("primary".to_string(), "#ff0000".to_string()); // Should not override
        other.colors.insert("secondary".to_string(), "#10b981".to_string()); // Should be added
        other.spacing.insert("3.25rem".to_string(), "13".to_string()); // Should be added

        base.merge(other);

        // Original value preserved
        assert_eq!(base.colors.get("primary"), Some(&"#3b82f6".to_string()));
        // New value added
        assert_eq!(base.colors.get("secondary"), Some(&"#10b981".to_string()));
        assert_eq!(base.spacing.get("3.25rem"), Some(&"13".to_string()));
    }
}
