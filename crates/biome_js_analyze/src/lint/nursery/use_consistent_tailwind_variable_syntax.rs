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
use regex::Regex;
use std::sync::LazyLock;

declare_lint_rule! {
    /// Enforce consistent CSS variable syntax in Tailwind CSS arbitrary values.
    ///
    /// Tailwind CSS supports two syntaxes for CSS variables in arbitrary values:
    /// - Modern syntax (Tailwind v3.1+): `bg-[--my-color]` or `text-[--brand]`
    /// - Legacy syntax: `bg-[var(--my-color)]` or `text-[var(--brand)]`
    ///
    /// This rule enforces the modern, shorter syntax for consistency and brevity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="bg-[var(--my-color)]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="text-[var(--brand-primary)]" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="bg-[--my-color]" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="text-[--brand-primary]" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="w-[calc(100%-var(--sidebar))]" />;
    /// ```
    ///
    pub UseConsistentTailwindVariableSyntax {
        version: "next",
        name: "useConsistentTailwindVariableSyntax",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

// Regex to match var(--name) that is the ONLY content in brackets (not part of calc, etc.)
static VAR_ONLY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[var\((--[a-zA-Z0-9_-]+)\)\]").unwrap());

/// State containing the fixed class string and the classes that need fixing
pub struct VariableSyntaxState {
    /// The fixed class string with consistent variable syntax
    pub fixed: Box<str>,
    /// List of classes that had inconsistent variable syntax
    pub inconsistent_classes: Vec<(String, String)>, // (original, fixed)
}

impl Rule for UseConsistentTailwindVariableSyntax {
    type Query = Ast<AnyClassStringLike>;
    type State = VariableSyntaxState;
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

        let mut inconsistent_classes = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            if let Some(fixed_class) = fix_variable_syntax(class) {
                inconsistent_classes.push((class.to_string(), fixed_class.clone()));
                fixed_parts.push(fixed_class);
            } else {
                fixed_parts.push(class.to_string());
            }
        }

        if inconsistent_classes.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(VariableSyntaxState {
            fixed: fixed.into(),
            inconsistent_classes,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let classes_str = state
            .inconsistent_classes
            .iter()
            .map(|(old, new)| format!("`{}` → `{}`", old, new))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Use modern CSS variable syntax in Tailwind classes."
                },
            )
            .note(markup! {
                "Replace: "{classes_str}
            })
            .note(markup! {
                "The modern syntax `[--var-name]` is shorter and cleaner than `[var(--var-name)]`."
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
                "Use modern CSS variable syntax."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Fix var(--name) to --name in arbitrary values
/// Only fixes when var() is the sole content of the brackets
fn fix_variable_syntax(class: &str) -> Option<String> {
    if !class.contains("[var(--") {
        return None;
    }

    // Check if this is a simple var() usage (not inside calc, etc.)
    if VAR_ONLY_PATTERN.is_match(class) {
        let fixed = VAR_ONLY_PATTERN.replace_all(class, "[${1}]").to_string();
        if fixed != class {
            return Some(fixed);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_variable_syntax() {
        // Should fix simple var() usage
        assert_eq!(
            fix_variable_syntax("bg-[var(--my-color)]"),
            Some("bg-[--my-color]".to_string())
        );
        assert_eq!(
            fix_variable_syntax("text-[var(--brand-primary)]"),
            Some("text-[--brand-primary]".to_string())
        );

        // Should NOT fix when var() is part of a larger expression
        assert_eq!(fix_variable_syntax("w-[calc(100%-var(--sidebar))]"), None);
        assert_eq!(fix_variable_syntax("h-[calc(var(--header)+20px)]"), None);

        // Should NOT touch classes without var()
        assert_eq!(fix_variable_syntax("bg-[--my-color]"), None);
        assert_eq!(fix_variable_syntax("text-red-500"), None);
        assert_eq!(fix_variable_syntax("p-4"), None);
    }

    #[test]
    fn test_with_variants() {
        assert_eq!(
            fix_variable_syntax("hover:bg-[var(--hover-color)]"),
            Some("hover:bg-[--hover-color]".to_string())
        );
        assert_eq!(
            fix_variable_syntax("dark:text-[var(--dark-text)]"),
            Some("dark:text-[--dark-text]".to_string())
        );
    }
}
