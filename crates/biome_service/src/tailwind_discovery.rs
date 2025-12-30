//! Tailwind CSS v4 auto-discovery module.
//!
//! This module handles automatic detection of Tailwind v4 CSS files in a project.
//! It searches for CSS files containing `@import "tailwindcss"` which indicates
//! a Tailwind v4 project.

use biome_configuration::TailwindConfiguration;
use biome_css_analyze::tailwind_theme_extractor::{TailwindThemeValues, extract_theme_from_css};
use biome_fs::FileSystem;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeMap;
use tracing::debug;

/// Common paths to check for Tailwind CSS files.
/// These are ordered by likelihood based on common project structures.
const COMMON_CSS_PATHS: &[&str] = &[
    // Next.js App Router
    "app/globals.css",
    "app/global.css",
    "src/app/globals.css",
    "src/app/global.css",
    // Common React/Vue/Svelte patterns
    "src/index.css",
    "src/app.css",
    "src/global.css",
    "src/globals.css",
    "src/styles.css",
    "src/main.css",
    // Styles directory patterns
    "styles/globals.css",
    "styles/global.css",
    "styles/index.css",
    "styles/main.css",
    "src/styles/globals.css",
    "src/styles/global.css",
    "src/styles/index.css",
    // Assets patterns
    "assets/css/main.css",
    "src/assets/css/main.css",
    // Root level
    "index.css",
    "global.css",
    "globals.css",
    "main.css",
    "styles.css",
];

/// Result of Tailwind CSS discovery.
#[derive(Debug, Clone)]
pub struct TailwindDiscoveryResult {
    /// Path to the discovered CSS file (relative to working directory).
    pub css_path: Utf8PathBuf,
    /// The CSS content of the file.
    pub content: String,
}

/// Discover Tailwind v4 CSS file in the project.
///
/// This function checks common paths where Tailwind CSS files are typically located.
///
/// # Arguments
/// * `fs` - The filesystem to use for reading files
/// * `working_directory` - The project root directory
///
/// # Returns
/// * `Some(TailwindDiscoveryResult)` if a Tailwind v4 CSS file is found
/// * `None` if no Tailwind v4 CSS file is found
pub fn discover_tailwind_css(
    fs: &dyn FileSystem,
    working_directory: &Utf8Path,
) -> Option<TailwindDiscoveryResult> {
    // Check common paths
    for path in COMMON_CSS_PATHS {
        let full_path = working_directory.join(path);
        if let Some(result) = check_css_file(fs, &full_path, path) {
            return Some(result);
        }
    }

    None
}

/// Check if a CSS file contains Tailwind v4 imports.
fn check_css_file(
    fs: &dyn FileSystem,
    full_path: &Utf8Path,
    relative_path: &str,
) -> Option<TailwindDiscoveryResult> {
    let content = fs.read_file_from_path(full_path).ok()?;

    if is_tailwind_v4_css(&content) {
        Some(TailwindDiscoveryResult {
            css_path: Utf8PathBuf::from(relative_path),
            content,
        })
    } else {
        None
    }
}

/// Check if CSS content contains Tailwind v4 imports.
pub fn is_tailwind_v4_css(content: &str) -> bool {
    content.contains("@import \"tailwindcss\"")
        || content.contains("@import 'tailwindcss'")
        || content.contains("@import\"tailwindcss\"")
        || content.contains("@import'tailwindcss'")
}

/// Load Tailwind v4 CSS configuration and merge it into the existing configuration.
///
/// This function:
/// 1. Checks for explicit `css_path` in the config or auto-discovers a CSS file
/// 2. Parses the CSS and extracts `@theme` values
/// 3. Merges extracted values into the TailwindConfiguration (biome.json values take precedence)
///
/// # Arguments
/// * `fs` - The filesystem to use for reading files
/// * `working_directory` - The project root directory
/// * `config` - The existing TailwindConfiguration (from biome.json)
///
/// # Returns
/// The modified TailwindConfiguration with CSS theme values merged in
pub fn load_tailwind_v4_config(
    fs: &dyn FileSystem,
    working_directory: &Utf8Path,
    config: &mut TailwindConfiguration,
) {
    // Try to get CSS content - either from explicit path or auto-discovery
    let css_content = if let Some(css_path) = config.css_path() {
        // Explicit css_path configured - load that file
        let full_path = working_directory.join(css_path);
        match fs.read_file_from_path(&full_path) {
            Ok(content) => {
                debug!("Loaded Tailwind CSS from explicit path: {}", full_path);
                Some(content)
            }
            Err(e) => {
                debug!("Failed to load Tailwind CSS from {}: {}", full_path, e);
                None
            }
        }
    } else {
        // Try auto-discovery
        discover_tailwind_css(fs, working_directory).map(|result| {
            debug!("Auto-discovered Tailwind CSS at: {}", result.css_path);
            result.content
        })
    };

    // If we found CSS content, extract and merge theme values
    if let Some(content) = css_content {
        let theme_values = extract_theme_from_css(&content);

        if !theme_values.is_empty() {
            debug!(
                "Extracted {} theme categories from CSS",
                count_theme_categories(&theme_values)
            );
            merge_theme_into_config(config, theme_values);
        }
    }
}

/// Count how many theme categories have values.
fn count_theme_categories(values: &TailwindThemeValues) -> usize {
    let mut count = 0;
    if !values.colors.is_empty() {
        count += 1;
    }
    if !values.spacing.is_empty() {
        count += 1;
    }
    if !values.font_size.is_empty() {
        count += 1;
    }
    if !values.border_radius.is_empty() {
        count += 1;
    }
    if !values.z_index.is_empty() {
        count += 1;
    }
    if !values.opacity.is_empty() {
        count += 1;
    }
    count
}

/// Merge extracted theme values into TailwindConfiguration.
///
/// Values from biome.json take precedence - CSS values only fill in gaps.
fn merge_theme_into_config(config: &mut TailwindConfiguration, theme: TailwindThemeValues) {
    // Merge spacing (CSS values merged, biome.json values take precedence)
    if !theme.spacing.is_empty() {
        let existing = config.spacing.take().unwrap_or_default();
        config.spacing = Some(merge_btree_maps(theme.spacing, existing));
    }

    // Merge opacity
    if !theme.opacity.is_empty() {
        let existing = config.opacity.take().unwrap_or_default();
        config.opacity = Some(merge_btree_maps(theme.opacity, existing));
    }

    // Merge z_index
    if !theme.z_index.is_empty() {
        let existing = config.z_index.take().unwrap_or_default();
        config.z_index = Some(merge_btree_maps(theme.z_index, existing));
    }

    // Merge font_size
    if !theme.font_size.is_empty() {
        let existing = config.font_size.take().unwrap_or_default();
        config.font_size = Some(merge_btree_maps(theme.font_size, existing));
    }

    // Merge border_radius
    if !theme.border_radius.is_empty() {
        let existing = config.border_radius.take().unwrap_or_default();
        config.border_radius = Some(merge_btree_maps(theme.border_radius, existing));
    }

    // Note: Colors are stored as name -> value in theme_values (e.g., "primary" -> "#3b82f6")
    // but TailwindConfiguration doesn't have a colors field - colors are used differently.
    // We skip colors for now as they're not used in the same way by tailwind_utils.
}

/// Merge two BTreeMaps, with `override_values` taking precedence.
fn merge_btree_maps(
    base: BTreeMap<String, String>,
    override_values: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut result = base;
    // Override values take precedence (these are from biome.json)
    result.extend(override_values);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tailwind_v4_css() {
        assert!(is_tailwind_v4_css("@import \"tailwindcss\";"));
        assert!(is_tailwind_v4_css("@import 'tailwindcss';"));
        assert!(is_tailwind_v4_css("@import\"tailwindcss\";"));
        assert!(is_tailwind_v4_css("@import'tailwindcss';"));
        // With other imports
        assert!(is_tailwind_v4_css(
            "@import \"./custom.css\";\n@import \"tailwindcss\";"
        ));
        // Not Tailwind v4
        assert!(!is_tailwind_v4_css("@import \"./styles.css\";"));
        assert!(!is_tailwind_v4_css("body { color: red; }"));
        assert!(!is_tailwind_v4_css("@tailwind base;"));
    }

    #[test]
    fn test_merge_btree_maps_override_values_take_precedence() {
        let mut base = BTreeMap::new();
        base.insert("4.5rem".to_string(), "18".to_string());
        base.insert("3.25rem".to_string(), "13".to_string());

        let mut override_values = BTreeMap::new();
        override_values.insert("4.5rem".to_string(), "custom".to_string()); // Override base
        override_values.insert("1rem".to_string(), "4".to_string()); // New value

        let result = merge_btree_maps(base, override_values);

        // Override value takes precedence
        assert_eq!(result.get("4.5rem"), Some(&"custom".to_string()));
        // Base value preserved
        assert_eq!(result.get("3.25rem"), Some(&"13".to_string()));
        // New value added
        assert_eq!(result.get("1rem"), Some(&"4".to_string()));
    }

    #[test]
    fn test_merge_theme_into_config_preserves_biome_json_values() {
        let mut config = TailwindConfiguration::default();
        // Simulate biome.json values
        let mut biome_spacing = BTreeMap::new();
        biome_spacing.insert("3.25rem".to_string(), "13".to_string());
        config.spacing = Some(biome_spacing);

        // Simulate CSS @theme values
        let mut theme = TailwindThemeValues::default();
        theme.spacing.insert("4.5rem".to_string(), "18".to_string());
        theme
            .spacing
            .insert("3.25rem".to_string(), "override-attempt".to_string());

        merge_theme_into_config(&mut config, theme);

        let spacing = config.spacing.unwrap();
        // biome.json value preserved (takes precedence)
        assert_eq!(spacing.get("3.25rem"), Some(&"13".to_string()));
        // CSS value added
        assert_eq!(spacing.get("4.5rem"), Some(&"18".to_string()));
    }

    #[test]
    fn test_merge_theme_into_config_all_categories() {
        let mut config = TailwindConfiguration::default();

        let mut theme = TailwindThemeValues::default();
        theme.spacing.insert("4.5rem".to_string(), "18".to_string());
        theme.opacity.insert("0.5".to_string(), "muted".to_string());
        theme.z_index.insert("100".to_string(), "modal".to_string());
        theme
            .font_size
            .insert("0.625rem".to_string(), "2xs".to_string());
        theme
            .border_radius
            .insert("1rem".to_string(), "xl".to_string());

        merge_theme_into_config(&mut config, theme);

        assert!(config.spacing.is_some());
        assert!(config.opacity.is_some());
        assert!(config.z_index.is_some());
        assert!(config.font_size.is_some());
        assert!(config.border_radius.is_some());

        assert_eq!(
            config.spacing.unwrap().get("4.5rem"),
            Some(&"18".to_string())
        );
        assert_eq!(
            config.opacity.unwrap().get("0.5"),
            Some(&"muted".to_string())
        );
        assert_eq!(
            config.z_index.unwrap().get("100"),
            Some(&"modal".to_string())
        );
        assert_eq!(
            config.font_size.unwrap().get("0.625rem"),
            Some(&"2xs".to_string())
        );
        assert_eq!(
            config.border_radius.unwrap().get("1rem"),
            Some(&"xl".to_string())
        );
    }

    #[test]
    fn test_count_theme_categories() {
        let mut theme = TailwindThemeValues::default();
        assert_eq!(count_theme_categories(&theme), 0);

        theme
            .colors
            .insert("primary".to_string(), "#3b82f6".to_string());
        assert_eq!(count_theme_categories(&theme), 1);

        theme.spacing.insert("4.5rem".to_string(), "18".to_string());
        assert_eq!(count_theme_categories(&theme), 2);

        theme.opacity.insert("0.5".to_string(), "muted".to_string());
        theme.z_index.insert("100".to_string(), "modal".to_string());
        theme
            .font_size
            .insert("0.625rem".to_string(), "2xs".to_string());
        theme
            .border_radius
            .insert("1rem".to_string(), "xl".to_string());
        assert_eq!(count_theme_categories(&theme), 6);
    }
}
