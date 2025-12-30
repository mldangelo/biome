use crate::JsRuleAction;
use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use crate::lint::nursery::use_sorted_classes::tailwind_utils;
use biome_analyze::options::TailwindAnalyzerConfig;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_factory::make::{
    js_literal_member_name, js_string_literal, js_string_literal_expression,
    js_string_literal_single_quotes, js_template_chunk, js_template_chunk_element, jsx_string,
};
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::use_sorted_classes::UseSortedClassesOptions;

declare_lint_rule! {
    /// Disallow unnecessary arbitrary values that have standard Tailwind equivalents.
    ///
    /// Tailwind CSS provides a comprehensive set of utility classes with predefined values.
    /// Using arbitrary values like `m-[1rem]` when a standard utility like `m-4` exists
    /// makes the code less maintainable and inconsistent with the design system.
    ///
    /// This rule detects arbitrary values that can be replaced with their standard
    /// Tailwind equivalents and provides automatic fixes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="m-[1rem]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="p-[16px]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="w-[100%]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="opacity-[0.5]" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="m-4" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="p-4" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="w-full" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="opacity-50" />;
    /// ```
    ///
    /// ```jsx
    /// // Arbitrary values without standard equivalents are allowed
    /// <div class="w-[137px] text-[#bada55]" />;
    /// ```
    ///
    pub NoUnnecessaryArbitraryValue {
        version: "next",
        name: "noUnnecessaryArbitraryValue",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// Size keywords for width/height that aren't in the spacing scale
/// These are additional keyword values for sizing utilities
fn lookup_size_keyword(value: &str) -> Option<&'static str> {
    match value {
        "100%" => Some("full"),
        "100vw" | "100vh" => Some("screen"),
        "100svh" => Some("svh"),
        "100lvh" => Some("lvh"),
        "100dvh" => Some("dvh"),
        "100dvw" => Some("dvw"),
        "min-content" => Some("min"),
        "max-content" => Some("max"),
        "fit-content" => Some("fit"),
        "auto" => Some("auto"),
        "50%" => Some("1/2"),
        "33.333333%" => Some("1/3"),
        "66.666667%" => Some("2/3"),
        "25%" => Some("1/4"),
        "75%" => Some("3/4"),
        "20%" => Some("1/5"),
        "40%" => Some("2/5"),
        "60%" => Some("3/5"),
        "80%" => Some("4/5"),
        "16.666667%" => Some("1/6"),
        "83.333333%" => Some("5/6"),
        _ => None,
    }
}

/// State containing the fixed class string and replacements
pub struct UnnecessaryArbitraryState {
    /// The fixed class string
    pub fixed: Box<str>,
    /// List of replacements (from -> to)
    pub replacements: Vec<(String, String)>,
}

impl Rule for NoUnnecessaryArbitraryValue {
    type Query = Ast<AnyClassStringLike>;
    type State = UnnecessaryArbitraryState;
    type Signals = Option<Self::State>;
    type Options = UseSortedClassesOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();
        let tailwind_config = ctx.tailwind();

        if !node.should_visit(options)? {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();

        let mut replacements = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            if let Some((original, replacement)) = find_replacement(class, tailwind_config) {
                replacements.push((original, replacement.clone()));
                fixed_parts.push(replacement);
            } else {
                fixed_parts.push(class.to_string());
            }
        }

        if replacements.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(UnnecessaryArbitraryState {
            fixed: fixed.into(),
            replacements,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let replacements_str = state
            .replacements
            .iter()
            .map(|(from, to)| format!("`{}` -> `{}`", from, to))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Unnecessary arbitrary value(s) detected."
                },
            )
            .note(markup! {
                "Replace: "{replacements_str}
            })
            .note(markup! {
                "Use standard Tailwind utilities instead of arbitrary values for consistency."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<JsRuleAction> {
        let mut mutation = ctx.root().begin();
        let fixed = &state.fixed;

        match ctx.query() {
            AnyClassStringLike::JsStringLiteralExpression(string_literal) => {
                let is_double_quote = string_literal
                    .value_token()
                    .map(|token| token.text_trimmed().starts_with('"'))
                    .unwrap_or(ctx.preferred_quote().is_double());
                let replacement = js_string_literal_expression(if is_double_quote {
                    js_string_literal(fixed)
                } else {
                    js_string_literal_single_quotes(fixed)
                });
                mutation.replace_node(string_literal.clone(), replacement);
            }
            AnyClassStringLike::JsLiteralMemberName(string_literal) => {
                let replacement = js_literal_member_name(if ctx.preferred_quote().is_double() {
                    js_string_literal(fixed)
                } else {
                    js_string_literal_single_quotes(fixed)
                });
                mutation.replace_node(string_literal.clone(), replacement);
            }
            AnyClassStringLike::JsxString(jsx_string_node) => {
                let is_double_quote = jsx_string_node
                    .value_token()
                    .map(|token| token.text_trimmed().starts_with('"'))
                    .unwrap_or(ctx.preferred_jsx_quote().is_double());
                let replacement = jsx_string(if is_double_quote {
                    js_string_literal(fixed)
                } else {
                    js_string_literal_single_quotes(fixed)
                });
                mutation.replace_node(jsx_string_node.clone(), replacement);
            }
            AnyClassStringLike::JsTemplateChunkElement(chunk) => {
                let replacement = js_template_chunk_element(js_template_chunk(fixed));
                mutation.replace_node(chunk.clone(), replacement);
            }
        };

        Some(JsRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! {
                "Replace with standard Tailwind utility."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Check if a utility supports negative values
/// Uses the shared tailwind_utils module which can be customized via biome.json
fn supports_negative(utility: &str, config: &TailwindAnalyzerConfig) -> bool {
    tailwind_utils::supports_negative(utility, config.negatable.as_deref())
}

/// Find a replacement for an arbitrary value class
fn find_replacement(class: &str, config: &TailwindAnalyzerConfig) -> Option<(String, String)> {
    // Extract variants and utility
    let (variants, utility) = extract_variants_and_utility(class);

    // Check if it contains an arbitrary value
    if !utility.contains('[') || !utility.contains(']') {
        return None;
    }

    // Capture important modifier position before extraction
    let important_prefix = utility.starts_with('!');
    let important_suffix = utility.ends_with('!');

    // Extract prefix and arbitrary value
    let (prefix, arbitrary) = extract_prefix_and_arbitrary(utility)?;
    let arbitrary = arbitrary.trim();

    // Check if the value is negative
    let (is_negative, positive_value) = if let Some(rest) = arbitrary.strip_prefix('-') {
        (true, rest)
    } else {
        (false, arbitrary)
    };

    // Build important modifier strings
    let important_pre = if important_prefix { "!" } else { "" };
    let important_suf = if important_suffix { "!" } else { "" };

    // Try to find a replacement based on the utility prefix
    // Uses custom scales from biome.json tailwind config if available
    let replacement_value: String = match prefix {
        // Spacing utilities
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "ps" | "pe" | "m" | "mx" | "my" | "mt"
        | "mr" | "mb" | "ml" | "ms" | "me" | "gap" | "gap-x" | "gap-y" | "space-x" | "space-y"
        | "inset" | "inset-x" | "inset-y" | "top" | "right" | "bottom" | "left" | "start"
        | "end" | "scroll-m" | "scroll-mx" | "scroll-my" | "scroll-mt" | "scroll-mr"
        | "scroll-mb" | "scroll-ml" | "translate-x" | "translate-y" | "indent" => {
            tailwind_utils::lookup_spacing(positive_value, config.spacing.as_ref())
        }

        // Width/Height (don't support negative prefix form)
        "w" | "h" | "size" | "min-w" | "max-w" | "min-h" | "max-h" => {
            if is_negative {
                return None; // Width/height don't have negative prefix form
            }
            tailwind_utils::lookup_spacing(positive_value, config.spacing.as_ref())
                .or_else(|| lookup_size_keyword(positive_value).map(String::from))
        }

        // Opacity
        "opacity" => {
            if is_negative {
                return None; // Opacity doesn't support negative
            }
            tailwind_utils::lookup_opacity(positive_value, config.opacity.as_ref())
        }

        // Font size
        "text" => {
            if is_negative {
                return None; // Text size doesn't support negative
            }
            // Only replace if it looks like a size value (rem/px), not a color
            if positive_value.ends_with("rem") || positive_value.ends_with("px") {
                tailwind_utils::lookup_font_size(positive_value, config.font_size.as_ref())
            } else {
                None
            }
        }

        // Border radius
        "rounded" | "rounded-t" | "rounded-r" | "rounded-b" | "rounded-l" | "rounded-tl"
        | "rounded-tr" | "rounded-br" | "rounded-bl" | "rounded-s" | "rounded-e" | "rounded-ss"
        | "rounded-se" | "rounded-es" | "rounded-ee" => {
            if is_negative {
                return None; // Border radius doesn't support negative
            }
            let value = tailwind_utils::lookup_border_radius(
                positive_value,
                config.border_radius.as_ref(),
            )?;
            // Handle DEFAULT case
            if value == "DEFAULT" {
                return Some((class.to_string(), format!("{}{}", variants, prefix)));
            }
            Some(value)
        }

        // Z-index
        "z" => tailwind_utils::lookup_z_index(positive_value, config.z_index.as_ref()),

        // Rotate, skew, scale, order
        "rotate" | "skew-x" | "skew-y" | "scale" | "scale-x" | "scale-y" | "order" => {
            // These use numeric values, check if it's a standard scale value
            None // Would need additional scale maps for these
        }

        _ => None,
    }?;

    // Build the replacement class
    let replacement = if is_negative && supports_negative(prefix, config) {
        // Negative value with negatable utility: m-[-1rem] → -m-4
        // Important goes after variants but before the negative dash
        format!(
            "{}{}-{}-{}{}",
            variants, important_pre, prefix, replacement_value, important_suf
        )
    } else {
        format!(
            "{}{}{}-{}{}",
            variants, important_pre, prefix, replacement_value, important_suf
        )
    };

    Some((class.to_string(), replacement))
}

/// Extract variants and utility from a class
fn extract_variants_and_utility(class: &str) -> (&str, &str) {
    if let Some(last_colon_idx) = class.rfind(':') {
        (&class[..=last_colon_idx], &class[last_colon_idx + 1..])
    } else {
        ("", class)
    }
}

/// Extract prefix and arbitrary value from a utility
/// e.g., "m-[1rem]" -> ("m", "1rem")
fn extract_prefix_and_arbitrary(utility: &str) -> Option<(&str, &str)> {
    // Handle important modifier
    let utility = utility.trim_start_matches('!').trim_end_matches('!');

    // Find the bracket
    let bracket_start = utility.find('[')?;
    let bracket_end = utility.find(']')?;

    if bracket_start >= bracket_end {
        return None;
    }

    // Get the prefix (everything before -[)
    let prefix = if bracket_start > 0 && utility.as_bytes()[bracket_start - 1] == b'-' {
        &utility[..bracket_start - 1]
    } else {
        return None;
    };

    // Get the arbitrary value (inside brackets)
    let arbitrary = &utility[bracket_start + 1..bracket_end];

    Some((prefix, arbitrary))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TailwindAnalyzerConfig {
        TailwindAnalyzerConfig::default()
    }

    #[test]
    fn test_extract_prefix_and_arbitrary() {
        assert_eq!(
            extract_prefix_and_arbitrary("m-[1rem]"),
            Some(("m", "1rem"))
        );
        assert_eq!(
            extract_prefix_and_arbitrary("px-[16px]"),
            Some(("px", "16px"))
        );
        assert_eq!(
            extract_prefix_and_arbitrary("w-[100%]"),
            Some(("w", "100%"))
        );
        assert_eq!(
            extract_prefix_and_arbitrary("opacity-[0.5]"),
            Some(("opacity", "0.5"))
        );
        assert_eq!(extract_prefix_and_arbitrary("flex"), None);
        assert_eq!(extract_prefix_and_arbitrary("[mask:url()]"), None);
    }

    #[test]
    fn test_find_replacement() {
        let config = default_config();
        // Spacing
        assert_eq!(
            find_replacement("m-[1rem]", &config),
            Some(("m-[1rem]".to_string(), "m-4".to_string()))
        );
        assert_eq!(
            find_replacement("p-[16px]", &config),
            Some(("p-[16px]".to_string(), "p-4".to_string()))
        );

        // Width/height
        assert_eq!(
            find_replacement("w-[100%]", &config),
            Some(("w-[100%]".to_string(), "w-full".to_string()))
        );

        // Opacity
        assert_eq!(
            find_replacement("opacity-[0.5]", &config),
            Some(("opacity-[0.5]".to_string(), "opacity-50".to_string()))
        );

        // No replacement available
        assert_eq!(find_replacement("w-[137px]", &config), None);
        assert_eq!(find_replacement("text-[#ff0000]", &config), None);
    }

    #[test]
    fn test_negative_value_conversion() {
        let config = default_config();
        // Negative spacing values should use dash prefix form
        assert_eq!(
            find_replacement("m-[-1rem]", &config),
            Some(("m-[-1rem]".to_string(), "-m-4".to_string()))
        );
        assert_eq!(
            find_replacement("mt-[-16px]", &config),
            Some(("mt-[-16px]".to_string(), "-mt-4".to_string()))
        );
        assert_eq!(
            find_replacement("top-[-1rem]", &config),
            Some(("top-[-1rem]".to_string(), "-top-4".to_string()))
        );
        assert_eq!(
            find_replacement("translate-x-[-1rem]", &config),
            Some((
                "translate-x-[-1rem]".to_string(),
                "-translate-x-4".to_string()
            ))
        );

        // Negative z-index
        assert_eq!(
            find_replacement("z-[-10]", &config),
            Some(("z-[-10]".to_string(), "-z-10".to_string()))
        );

        // Width/height don't support negative prefix
        assert_eq!(find_replacement("w-[-100px]", &config), None);
        assert_eq!(find_replacement("h-[-50px]", &config), None);

        // Opacity doesn't support negative
        assert_eq!(find_replacement("opacity-[-0.5]", &config), None);
    }

    #[test]
    fn test_with_variants() {
        let config = default_config();
        assert_eq!(
            find_replacement("hover:m-[1rem]", &config),
            Some(("hover:m-[1rem]".to_string(), "hover:m-4".to_string()))
        );
        assert_eq!(
            find_replacement("dark:md:p-[16px]", &config),
            Some(("dark:md:p-[16px]".to_string(), "dark:md:p-4".to_string()))
        );
        // Negative with variants
        assert_eq!(
            find_replacement("hover:m-[-1rem]", &config),
            Some(("hover:m-[-1rem]".to_string(), "hover:-m-4".to_string()))
        );
    }

    #[test]
    fn test_with_important_modifier() {
        let config = default_config();
        // Important prefix
        assert_eq!(
            find_replacement("!m-[1rem]", &config),
            Some(("!m-[1rem]".to_string(), "!m-4".to_string()))
        );

        // Important suffix
        assert_eq!(
            find_replacement("m-[1rem]!", &config),
            Some(("m-[1rem]!".to_string(), "m-4!".to_string()))
        );

        // Important with variants
        assert_eq!(
            find_replacement("hover:!m-[1rem]", &config),
            Some(("hover:!m-[1rem]".to_string(), "hover:!m-4".to_string()))
        );

        // Important with negative value
        assert_eq!(
            find_replacement("!m-[-1rem]", &config),
            Some(("!m-[-1rem]".to_string(), "!-m-4".to_string()))
        );

        // Important suffix with negative value
        assert_eq!(
            find_replacement("m-[-1rem]!", &config),
            Some(("m-[-1rem]!".to_string(), "-m-4!".to_string()))
        );

        // Important with variants and negative value
        assert_eq!(
            find_replacement("hover:!m-[-1rem]", &config),
            Some(("hover:!m-[-1rem]".to_string(), "hover:!-m-4".to_string()))
        );
    }

    #[test]
    fn test_with_custom_spacing() {
        use std::collections::BTreeMap;
        let mut spacing = BTreeMap::new();
        spacing.insert("3.25rem".to_string(), "13".to_string());

        let config = TailwindAnalyzerConfig {
            spacing: Some(spacing),
            ..Default::default()
        };

        // Custom spacing value should be recognized
        assert_eq!(
            find_replacement("m-[3.25rem]", &config),
            Some(("m-[3.25rem]".to_string(), "m-13".to_string()))
        );

        // Default values should still work
        assert_eq!(
            find_replacement("m-[1rem]", &config),
            Some(("m-[1rem]".to_string(), "m-4".to_string()))
        );
    }
}
