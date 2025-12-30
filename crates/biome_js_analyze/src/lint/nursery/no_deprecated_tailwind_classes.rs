use crate::JsRuleAction;
use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_factory::make::{
    js_literal_member_name, js_string_literal, js_string_literal_expression,
    js_string_literal_single_quotes, js_template_chunk, js_template_chunk_element, jsx_string,
};
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::use_sorted_classes::UseSortedClassesOptions;
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

declare_lint_rule! {
    /// Disallow deprecated Tailwind CSS classes and suggest modern replacements.
    ///
    /// Tailwind CSS has deprecated certain utility classes in favor of more consistent naming.
    /// This rule detects deprecated classes and provides automatic fixes to update them
    /// to their modern equivalents.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="flex-shrink" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="flex-grow-0" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="overflow-ellipsis" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="shrink" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="grow-0" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="text-ellipsis" />;
    /// ```
    ///
    pub NoDeprecatedTailwindClasses {
        version: "next",
        name: "noDeprecatedTailwindClasses",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// Map of deprecated class patterns to their modern replacements
static DEPRECATED_CLASSES: LazyLock<FxHashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();

    // ========================================
    // Tailwind v3 deprecations (flex utilities)
    // ========================================
    map.insert("flex-shrink-0", "shrink-0");
    map.insert("flex-shrink", "shrink");
    map.insert("flex-grow-0", "grow-0");
    map.insert("flex-grow", "grow");

    // ========================================
    // Text overflow utilities renamed
    // ========================================
    map.insert("overflow-ellipsis", "text-ellipsis");
    map.insert("overflow-clip", "text-clip");

    // ========================================
    // Box decoration break renamed
    // ========================================
    map.insert("decoration-slice", "box-decoration-slice");
    map.insert("decoration-clone", "box-decoration-clone");

    // ========================================
    // Tailwind v2 → v3 color renames
    // ========================================
    // Gray renamed to Slate/Gray/Zinc/Neutral/Stone
    // These are the old gray-* that should become slate-* or gray-*
    // Note: Tailwind v3 kept gray but changed the shades, so direct
    // replacements depend on the design system being used.

    // ========================================
    // Transform utilities (v2 → v3)
    // In v3, transform is applied automatically, these are deprecated
    // ========================================
    map.insert("transform", ""); // No longer needed in v3 (implicit)
    map.insert("transform-gpu", ""); // No longer needed
    map.insert("transform-none", ""); // Still exists but rarely needed

    // ========================================
    // Filter utilities (v2 → v3)
    // In v3, filter is applied automatically
    // ========================================
    map.insert("filter", ""); // No longer needed in v3 (implicit)
    map.insert("filter-none", ""); // Rarely needed

    // ========================================
    // Backdrop filter utilities (v2 → v3)
    // ========================================
    map.insert("backdrop-filter", ""); // No longer needed in v3 (implicit)
    map.insert("backdrop-filter-none", ""); // Rarely needed

    // ========================================
    // Deprecated blur notation (if using old syntax)
    // ========================================

    // ========================================
    // Ring offset renamed in some contexts
    // ========================================

    // ========================================
    // Placeholder opacity (v2 used separate classes)
    // In v3, use placeholder:text-opacity-* or just placeholder:opacity-*
    // ========================================
    map.insert("placeholder-opacity-0", "placeholder:opacity-0");
    map.insert("placeholder-opacity-5", "placeholder:opacity-5");
    map.insert("placeholder-opacity-10", "placeholder:opacity-10");
    map.insert("placeholder-opacity-20", "placeholder:opacity-20");
    map.insert("placeholder-opacity-25", "placeholder:opacity-25");
    map.insert("placeholder-opacity-30", "placeholder:opacity-30");
    map.insert("placeholder-opacity-40", "placeholder:opacity-40");
    map.insert("placeholder-opacity-50", "placeholder:opacity-50");
    map.insert("placeholder-opacity-60", "placeholder:opacity-60");
    map.insert("placeholder-opacity-70", "placeholder:opacity-70");
    map.insert("placeholder-opacity-75", "placeholder:opacity-75");
    map.insert("placeholder-opacity-80", "placeholder:opacity-80");
    map.insert("placeholder-opacity-90", "placeholder:opacity-90");
    map.insert("placeholder-opacity-95", "placeholder:opacity-95");
    map.insert("placeholder-opacity-100", "placeholder:opacity-100");

    // ========================================
    // Background opacity (v2 → v3)
    // In v3, use bg-{color}/{opacity} syntax
    // These are listed for awareness; direct replacement requires context
    // ========================================

    // ========================================
    // Text opacity (v2 → v3)
    // In v3, use text-{color}/{opacity} syntax
    // ========================================

    // ========================================
    // Border opacity (v2 → v3)
    // In v3, use border-{color}/{opacity} syntax
    // ========================================

    // ========================================
    // Ring opacity is now part of ring-{color}/{opacity}
    // ========================================

    // ========================================
    // Divide opacity similar to border opacity
    // ========================================

    map
});

/// State containing the fixed class string and the deprecated classes found
pub struct DeprecatedClassState {
    /// The fixed class string with deprecated classes replaced
    pub fixed: Box<str>,
    /// List of deprecated classes and their replacements
    pub deprecated_classes: Vec<(String, String)>,
}

impl Rule for NoDeprecatedTailwindClasses {
    type Query = Ast<AnyClassStringLike>;
    type State = DeprecatedClassState;
    type Signals = Option<Self::State>;
    type Options = UseSortedClassesOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        if !node.should_visit(options)? {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();

        let mut deprecated_classes = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            // Check if the class (without variants) is deprecated
            let (variants, utility) = extract_variants_and_utility(class);

            if let Some(replacement) = DEPRECATED_CLASSES.get(utility) {
                if replacement.is_empty() {
                    // Empty replacement means the class should be removed entirely
                    deprecated_classes.push((class.to_string(), "(removed)".to_string()));
                    // Don't add anything to fixed_parts - this effectively removes the class
                } else {
                    deprecated_classes
                        .push((class.to_string(), format!("{}{}", variants, replacement)));
                    fixed_parts.push(format!("{}{}", variants, replacement));
                }
            } else {
                fixed_parts.push(class.to_string());
            }
        }

        if deprecated_classes.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(DeprecatedClassState {
            fixed: fixed.into(),
            deprecated_classes,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let classes_str = state
            .deprecated_classes
            .iter()
            .map(|(old, new)| format!("`{}` → `{}`", old, new))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Deprecated Tailwind CSS class(es) found: "{classes_str}"."
                },
            )
            .note(markup! {
                "Use the modern replacement class(es) for better compatibility."
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
                "Replace with modern Tailwind CSS class(es)."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Extract variants (like "hover:", "focus:") and the utility class
fn extract_variants_and_utility(class: &str) -> (&str, &str) {
    if let Some(last_colon_idx) = class.rfind(':') {
        (&class[..=last_colon_idx], &class[last_colon_idx + 1..])
    } else {
        ("", class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variants_and_utility() {
        assert_eq!(
            extract_variants_and_utility("flex-shrink"),
            ("", "flex-shrink")
        );
        assert_eq!(
            extract_variants_and_utility("hover:flex-shrink"),
            ("hover:", "flex-shrink")
        );
        assert_eq!(
            extract_variants_and_utility("focus:hover:flex-grow"),
            ("focus:hover:", "flex-grow")
        );
    }

    #[test]
    fn test_deprecated_class_detection() {
        assert!(DEPRECATED_CLASSES.contains_key("flex-shrink"));
        assert!(DEPRECATED_CLASSES.contains_key("flex-grow"));
        assert!(DEPRECATED_CLASSES.contains_key("overflow-ellipsis"));
        assert!(!DEPRECATED_CLASSES.contains_key("shrink"));
        assert!(!DEPRECATED_CLASSES.contains_key("grow"));
    }
}
