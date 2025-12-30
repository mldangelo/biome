use crate::lint::nursery::use_sorted_classes::any_class_string_like::AnyClassStringLike;
use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_rowan::AstNode;
use biome_rule_options::no_arbitrary_tailwind_value::NoArbitraryTailwindValueOptions;
use regex::Regex;

declare_lint_rule! {
    /// Disallow arbitrary values in Tailwind CSS classes.
    ///
    /// Tailwind CSS allows arbitrary values using bracket notation like `w-[100px]` or `bg-[#ff0000]`.
    /// While useful for one-off values, overuse of arbitrary values defeats the purpose of using
    /// a utility-first CSS framework and can lead to inconsistent designs.
    ///
    /// This rule enforces using only predefined Tailwind utilities, encouraging developers to
    /// extend the Tailwind configuration when custom values are needed consistently.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="w-[100px]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="bg-[#ff0000]" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="text-[14px] p-[10px]" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="w-24" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="bg-red-500" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="text-sm p-2" />;
    /// ```
    ///
    /// ## Options
    ///
    /// ### `allowlist`
    ///
    /// An array of patterns to allow. Patterns can be exact class names or regex patterns.
    ///
    /// ```json,options
    /// {
    ///     "options": {
    ///         "allowlist": ["bg-\\[url\\(.*\\)\\]", "content-\\[.*\\]"]
    ///     }
    /// }
    /// ```
    ///
    /// This allows `bg-[url(...)]` for background images and `content-[...]` for pseudo-element content,
    /// which often require arbitrary values.
    ///
    pub NoArbitraryTailwindValue {
        version: "next",
        name: "noArbitraryTailwindValue",
        language: "jsx",
        recommended: false,
    }
}

/// State containing the found arbitrary classes
pub struct ArbitraryClassState {
    /// List of arbitrary classes found
    pub arbitrary_classes: Vec<String>,
}

impl Rule for NoArbitraryTailwindValue {
    type Query = Ast<AnyClassStringLike>;
    type State = ArbitraryClassState;
    type Signals = Option<Self::State>;
    type Options = NoArbitraryTailwindValueOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let options = ctx.options();
        let node = ctx.query();

        let value = node.value()?;
        let value_str = value.text();

        // Pre-compile allowlist patterns
        let allowlist_patterns: Vec<Regex> = options
            .allowlist
            .iter()
            .flat_map(|patterns| patterns.iter().filter_map(|p| Regex::new(p.as_ref()).ok()))
            .collect();

        let mut arbitrary_classes = Vec::new();

        for class in value_str.split_whitespace() {
            // Check if this class contains an arbitrary value (has [...])
            if contains_arbitrary_value(class) {
                // Check if it's in the allowlist
                let is_allowed = allowlist_patterns.iter().any(|re| re.is_match(class));

                if !is_allowed {
                    arbitrary_classes.push(class.to_string());
                }
            }
        }

        if arbitrary_classes.is_empty() {
            return None;
        }

        Some(ArbitraryClassState { arbitrary_classes })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        let classes_str = state
            .arbitrary_classes
            .iter()
            .map(|s| format!("`{}`", s))
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Arbitrary Tailwind CSS value"<Emphasis>{
                        if state.arbitrary_classes.len() > 1 { "s" } else { "" }
                    }</Emphasis>" detected: "{classes_str}
                },
            )
            .note(markup! {
                "Arbitrary values can lead to inconsistent designs. Consider extending your Tailwind configuration instead."
            }),
        )
    }
}

/// Check if a class contains an arbitrary value (bracket notation)
fn contains_arbitrary_value(class: &str) -> bool {
    // Look for [...] pattern that's not empty
    // Need to handle nested brackets for things like bg-[url('...[...]...')]
    let mut bracket_depth = 0;
    let mut found_open = false;

    for ch in class.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                found_open = true;
            }
            ']' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                    // If we found a matching close bracket at depth 0, it's an arbitrary value
                    if bracket_depth == 0 && found_open {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_arbitrary_value() {
        // Should detect arbitrary values
        assert!(contains_arbitrary_value("w-[100px]"));
        assert!(contains_arbitrary_value("bg-[#ff0000]"));
        assert!(contains_arbitrary_value("text-[14px]"));
        assert!(contains_arbitrary_value("hover:w-[100px]"));
        assert!(contains_arbitrary_value("bg-[url('image.png')]"));
        assert!(contains_arbitrary_value("content-['']"));

        // Should not detect non-arbitrary classes
        assert!(!contains_arbitrary_value("w-24"));
        assert!(!contains_arbitrary_value("bg-red-500"));
        assert!(!contains_arbitrary_value("hover:bg-blue-500"));
        assert!(!contains_arbitrary_value("text-sm"));
    }
}
