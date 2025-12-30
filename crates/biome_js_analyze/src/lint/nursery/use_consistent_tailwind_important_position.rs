use crate::JsRuleAction;
use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_factory::make::{
    js_literal_member_name, js_string_literal, js_string_literal_expression,
    js_string_literal_single_quotes, js_template_chunk, js_template_chunk_element, jsx_string,
};
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::use_consistent_tailwind_important_position::{
    ImportantPosition, UseConsistentTailwindImportantPositionOptions,
};

type Options = UseConsistentTailwindImportantPositionOptions;

declare_lint_rule! {
    /// Enforce consistent placement of the important modifier (`!`) in Tailwind CSS classes.
    ///
    /// In Tailwind CSS, the important modifier can be placed either at the start or end of a class:
    /// - Start: `!text-red-500`
    /// - End: `text-red-500!`
    ///
    /// This rule enforces a consistent position across all utility classes.
    ///
    /// By default, this rule enforces the important modifier at the **start** of the class,
    /// which is the recommended style in Tailwind CSS v4+.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// With default options (position: "start"):
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="text-red-500!" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="hover:bg-blue-500!" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// With default options (position: "start"):
    ///
    /// ```jsx
    /// <div class="!text-red-500" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="hover:!bg-blue-500" />;
    /// ```
    ///
    /// ## Options
    ///
    /// ### position
    ///
    /// The `position` option controls where the important modifier should be placed.
    ///
    /// - `"start"` (default): The important modifier should be at the start of the utility
    /// - `"end"`: The important modifier should be at the end of the class
    ///
    /// This rule also inherits options from [`useSortedClasses`](/linter/rules/use-sorted-classes)
    /// to control which attributes and functions are checked.
    ///
    pub UseConsistentTailwindImportantPosition {
        version: "next",
        name: "useConsistentTailwindImportantPosition",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// State containing the fixed class string and the classes that need fixing
pub struct ImportantPositionState {
    /// The fixed class string with consistent important positions
    pub fixed: Box<str>,
    /// List of classes that had inconsistent important positions
    pub inconsistent_classes: Vec<String>,
}

impl Rule for UseConsistentTailwindImportantPosition {
    type Query = Ast<AnyClassStringLike>;
    type State = ImportantPositionState;
    type Signals = Option<Self::State>;
    type Options = Options;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        if !node.should_visit(options)? {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();
        // Get position from options, defaulting to "start" (Tailwind v4+ style)
        let position = options.position();

        let mut inconsistent_classes = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            let fixed_class = fix_important_position(class, position);
            if fixed_class != class {
                inconsistent_classes.push(class.to_string());
            }
            fixed_parts.push(fixed_class);
        }

        if inconsistent_classes.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(ImportantPositionState {
            fixed: fixed.into(),
            inconsistent_classes,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        let options = ctx.options();
        let position_str = options.position().as_str();

        let classes_str = state
            .inconsistent_classes
            .iter()
            .map(|s| format!("`{}`", s))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Inconsistent important modifier position in: "{classes_str}"."
                },
            )
            .note(markup! {
                "The important modifier should be at the "{position_str}" of the utility class."
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
                "Move important modifier to consistent position."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Fix the important position of a single class
fn fix_important_position(class: &str, target_position: ImportantPosition) -> String {
    // Check for important modifier anywhere in the class
    let has_important_at_end = class.ends_with('!');

    // Check for important after variant or at start
    let has_important_after_variant = class.contains(":!");
    let has_important_at_start = class.starts_with('!');

    // If no important modifier anywhere, return as-is
    if !has_important_at_start && !has_important_at_end && !has_important_after_variant {
        return class.to_string();
    }

    // Extract the base parts: variants and utility
    let class_no_end_bang = class.trim_end_matches('!');

    // Find the utility part (after the last colon, or the whole thing if no colon)
    let (variants, utility) = if let Some(last_colon_idx) = class_no_end_bang.rfind(':') {
        let v = &class_no_end_bang[..=last_colon_idx];
        let u = &class_no_end_bang[last_colon_idx + 1..];
        (v.to_string(), u.trim_start_matches('!').to_string())
    } else {
        (
            String::new(),
            class_no_end_bang.trim_start_matches('!').to_string(),
        )
    };

    match target_position {
        ImportantPosition::Start => {
            if variants.is_empty() {
                format!("!{}", utility)
            } else {
                format!("{}!{}", variants, utility)
            }
        }
        ImportantPosition::End => {
            format!("{}{}!", variants, utility)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_important_position_start() {
        // From end to start
        assert_eq!(
            fix_important_position("text-red-500!", ImportantPosition::Start),
            "!text-red-500"
        );
        // Already at start
        assert_eq!(
            fix_important_position("!text-red-500", ImportantPosition::Start),
            "!text-red-500"
        );
        // With variants, from end to start
        assert_eq!(
            fix_important_position("hover:text-red-500!", ImportantPosition::Start),
            "hover:!text-red-500"
        );
        // With variants, already at start
        assert_eq!(
            fix_important_position("hover:!text-red-500", ImportantPosition::Start),
            "hover:!text-red-500"
        );
        // No important modifier
        assert_eq!(
            fix_important_position("text-red-500", ImportantPosition::Start),
            "text-red-500"
        );
    }

    #[test]
    fn test_fix_important_position_end() {
        // From start to end
        assert_eq!(
            fix_important_position("!text-red-500", ImportantPosition::End),
            "text-red-500!"
        );
        // Already at end
        assert_eq!(
            fix_important_position("text-red-500!", ImportantPosition::End),
            "text-red-500!"
        );
        // With variants, from start to end
        assert_eq!(
            fix_important_position("hover:!text-red-500", ImportantPosition::End),
            "hover:text-red-500!"
        );
        // No important modifier
        assert_eq!(
            fix_important_position("text-red-500", ImportantPosition::End),
            "text-red-500"
        );
    }
}
