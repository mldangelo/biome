use crate::JsRuleAction;
use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_factory::make::{
    js_literal_member_name, js_string_literal, js_string_literal_expression,
    js_string_literal_single_quotes, js_template_chunk, js_template_chunk_element, jsx_string,
};
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::no_restricted_tailwind_classes::NoRestrictedTailwindClassesOptions;
use regex::Regex;

declare_lint_rule! {
    /// Disallow specific Tailwind CSS classes based on configurable patterns.
    ///
    /// This rule allows you to forbid specific Tailwind CSS utility classes
    /// in your codebase. You can configure exact class names or regular expression
    /// patterns to match against.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// With configuration: `{ "classes": [{ "pattern": "hidden", "message": "Use 'invisible' instead" }] }`
    ///
    /// ```jsx,ignore
    /// <div class="hidden" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="flex p-4" />;
    /// ```
    ///
    /// ## Options
    ///
    /// Use the `classes` option to specify which classes are forbidden.
    /// Each entry can have:
    /// - `pattern`: The class name or regex pattern to match
    /// - `regex`: Set to `true` to treat `pattern` as a regular expression (default: `false`)
    /// - `message`: Optional custom message to display
    /// - `replacement`: Optional replacement class (not supported with regex patterns)
    ///
    /// ### Regex patterns
    ///
    /// When `regex` is `true`, the pattern is treated as a regular expression.
    /// The regex is matched against the utility part of the class (without variants).
    ///
    /// ```json
    /// {
    ///   "options": {
    ///     "classes": [
    ///       { "pattern": "^bg-red-", "regex": true, "message": "Use semantic color tokens instead of red" }
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    pub NoRestrictedTailwindClasses {
        version: "next",
        name: "noRestrictedTailwindClasses",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Unsafe,
    }
}

/// State containing the found restricted classes
pub struct RestrictedClassState {
    /// List of restricted classes found with their messages
    pub restricted_classes: Vec<(String, Option<Box<str>>)>,
    /// The fixed class string with restricted classes removed/replaced
    pub fixed: Box<str>,
}

impl Rule for NoRestrictedTailwindClasses {
    type Query = Ast<AnyClassStringLike>;
    type State = RestrictedClassState;
    type Signals = Option<Self::State>;
    type Options = NoRestrictedTailwindClassesOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        // If no patterns are configured, don't run the rule
        let classes_config = options.classes.as_ref()?;
        if classes_config.is_empty() {
            return None;
        }

        let value = node.value()?;
        let value_str = value.text();

        // Pre-compile regex patterns
        let compiled_patterns: Vec<_> = classes_config
            .iter()
            .map(|p| {
                if p.regex {
                    Regex::new(&p.pattern).ok()
                } else {
                    None
                }
            })
            .collect();

        let mut restricted_classes = Vec::new();
        let mut fixed_parts: Vec<String> = Vec::new();

        for class in value_str.split_whitespace() {
            // Extract the utility part (without variants)
            let utility = extract_utility(class);
            let variants = extract_variants(class);

            let mut is_restricted = false;
            for (i, pattern) in classes_config.iter().enumerate() {
                let matches = if pattern.regex {
                    // Use compiled regex for matching
                    compiled_patterns[i]
                        .as_ref()
                        .is_some_and(|re| re.is_match(utility) || re.is_match(class))
                } else {
                    // Exact match
                    utility == pattern.pattern.as_ref() || class == pattern.pattern.as_ref()
                };

                if matches {
                    restricted_classes.push((class.to_string(), pattern.message.clone()));
                    // If there's a replacement and it's not a regex pattern, add it
                    // (regex patterns don't support replacements)
                    if !pattern.regex
                        && let Some(replacement) = &pattern.replacement
                    {
                        if variants.is_empty() {
                            fixed_parts.push(replacement.to_string());
                        } else {
                            fixed_parts.push(format!("{}{}", variants, replacement));
                        }
                    }
                    // If no replacement or regex pattern, the class is simply removed
                    is_restricted = true;
                    break;
                }
            }

            if !is_restricted {
                fixed_parts.push(class.to_string());
            }
        }

        if restricted_classes.is_empty() {
            return None;
        }

        let fixed = fixed_parts.join(" ");

        Some(RestrictedClassState {
            restricted_classes,
            fixed: fixed.into(),
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let (classes_str, custom_message) = if state.restricted_classes.len() == 1 {
            let (class, message) = &state.restricted_classes[0];
            (format!("`{}`", class), message.clone())
        } else {
            let classes = state
                .restricted_classes
                .iter()
                .map(|(c, _)| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", ");
            (classes, None)
        };

        let mut diagnostic = RuleDiagnostic::new(
            rule_category!(),
            node.range(),
            markup! {
                "Restricted Tailwind CSS class(es) found: "{classes_str}"."
            },
        );

        if let Some(message) = custom_message {
            diagnostic = diagnostic.note(markup! {
                {message}
            });
        } else {
            diagnostic = diagnostic.note(markup! {
                "This class has been restricted in the project configuration."
            });
        }

        Some(diagnostic)
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
                "Remove or replace restricted class(es)."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Extract the utility class (the part after all variants)
fn extract_utility(class: &str) -> &str {
    if let Some(last_colon_idx) = class.rfind(':') {
        &class[last_colon_idx + 1..]
    } else {
        class
    }
}

/// Extract the variants (the part before the utility, including the trailing colon)
fn extract_variants(class: &str) -> &str {
    if let Some(last_colon_idx) = class.rfind(':') {
        &class[..=last_colon_idx]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_utility() {
        assert_eq!(extract_utility("hidden"), "hidden");
        assert_eq!(extract_utility("hover:hidden"), "hidden");
        assert_eq!(extract_utility("focus:hover:hidden"), "hidden");
        assert_eq!(extract_utility("md:hover:bg-blue-500"), "bg-blue-500");
    }

    #[test]
    fn test_regex_matching() {
        // Test basic regex pattern
        let re = Regex::new("^bg-red-").unwrap();
        assert!(re.is_match("bg-red-500"));
        assert!(re.is_match("bg-red-100"));
        assert!(!re.is_match("bg-blue-500"));
        assert!(!re.is_match("text-red-500"));

        // Test pattern for all background colors
        let re = Regex::new("^bg-").unwrap();
        assert!(re.is_match("bg-red-500"));
        assert!(re.is_match("bg-blue-500"));
        assert!(!re.is_match("text-red-500"));

        // Test pattern for deprecated classes
        let re = Regex::new("-(xs|sm)$").unwrap();
        assert!(re.is_match("text-xs"));
        assert!(re.is_match("text-sm"));
        assert!(!re.is_match("text-lg"));
    }
}
