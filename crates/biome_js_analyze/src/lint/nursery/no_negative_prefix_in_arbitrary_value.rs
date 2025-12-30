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
    /// Disallow using the negative dash prefix with arbitrary values.
    ///
    /// Tailwind CSS allows negative values in two ways:
    /// 1. Using the negative prefix: `-m-4`
    /// 2. Using negative arbitrary values: `m-[-1rem]`
    ///
    /// When using arbitrary values, it's clearer to put the negative sign inside
    /// the brackets rather than using the dash prefix. This rule enforces moving
    /// the negative sign inside the brackets for consistency and clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// // Dash prefix with arbitrary value should use negative inside brackets
    /// <div class="-top-[10px]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// // Double negative is confusing
    /// <div class="-m-[-1rem]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// // CSS variables should use calc() for negation
    /// <div class="-left-[var(--spacing)]" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// // Negative value inside brackets (preferred for arbitrary values)
    /// <div class="top-[-10px]" />;
    /// <div class="m-[-1rem]" />;
    /// <div class="left-[calc(var(--spacing)*-1)]" />;
    /// ```
    ///
    /// ```jsx
    /// // Dash prefix with standard values is fine
    /// <div class="-m-4" />;
    /// <div class="-translate-x-1/2" />;
    /// ```
    ///
    pub NoNegativePrefixInArbitraryValue {
        version: "next",
        name: "noNegativePrefixInArbitraryValue",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// Check if a utility supports negative values
/// Uses the shared tailwind_utils module which can be customized via biome.json
fn supports_negative(utility: &str, config: &TailwindAnalyzerConfig) -> bool {
    tailwind_utils::supports_negative(utility, config.negatable.as_deref())
}

/// Check if a class has a dash prefix with an arbitrary value
/// e.g., "-top-[10px]" or "-top-[-10px]" returns Some((variants, utility, value, important_prefix, important_suffix))
fn has_dash_prefix_arbitrary<'a>(
    class: &'a str,
    config: &TailwindAnalyzerConfig,
) -> Option<(&'a str, &'a str, &'a str, bool, bool)> {
    // Split off variants (e.g., "hover:md:-top-[10px]" -> "hover:md:", "-top-[10px]")
    let (variants, base_class) = if let Some(bracket_pos) = class.find('[') {
        let before_bracket = &class[..bracket_pos];
        if let Some(last_colon) = before_bracket.rfind(':') {
            (&class[..=last_colon], &class[last_colon + 1..])
        } else {
            ("", class)
        }
    } else {
        return None;
    };

    // Handle important modifier
    let important_prefix = base_class.starts_with('!');
    let important_suffix = base_class.ends_with('!');

    // Strip important modifiers for analysis
    let base_class = base_class.trim_start_matches('!').trim_end_matches('!');

    // Check if base class starts with dash (negative prefix)
    if !base_class.starts_with('-') {
        return None;
    }

    // Find the bracket to extract utility and value
    let bracket_start = base_class.find('[')?;
    let bracket_end = base_class.rfind(']')?;

    if bracket_start >= bracket_end {
        return None;
    }

    // Extract the utility (between the leading dash and the bracket)
    // e.g., "-top-[10px]" -> "top"
    let utility_with_dash = &base_class[1..bracket_start]; // "top-"
    let utility = utility_with_dash
        .strip_suffix('-')
        .unwrap_or(utility_with_dash);

    // Check if this utility supports negative values
    if !supports_negative(utility, config) {
        return None;
    }

    // Extract the value inside brackets
    let value = &base_class[bracket_start + 1..bracket_end];

    Some((variants, utility, value, important_prefix, important_suffix))
}

/// Build the fixed class by removing dash prefix and negating the value
fn fix_dash_prefix_arbitrary(
    variants: &str,
    utility: &str,
    value: &str,
    important_prefix: bool,
    important_suffix: bool,
) -> String {
    let negated_value = negate_value(value);
    let prefix = if important_prefix { "!" } else { "" };
    let suffix = if important_suffix { "!" } else { "" };
    format!(
        "{}{}{}-[{}]{}",
        variants, prefix, utility, negated_value, suffix
    )
}

/// Negate an arbitrary value
/// - If already negative (starts with -), remove the negative (double negative)
/// - If it's a CSS variable or calc, wrap in calc(...*-1)
/// - Otherwise, prepend a negative sign
fn negate_value(value: &str) -> String {
    let trimmed = value.trim();

    // Double negative: -10px -> 10px (remove the leading minus)
    if let Some(rest) = trimmed.strip_prefix('-') {
        return rest.to_string();
    }

    // CSS variable: var(--x) -> calc(var(--x)*-1)
    if trimmed.starts_with("var(") {
        return format!("calc({}*-1)", trimmed);
    }

    // Already wrapped in calc: calc(...) -> calc((...)*-1)
    if trimmed.starts_with("calc(") {
        return format!("calc(({})*-1)", &trimmed[5..trimmed.len() - 1]);
    }

    // Simple positive value: 10px -> -10px
    format!("-{}", trimmed)
}

/// State containing the fixed class string and the violations found
pub struct DashPrefixArbitraryState {
    /// The fixed class string
    pub fixed: Box<str>,
    /// List of violations (original -> fixed)
    pub violations: Vec<(String, String)>,
}

impl Rule for NoNegativePrefixInArbitraryValue {
    type Query = Ast<AnyClassStringLike>;
    type State = DashPrefixArbitraryState;
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

        let mut violations = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            if let Some((variants, utility, arb_value, important_prefix, important_suffix)) =
                has_dash_prefix_arbitrary(class, tailwind_config)
            {
                let fixed_class = fix_dash_prefix_arbitrary(
                    variants,
                    utility,
                    arb_value,
                    important_prefix,
                    important_suffix,
                );
                violations.push((class.to_string(), fixed_class.clone()));
                fixed_parts.push(fixed_class);
            } else {
                fixed_parts.push(class.to_string());
            }
        }

        if violations.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(DashPrefixArbitraryState {
            fixed: fixed.into(),
            violations,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let violations_str = state
            .violations
            .iter()
            .map(|(from, to)| format!("`{}` → `{}`", from, to))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Avoid using the dash prefix with arbitrary values."
                },
            )
            .note(markup! {
                "Replace: "{violations_str}
            })
            .note(markup! {
                "Move the negative sign inside the brackets for clarity."
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
                "Move the negative sign inside the brackets."
            }
            .to_owned(),
            mutation,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TailwindAnalyzerConfig {
        TailwindAnalyzerConfig::default()
    }

    #[test]
    fn test_has_dash_prefix_arbitrary() {
        let config = default_config();
        // Should detect dash prefix with ANY arbitrary value
        assert!(has_dash_prefix_arbitrary("-top-[-10px]", &config).is_some()); // double negative
        assert!(has_dash_prefix_arbitrary("-top-[10px]", &config).is_some()); // positive value
        assert!(has_dash_prefix_arbitrary("-m-[-1rem]", &config).is_some());
        assert!(has_dash_prefix_arbitrary("-m-[1rem]", &config).is_some());
        assert!(has_dash_prefix_arbitrary("-translate-x-[-50%]", &config).is_some());
        assert!(has_dash_prefix_arbitrary("-left-[var(--spacing)]", &config).is_some()); // CSS var

        // Should not flag non-dash-prefix classes
        assert!(has_dash_prefix_arbitrary("top-[-10px]", &config).is_none()); // No dash prefix
        assert!(has_dash_prefix_arbitrary("top-[10px]", &config).is_none()); // No dash prefix
        assert!(has_dash_prefix_arbitrary("-m-4", &config).is_none()); // No arbitrary value
        assert!(has_dash_prefix_arbitrary("m-[-1rem]", &config).is_none()); // No dash prefix
    }

    #[test]
    fn test_has_dash_prefix_arbitrary_with_important() {
        let config = default_config();
        // Important prefix
        let result = has_dash_prefix_arbitrary("!-top-[10px]", &config);
        assert!(result.is_some());
        let (variants, utility, value, important_prefix, important_suffix) = result.unwrap();
        assert_eq!(variants, "");
        assert_eq!(utility, "top");
        assert_eq!(value, "10px");
        assert!(important_prefix);
        assert!(!important_suffix);

        // Important suffix
        let result = has_dash_prefix_arbitrary("-top-[10px]!", &config);
        assert!(result.is_some());
        let (_, _, _, important_prefix, important_suffix) = result.unwrap();
        assert!(!important_prefix);
        assert!(important_suffix);

        // With variants and important prefix
        let result = has_dash_prefix_arbitrary("hover:!-m-[1rem]", &config);
        assert!(result.is_some());
        let (variants, utility, value, important_prefix, important_suffix) = result.unwrap();
        assert_eq!(variants, "hover:");
        assert_eq!(utility, "m");
        assert_eq!(value, "1rem");
        assert!(important_prefix);
        assert!(!important_suffix);
    }

    #[test]
    fn test_negate_value() {
        // Double negative (remove the minus)
        assert_eq!(negate_value("-10px"), "10px");
        assert_eq!(negate_value("-1rem"), "1rem");
        assert_eq!(negate_value("-50%"), "50%");

        // Positive value (add minus)
        assert_eq!(negate_value("10px"), "-10px");
        assert_eq!(negate_value("1rem"), "-1rem");
        assert_eq!(negate_value("50%"), "-50%");

        // CSS variable (wrap in calc)
        assert_eq!(negate_value("var(--x)"), "calc(var(--x)*-1)");
        assert_eq!(negate_value("var(--spacing)"), "calc(var(--spacing)*-1)");

        // Already calc (wrap the inner expression)
        assert_eq!(negate_value("calc(100%-1rem)"), "calc((100%-1rem)*-1)");
    }

    #[test]
    fn test_fix_dash_prefix_arbitrary() {
        // Double negative -> remove prefix and minus
        assert_eq!(
            fix_dash_prefix_arbitrary("", "top", "-10px", false, false),
            "top-[10px]"
        );
        assert_eq!(
            fix_dash_prefix_arbitrary("", "m", "-1rem", false, false),
            "m-[1rem]"
        );

        // Positive value -> remove prefix and add minus
        assert_eq!(
            fix_dash_prefix_arbitrary("", "top", "10px", false, false),
            "top-[-10px]"
        );
        assert_eq!(
            fix_dash_prefix_arbitrary("", "m", "1rem", false, false),
            "m-[-1rem]"
        );

        // CSS variable -> wrap in calc
        assert_eq!(
            fix_dash_prefix_arbitrary("", "left", "var(--x)", false, false),
            "left-[calc(var(--x)*-1)]"
        );

        // With variants
        assert_eq!(
            fix_dash_prefix_arbitrary("hover:", "translate-x", "50%", false, false),
            "hover:translate-x-[-50%]"
        );

        // With important prefix
        assert_eq!(
            fix_dash_prefix_arbitrary("", "top", "10px", true, false),
            "!top-[-10px]"
        );

        // With important suffix
        assert_eq!(
            fix_dash_prefix_arbitrary("", "top", "10px", false, true),
            "top-[-10px]!"
        );

        // With variants and important prefix
        assert_eq!(
            fix_dash_prefix_arbitrary("hover:", "m", "1rem", true, false),
            "hover:!m-[-1rem]"
        );
    }

    #[test]
    fn test_with_variants() {
        let config = default_config();
        let result = has_dash_prefix_arbitrary("hover:-m-[1rem]", &config);
        assert!(result.is_some());
        let (variants, utility, value, important_prefix, important_suffix) = result.unwrap();
        assert_eq!(variants, "hover:");
        assert_eq!(utility, "m");
        assert_eq!(value, "1rem");
        assert!(!important_prefix);
        assert!(!important_suffix);
    }

    #[test]
    fn test_with_custom_negatable() {
        // Test that custom negatable utilities are recognized
        let config = TailwindAnalyzerConfig {
            negatable: Some(vec!["custom-util".to_string()]),
            ..Default::default()
        };

        // Custom utility should be recognized as negatable
        assert!(has_dash_prefix_arbitrary("-custom-util-[10px]", &config).is_some());

        // Default negatable utilities should still work
        assert!(has_dash_prefix_arbitrary("-m-[1rem]", &config).is_some());
    }
}
