//! Shared utilities for Tailwind CSS lint rules.
//!
//! This module provides default scales and utilities that are used by multiple
//! Tailwind CSS lint rules. It also supports merging custom configuration
//! with these defaults.

use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Default Tailwind CSS spacing scale.
/// Maps CSS values (rem/px) to Tailwind utility suffixes.
pub static DEFAULT_SPACING_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            // rem values
            ("0rem", "0"),
            ("0.125rem", "0.5"),
            ("0.25rem", "1"),
            ("0.375rem", "1.5"),
            ("0.5rem", "2"),
            ("0.625rem", "2.5"),
            ("0.75rem", "3"),
            ("0.875rem", "3.5"),
            ("1rem", "4"),
            ("1.25rem", "5"),
            ("1.5rem", "6"),
            ("1.75rem", "7"),
            ("2rem", "8"),
            ("2.25rem", "9"),
            ("2.5rem", "10"),
            ("2.75rem", "11"),
            ("3rem", "12"),
            ("3.5rem", "14"),
            ("4rem", "16"),
            ("5rem", "20"),
            ("6rem", "24"),
            ("7rem", "28"),
            ("8rem", "32"),
            ("9rem", "36"),
            ("10rem", "40"),
            ("11rem", "44"),
            ("12rem", "48"),
            ("13rem", "52"),
            ("14rem", "56"),
            ("15rem", "60"),
            ("16rem", "64"),
            ("18rem", "72"),
            ("20rem", "80"),
            ("24rem", "96"),
            // px values (1rem = 16px)
            ("0px", "0"),
            ("1px", "px"),
            ("2px", "0.5"),
            ("4px", "1"),
            ("6px", "1.5"),
            ("8px", "2"),
            ("10px", "2.5"),
            ("12px", "3"),
            ("14px", "3.5"),
            ("16px", "4"),
            ("20px", "5"),
            ("24px", "6"),
            ("28px", "7"),
            ("32px", "8"),
            ("36px", "9"),
            ("40px", "10"),
            ("44px", "11"),
            ("48px", "12"),
            ("56px", "14"),
            ("64px", "16"),
            ("80px", "20"),
            ("96px", "24"),
            ("112px", "28"),
            ("128px", "32"),
            ("144px", "36"),
            ("160px", "40"),
            ("176px", "44"),
            ("192px", "48"),
            ("208px", "52"),
            ("224px", "56"),
            ("240px", "60"),
            ("256px", "64"),
            ("288px", "72"),
            ("320px", "80"),
            ("384px", "96"),
        ])
    });

/// Default Tailwind CSS opacity scale.
pub static DEFAULT_OPACITY_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            ("0", "0"),
            ("0.05", "5"),
            ("0.1", "10"),
            ("0.15", "15"),
            ("0.2", "20"),
            ("0.25", "25"),
            ("0.3", "30"),
            ("0.35", "35"),
            ("0.4", "40"),
            ("0.45", "45"),
            ("0.5", "50"),
            ("0.55", "55"),
            ("0.6", "60"),
            ("0.65", "65"),
            ("0.7", "70"),
            ("0.75", "75"),
            ("0.8", "80"),
            ("0.85", "85"),
            ("0.9", "90"),
            ("0.95", "95"),
            ("1", "100"),
        ])
    });

/// Default Tailwind CSS z-index scale.
pub static DEFAULT_Z_INDEX_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            ("0", "0"),
            ("10", "10"),
            ("20", "20"),
            ("30", "30"),
            ("40", "40"),
            ("50", "50"),
            ("auto", "auto"),
        ])
    });

/// Default Tailwind CSS font-size scale.
pub static DEFAULT_FONT_SIZE_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            ("0.75rem", "xs"),
            ("0.875rem", "sm"),
            ("1rem", "base"),
            ("1.125rem", "lg"),
            ("1.25rem", "xl"),
            ("1.5rem", "2xl"),
            ("1.875rem", "3xl"),
            ("2.25rem", "4xl"),
            ("3rem", "5xl"),
            ("3.75rem", "6xl"),
            ("4.5rem", "7xl"),
            ("6rem", "8xl"),
            ("8rem", "9xl"),
            // px equivalents
            ("12px", "xs"),
            ("14px", "sm"),
            ("16px", "base"),
            ("18px", "lg"),
            ("20px", "xl"),
            ("24px", "2xl"),
            ("30px", "3xl"),
            ("36px", "4xl"),
            ("48px", "5xl"),
            ("60px", "6xl"),
            ("72px", "7xl"),
            ("96px", "8xl"),
            ("128px", "9xl"),
        ])
    });

/// Default Tailwind CSS border-radius scale.
pub static DEFAULT_BORDER_RADIUS_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            ("0px", "none"),
            ("0.125rem", "sm"),
            ("0.25rem", "DEFAULT"),
            ("0.375rem", "md"),
            ("0.5rem", "lg"),
            ("0.75rem", "xl"),
            ("1rem", "2xl"),
            ("1.5rem", "3xl"),
            ("9999px", "full"),
            // px equivalents
            ("2px", "sm"),
            ("4px", "DEFAULT"),
            ("6px", "md"),
            ("8px", "lg"),
            ("12px", "xl"),
            ("16px", "2xl"),
            ("24px", "3xl"),
        ])
    });

/// Default Tailwind CSS width/height percentage scale.
pub static DEFAULT_PERCENTAGE_SCALE: LazyLock<FxHashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            ("100%", "full"),
            ("50%", "1/2"),
            ("33.333333%", "1/3"),
            ("66.666667%", "2/3"),
            ("25%", "1/4"),
            ("75%", "3/4"),
            ("20%", "1/5"),
            ("40%", "2/5"),
            ("60%", "3/5"),
            ("80%", "4/5"),
            ("16.666667%", "1/6"),
            ("83.333333%", "5/6"),
            ("100vh", "screen"),
            ("100vw", "screen"),
            ("100dvh", "dvh"),
            ("100dvw", "dvw"),
            ("100svh", "svh"),
            ("100lvh", "lvh"),
        ])
    });

/// Default utilities that support negative values.
pub static DEFAULT_NEGATABLE_UTILITIES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "m",
        "mx",
        "my",
        "mt",
        "mr",
        "mb",
        "ml",
        "ms",
        "me",
        "p",
        "px",
        "py",
        "pt",
        "pr",
        "pb",
        "pl",
        "ps",
        "pe",
        "top",
        "right",
        "bottom",
        "left",
        "inset",
        "inset-x",
        "inset-y",
        "start",
        "end",
        "translate-x",
        "translate-y",
        "rotate",
        "skew-x",
        "skew-y",
        "scale",
        "scale-x",
        "scale-y",
        "scroll-m",
        "scroll-mx",
        "scroll-my",
        "scroll-mt",
        "scroll-mr",
        "scroll-mb",
        "scroll-ml",
        "scroll-ms",
        "scroll-me",
        "scroll-p",
        "scroll-px",
        "scroll-py",
        "scroll-pt",
        "scroll-pr",
        "scroll-pb",
        "scroll-pl",
        "scroll-ps",
        "scroll-pe",
        "indent",
        "z",
        "order",
        "space-x",
        "space-y",
        "tracking",
        "hue-rotate",
        "backdrop-hue-rotate",
    ]
});

/// Merged spacing scale that combines defaults with custom values.
pub fn get_spacing_scale(custom: Option<&BTreeMap<String, String>>) -> FxHashMap<String, String> {
    let mut result: FxHashMap<String, String> = DEFAULT_SPACING_SCALE
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();

    if let Some(custom) = custom {
        for (k, v) in custom {
            result.insert(k.clone(), v.clone());
        }
    }

    result
}

/// Merged opacity scale that combines defaults with custom values.
pub fn get_opacity_scale(custom: Option<&BTreeMap<String, String>>) -> FxHashMap<String, String> {
    let mut result: FxHashMap<String, String> = DEFAULT_OPACITY_SCALE
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();

    if let Some(custom) = custom {
        for (k, v) in custom {
            result.insert(k.clone(), v.clone());
        }
    }

    result
}

/// Check if a utility supports negative values.
pub fn supports_negative(utility: &str, custom_negatable: Option<&[String]>) -> bool {
    if DEFAULT_NEGATABLE_UTILITIES.contains(&utility) {
        return true;
    }

    if let Some(custom) = custom_negatable {
        return custom.iter().any(|s| s == utility);
    }

    false
}

/// Look up a CSS value in the spacing scale and return the Tailwind utility suffix.
pub fn lookup_spacing(value: &str, custom: Option<&BTreeMap<String, String>>) -> Option<String> {
    // Check custom first (higher priority)
    if let Some(suffix) = custom.and_then(|c| c.get(value)) {
        return Some(suffix.clone());
    }

    // Then check defaults
    DEFAULT_SPACING_SCALE.get(value).map(|s| (*s).to_string())
}

/// Look up a CSS value in the opacity scale and return the Tailwind utility suffix.
pub fn lookup_opacity(value: &str, custom: Option<&BTreeMap<String, String>>) -> Option<String> {
    if let Some(suffix) = custom.and_then(|c| c.get(value)) {
        return Some(suffix.clone());
    }

    DEFAULT_OPACITY_SCALE.get(value).map(|s| (*s).to_string())
}

/// Look up a CSS value in the z-index scale and return the Tailwind utility suffix.
pub fn lookup_z_index(value: &str, custom: Option<&BTreeMap<String, String>>) -> Option<String> {
    if let Some(suffix) = custom.and_then(|c| c.get(value)) {
        return Some(suffix.clone());
    }

    DEFAULT_Z_INDEX_SCALE.get(value).map(|s| (*s).to_string())
}

/// Look up a CSS value in the font-size scale and return the Tailwind utility suffix.
pub fn lookup_font_size(value: &str, custom: Option<&BTreeMap<String, String>>) -> Option<String> {
    if let Some(suffix) = custom.and_then(|c| c.get(value)) {
        return Some(suffix.clone());
    }

    DEFAULT_FONT_SIZE_SCALE.get(value).map(|s| (*s).to_string())
}

/// Look up a CSS value in the border-radius scale and return the Tailwind utility suffix.
pub fn lookup_border_radius(
    value: &str,
    custom: Option<&BTreeMap<String, String>>,
) -> Option<String> {
    if let Some(suffix) = custom.and_then(|c| c.get(value)) {
        return Some(suffix.clone());
    }

    DEFAULT_BORDER_RADIUS_SCALE
        .get(value)
        .map(|s| (*s).to_string())
}

/// Look up a CSS value in the percentage scale and return the Tailwind utility suffix.
pub fn lookup_percentage(value: &str) -> Option<&'static str> {
    DEFAULT_PERCENTAGE_SCALE.get(value).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_lookup() {
        assert_eq!(lookup_spacing("1rem", None), Some("4".to_string()));
        assert_eq!(lookup_spacing("16px", None), Some("4".to_string()));
        assert_eq!(lookup_spacing("unknown", None), None);
    }

    #[test]
    fn test_custom_spacing() {
        let mut custom = BTreeMap::new();
        custom.insert("3.25rem".to_string(), "13".to_string());

        assert_eq!(
            lookup_spacing("3.25rem", Some(&custom)),
            Some("13".to_string())
        );
        // Default still works
        assert_eq!(lookup_spacing("1rem", Some(&custom)), Some("4".to_string()));
    }

    #[test]
    fn test_supports_negative() {
        assert!(supports_negative("m", None));
        assert!(supports_negative("translate-x", None));
        assert!(!supports_negative("bg", None));

        let custom = vec!["custom-margin".to_string()];
        assert!(supports_negative("custom-margin", Some(&custom)));
    }
}
