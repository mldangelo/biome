use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_rowan::AstNode;
use biome_rule_options::no_unregistered_tailwind_classes::NoUnregisteredTailwindClassesOptions;
use rustc_hash::FxHashSet;
use std::sync::LazyLock;

declare_lint_rule! {
    /// Detects potentially unregistered or unknown Tailwind CSS classes.
    ///
    /// This rule identifies class names that don't match known Tailwind CSS patterns,
    /// which may indicate typos or the use of undefined custom classes.
    ///
    /// The rule validates against common Tailwind utility patterns and allows:
    /// - All standard Tailwind utilities (flex, grid, p-*, m-*, text-*, etc.)
    /// - Arbitrary values with bracket syntax (`[value]`)
    /// - Variants (hover:, dark:, sm:, etc.)
    /// - Negative values (-m-4, -translate-x-1, etc.)
    /// - Important modifier (!)
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="hello-world" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="text-prrimary" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="flex items-center p-4" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="text-[#ff0000] bg-[var(--color)]" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="hover:bg-blue-500 dark:text-white" />;
    /// ```
    ///
    pub NoUnregisteredTailwindClasses {
        version: "next",
        name: "noUnregisteredTailwindClasses",
        language: "jsx",
        recommended: false,
    }
}

/// Known Tailwind utility prefixes
/// This is not exhaustive but covers the most common utilities
static KNOWN_UTILITY_PREFIXES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    FxHashSet::from_iter([
        // Layout
        "container",
        "columns",
        "break-after",
        "break-before",
        "break-inside",
        "box-decoration",
        "box",
        "float",
        "clear",
        "isolation",
        "object",
        "overflow",
        "overscroll",
        "position",
        "static",
        "fixed",
        "absolute",
        "relative",
        "sticky",
        "inset",
        "top",
        "right",
        "bottom",
        "left",
        "start",
        "end",
        "visible",
        "invisible",
        "collapse",
        "z",
        // Flexbox & Grid
        "basis",
        "flex",
        "shrink",
        "grow",
        "order",
        "grid",
        "col",
        "row",
        "auto-cols",
        "auto-rows",
        "gap",
        "justify",
        "content",
        "items",
        "self",
        "place",
        // Spacing
        "p",
        "px",
        "py",
        "ps",
        "pe",
        "pt",
        "pr",
        "pb",
        "pl",
        "m",
        "mx",
        "my",
        "ms",
        "me",
        "mt",
        "mr",
        "mb",
        "ml",
        "space",
        // Sizing
        "w",
        "min-w",
        "max-w",
        "h",
        "min-h",
        "max-h",
        "size",
        // Typography
        "font",
        "text",
        "antialiased",
        "subpixel-antialiased",
        "italic",
        "not-italic",
        "normal-nums",
        "ordinal",
        "slashed-zero",
        "lining-nums",
        "oldstyle-nums",
        "proportional-nums",
        "tabular-nums",
        "diagonal-fractions",
        "stacked-fractions",
        "tracking",
        "leading",
        "list",
        "decoration",
        "underline",
        "overline",
        "line-through",
        "no-underline",
        "uppercase",
        "lowercase",
        "capitalize",
        "normal-case",
        "truncate",
        "indent",
        "align",
        "whitespace",
        "break",
        "hyphens",
        "content",
        // Backgrounds
        "bg",
        "from",
        "via",
        "to",
        "gradient",
        // Borders
        "rounded",
        "border",
        "divide",
        "outline",
        "ring",
        // Effects
        "shadow",
        "opacity",
        "mix-blend",
        "bg-blend",
        // Filters
        "blur",
        "brightness",
        "contrast",
        "drop-shadow",
        "grayscale",
        "hue-rotate",
        "invert",
        "saturate",
        "sepia",
        "backdrop",
        // Tables
        "table",
        "caption",
        // Transitions & Animation
        "transition",
        "duration",
        "ease",
        "delay",
        "animate",
        // Transforms
        "scale",
        "rotate",
        "translate",
        "skew",
        "origin",
        // Interactivity
        "accent",
        "appearance",
        "cursor",
        "caret",
        "pointer-events",
        "resize",
        "scroll",
        "snap",
        "touch",
        "select",
        "will-change",
        // SVG
        "fill",
        "stroke",
        // Accessibility
        "sr-only",
        "not-sr-only",
        // Display
        "block",
        "inline-block",
        "inline",
        "hidden",
        "aspect",
        // Additional common utilities
        "prose",
        "line-clamp",
        "forced-color-adjust",
    ])
});

/// Known Tailwind variants
static KNOWN_VARIANTS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    FxHashSet::from_iter([
        // Pseudo-class variants
        "hover",
        "focus",
        "focus-within",
        "focus-visible",
        "active",
        "visited",
        "target",
        "first",
        "last",
        "only",
        "odd",
        "even",
        "first-of-type",
        "last-of-type",
        "only-of-type",
        "empty",
        "disabled",
        "enabled",
        "checked",
        "indeterminate",
        "default",
        "required",
        "valid",
        "invalid",
        "in-range",
        "out-of-range",
        "placeholder-shown",
        "autofill",
        "read-only",
        // Pseudo-element variants
        "before",
        "after",
        "first-letter",
        "first-line",
        "marker",
        "selection",
        "file",
        "backdrop",
        "placeholder",
        // Media query variants
        "sm",
        "md",
        "lg",
        "xl",
        "2xl",
        "min",
        "max",
        // Preference variants
        "dark",
        "light",
        "motion-safe",
        "motion-reduce",
        "contrast-more",
        "contrast-less",
        "forced-colors",
        "print",
        "portrait",
        "landscape",
        // State variants
        "open",
        "closed",
        // RTL/LTR
        "rtl",
        "ltr",
        // Groups/peers
        "group",
        "group-hover",
        "group-focus",
        "group-active",
        "group-first",
        "group-last",
        "group-odd",
        "group-even",
        "peer",
        "peer-hover",
        "peer-focus",
        "peer-active",
        "peer-checked",
        "peer-disabled",
        "peer-invalid",
        // Has variants
        "has",
        // Aria variants
        "aria-checked",
        "aria-disabled",
        "aria-expanded",
        "aria-hidden",
        "aria-pressed",
        "aria-readonly",
        "aria-required",
        "aria-selected",
        // Data variants
        "data",
        // Supports variants
        "supports",
    ])
});

/// State containing the list of unregistered classes
pub struct UnregisteredClassesState {
    pub unregistered: Vec<String>,
}

impl Rule for NoUnregisteredTailwindClasses {
    type Query = Ast<AnyClassStringLike>;
    type State = UnregisteredClassesState;
    type Signals = Option<Self::State>;
    type Options = NoUnregisteredTailwindClassesOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        if !node.should_visit(options)? {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();

        let mut unregistered = Vec::new();

        for class in value_str.split_whitespace() {
            // Skip if whitelisted
            if options.is_whitelisted(class) {
                continue;
            }
            if !is_valid_tailwind_class(class) {
                unregistered.push(class.to_string());
            }
        }

        if unregistered.is_empty() {
            return None;
        }

        Some(UnregisteredClassesState { unregistered })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let classes_str = state
            .unregistered
            .iter()
            .map(|c| format!("`{}`", c))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Unknown Tailwind CSS class(es) detected."
                },
            )
            .note(markup! {
                "Unrecognized: "{classes_str}
            })
            .note(markup! {
                "These class names don't match known Tailwind patterns. Check for typos or ensure custom classes are properly configured."
            }),
        )
    }
}

/// Check if a class is a valid Tailwind class
fn is_valid_tailwind_class(class: &str) -> bool {
    // Handle important modifier
    let class = class.strip_prefix('!').unwrap_or(class);

    // Handle negative modifier
    let class = class.strip_prefix('-').unwrap_or(class);

    // Pure arbitrary property (e.g., [mask-type:alpha]) - check before splitting by :
    if class.starts_with('[') && class.ends_with(']') {
        return true;
    }

    // Split variants from utility
    let parts: Vec<&str> = class.split(':').collect();

    // Validate variants (all but the last part)
    for variant in &parts[..parts.len().saturating_sub(1)] {
        if !is_valid_variant(variant) {
            return false;
        }
    }

    // Validate the utility (the last part)
    let utility = parts.last().unwrap_or(&"");
    is_valid_utility(utility)
}

/// Check if a variant is valid
fn is_valid_variant(variant: &str) -> bool {
    // Handle arbitrary variants like [&:hover]
    if variant.starts_with('[') && variant.ends_with(']') {
        return true;
    }

    // Handle group-* and peer-* variants
    if variant.starts_with("group-") || variant.starts_with("peer-") {
        return true;
    }

    // Handle aria-* variants
    if variant.starts_with("aria-") {
        return true;
    }

    // Handle data-* variants
    if variant.starts_with("data-") {
        return true;
    }

    // Handle max-* and min-* variants
    if variant.starts_with("max-") || variant.starts_with("min-") {
        return true;
    }

    // Handle supports-* variants
    if variant.starts_with("supports-") {
        return true;
    }

    // Handle has-* variants
    if variant.starts_with("has-") {
        return true;
    }

    KNOWN_VARIANTS.contains(variant)
}

/// Check if a utility is valid
fn is_valid_utility(utility: &str) -> bool {
    // Empty utility is invalid
    if utility.is_empty() {
        return false;
    }

    // Pure arbitrary property (e.g., [mask-type:alpha])
    if utility.starts_with('[') && utility.ends_with(']') {
        return true;
    }

    // Arbitrary values are always valid
    if utility.contains('[') && utility.contains(']') {
        return true;
    }

    // Handle negative utility
    let utility = utility.strip_prefix('-').unwrap_or(utility);

    // Check if the utility matches a known prefix
    // First, try exact match for single-word utilities
    if KNOWN_UTILITY_PREFIXES.contains(utility) {
        return true;
    }

    // Try matching prefix with various separators
    for sep in ['-', '/'] {
        if let Some((prefix, _)) = utility.split_once(sep)
            && KNOWN_UTILITY_PREFIXES.contains(prefix)
        {
            return true;
        }
    }

    // Check for compound prefixes (like min-w, max-h, etc.)
    for compound in ["min-w", "max-w", "min-h", "max-h", "auto-cols", "auto-rows"] {
        if utility.starts_with(compound) {
            return true;
        }
    }

    // Check for special single-word utilities
    let single_word_utilities: &[&str] = &[
        "container",
        "prose",
        "static",
        "fixed",
        "absolute",
        "relative",
        "sticky",
        "visible",
        "invisible",
        "collapse",
        "block",
        "inline-block",
        "inline",
        "hidden",
        "flex",
        "inline-flex",
        "grid",
        "inline-grid",
        "table",
        "inline-table",
        "table-caption",
        "table-cell",
        "table-column",
        "table-column-group",
        "table-footer-group",
        "table-header-group",
        "table-row-group",
        "table-row",
        "flow-root",
        "contents",
        "list-item",
        "antialiased",
        "subpixel-antialiased",
        "italic",
        "not-italic",
        "underline",
        "overline",
        "line-through",
        "no-underline",
        "uppercase",
        "lowercase",
        "capitalize",
        "normal-case",
        "truncate",
        "sr-only",
        "not-sr-only",
        "pointer-events-none",
        "pointer-events-auto",
        "resize-none",
        "resize-y",
        "resize-x",
        "resize",
        "select-none",
        "select-text",
        "select-all",
        "select-auto",
        "appearance-none",
        "appearance-auto",
    ];

    single_word_utilities.contains(&utility)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_utilities() {
        assert!(is_valid_tailwind_class("flex"));
        assert!(is_valid_tailwind_class("p-4"));
        assert!(is_valid_tailwind_class("mx-auto"));
        assert!(is_valid_tailwind_class("text-red-500"));
        assert!(is_valid_tailwind_class("bg-blue-500/50"));
        assert!(is_valid_tailwind_class("-translate-x-1"));
        assert!(is_valid_tailwind_class("!p-4"));
    }

    #[test]
    fn test_valid_with_variants() {
        assert!(is_valid_tailwind_class("hover:bg-blue-500"));
        assert!(is_valid_tailwind_class("dark:text-white"));
        assert!(is_valid_tailwind_class("sm:md:lg:flex"));
        assert!(is_valid_tailwind_class("group-hover:opacity-100"));
    }

    #[test]
    fn test_arbitrary_values() {
        assert!(is_valid_tailwind_class("w-[100px]"));
        assert!(is_valid_tailwind_class("bg-[#ff0000]"));
        assert!(is_valid_tailwind_class("text-[var(--color)]"));
        assert!(is_valid_tailwind_class("[mask-type:alpha]"));
    }

    #[test]
    fn test_invalid_classes() {
        assert!(!is_valid_tailwind_class("hello-world"));
        assert!(!is_valid_tailwind_class("unknownprefix-value"));
        assert!(!is_valid_tailwind_class("myclass"));
    }
}
