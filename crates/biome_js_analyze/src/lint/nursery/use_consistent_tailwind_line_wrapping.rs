use crate::JsRuleAction;
use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_factory::make::{
    js_literal_member_name, js_string_literal, js_string_literal_expression,
    js_string_literal_single_quotes, js_template_chunk, js_template_chunk_element, jsx_string,
};
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::use_consistent_tailwind_line_wrapping::UseConsistentTailwindLineWrappingOptions;

declare_lint_rule! {
    /// Enforce consistent line wrapping in Tailwind CSS class strings.
    ///
    /// This rule ensures that Tailwind CSS class strings are formatted consistently
    /// by normalizing whitespace. By default, multi-line class strings are collapsed
    /// to single lines with single spaces between classes.
    ///
    /// When the `printWidth` option is set, classes are wrapped to stay within the
    /// specified line width, providing a balance between readability and line length.
    ///
    /// Consistent formatting improves readability and makes diffs cleaner when
    /// class lists change.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="flex
    ///   items-center
    ///   justify-between" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="flex items-center justify-between" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="bg-blue-500 text-white p-4 rounded-lg shadow-md" />;
    /// ```
    ///
    /// ## Options
    ///
    /// ### `printWidth`
    ///
    /// The maximum line width before wrapping classes. If not specified, classes
    /// are always collapsed to a single line.
    ///
    /// ```json
    /// {
    ///   "linter": {
    ///     "rules": {
    ///       "nursery": {
    ///         "useConsistentTailwindLineWrapping": {
    ///           "level": "warn",
    ///           "options": {
    ///             "printWidth": 80
    ///           }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    pub UseConsistentTailwindLineWrapping {
        version: "next",
        name: "useConsistentTailwindLineWrapping",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// State containing the normalized class string
pub struct LineWrappingState {
    /// The normalized/wrapped class string
    pub normalized: Box<str>,
}

impl Rule for UseConsistentTailwindLineWrapping {
    type Query = Ast<AnyClassStringLike>;
    type State = LineWrappingState;
    type Signals = Option<Self::State>;
    type Options = UseConsistentTailwindLineWrappingOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        if !node.should_visit(options)? {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();

        // Get the print width option
        let print_width = options.print_width();

        // Check if the string contains newlines or multiple consecutive spaces
        let has_inconsistent_whitespace =
            value_str.contains('\n') || value_str.contains('\r') || value_str.contains("  "); // two or more spaces

        // If no print_width set, we just normalize to single line
        if print_width.is_none() {
            if !has_inconsistent_whitespace {
                return None;
            }

            // Normalize: split by whitespace and rejoin with single spaces
            let normalized: String = value_str.split_whitespace().collect::<Vec<_>>().join(" ");

            // If after normalization it's the same, no issue
            if normalized == value_str {
                return None;
            }

            return Some(LineWrappingState {
                normalized: normalized.into(),
            });
        }

        // With print_width, we need to wrap lines
        let print_width = print_width.unwrap() as usize;
        let classes: Vec<&str> = value_str.split_whitespace().collect();

        if classes.is_empty() {
            return None;
        }

        // Build the wrapped output
        let wrapped = wrap_classes(&classes, print_width);

        // If the wrapped output is the same as the original (after normalization), no issue
        let normalized_original: String = classes.join(" ");
        if wrapped == value_str || (wrapped == normalized_original && !has_inconsistent_whitespace)
        {
            return None;
        }

        Some(LineWrappingState {
            normalized: wrapped.into(),
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Tailwind class string contains inconsistent whitespace."
                },
            )
            .note(markup! {
                "Class strings should be on a single line with single spaces between classes."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<JsRuleAction> {
        let mut mutation = ctx.root().begin();
        let normalized = &state.normalized;

        match ctx.query() {
            AnyClassStringLike::JsStringLiteralExpression(string_literal) => {
                let is_double_quote = string_literal
                    .value_token()
                    .map(|token| token.text_trimmed().starts_with('"'))
                    .unwrap_or(ctx.preferred_quote().is_double());
                let replacement = js_string_literal_expression(if is_double_quote {
                    js_string_literal(normalized)
                } else {
                    js_string_literal_single_quotes(normalized)
                });
                mutation.replace_node(string_literal.clone(), replacement);
            }
            AnyClassStringLike::JsLiteralMemberName(string_literal) => {
                let replacement = js_literal_member_name(if ctx.preferred_quote().is_double() {
                    js_string_literal(normalized)
                } else {
                    js_string_literal_single_quotes(normalized)
                });
                mutation.replace_node(string_literal.clone(), replacement);
            }
            AnyClassStringLike::JsxString(jsx_string_node) => {
                let is_double_quote = jsx_string_node
                    .value_token()
                    .map(|token| token.text_trimmed().starts_with('"'))
                    .unwrap_or(ctx.preferred_jsx_quote().is_double());
                let replacement = jsx_string(if is_double_quote {
                    js_string_literal(normalized)
                } else {
                    js_string_literal_single_quotes(normalized)
                });
                mutation.replace_node(jsx_string_node.clone(), replacement);
            }
            AnyClassStringLike::JsTemplateChunkElement(chunk) => {
                let replacement = js_template_chunk_element(js_template_chunk(normalized));
                mutation.replace_node(chunk.clone(), replacement);
            }
        };

        Some(JsRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! {
                "Normalize whitespace in class string."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Wrap classes to stay within the specified print width.
/// Uses a greedy algorithm to fit as many classes as possible on each line.
fn wrap_classes(classes: &[&str], print_width: usize) -> String {
    if classes.is_empty() {
        return String::new();
    }

    // If all classes fit on one line, return single line
    let single_line = classes.join(" ");
    if single_line.len() <= print_width {
        return single_line;
    }

    // Otherwise, wrap lines
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for class in classes {
        if current_line.is_empty() {
            // First class on a new line always gets added
            current_line.push_str(class);
        } else {
            // Check if adding this class would exceed the width
            let potential_len = current_line.len() + 1 + class.len(); // +1 for space
            if potential_len <= print_width {
                current_line.push(' ');
                current_line.push_str(class);
            } else {
                // Start a new line
                lines.push(current_line);
                current_line = (*class).to_string();
            }
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        // Test that multiple spaces are normalized
        let input = "flex  items-center   justify-between";
        let normalized: String = input.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(normalized, "flex items-center justify-between");

        // Test that newlines are normalized
        let input = "flex\n  items-center\n  justify-between";
        let normalized: String = input.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(normalized, "flex items-center justify-between");

        // Test that already normalized string stays the same
        let input = "flex items-center justify-between";
        let normalized: String = input.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(normalized, input);
    }

    #[test]
    fn test_wrap_classes() {
        // Test single line when within width
        let classes = vec!["flex", "items-center"];
        assert_eq!(wrap_classes(&classes, 80), "flex items-center");

        // Test wrapping when exceeding width
        let classes = vec!["flex", "items-center", "justify-between", "p-4", "m-2"];
        assert_eq!(
            wrap_classes(&classes, 25),
            "flex items-center\njustify-between p-4 m-2"
        );

        // Test with very small width
        let classes = vec!["flex", "items-center"];
        assert_eq!(wrap_classes(&classes, 10), "flex\nitems-center");

        // Test empty input
        let classes: Vec<&str> = vec![];
        assert_eq!(wrap_classes(&classes, 80), "");
    }
}
