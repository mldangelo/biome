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

declare_lint_rule! {
    /// Enforce using shorthand Tailwind CSS classes when all sides have the same value.
    ///
    /// When you specify the same value for all sides of a property (padding, margin, etc.),
    /// you can use the shorthand class instead. This rule detects these cases and suggests
    /// the shorthand version.
    ///
    /// Supports:
    /// - `px-* py-*` → `p-*`
    /// - `mx-* my-*` → `m-*`
    /// - `pt-* pr-* pb-* pl-*` → `p-*`
    /// - `mt-* mr-* mb-* ml-*` → `m-*`
    /// - `w-* h-*` → `size-*` (Tailwind v3.4+)
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="px-4 py-4" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="pt-2 pr-2 pb-2 pl-2" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="mt-4 mr-4 mb-4 ml-4" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="w-4 h-4" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="p-4" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="px-4 py-2" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="m-4" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="size-4" />;
    /// ```
    ///
    pub UseTailwindShorthandClasses {
        version: "next",
        name: "useTailwindShorthandClasses",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// State containing the fixed class string and the shorthand suggestions
pub struct ShorthandState {
    /// The fixed class string with shorthand classes applied
    pub fixed: Box<str>,
    /// List of shorthand suggestions (from -> to)
    pub suggestions: Vec<ShorthandSuggestion>,
}

/// A suggestion to use a shorthand class
pub struct ShorthandSuggestion {
    /// The classes that can be replaced
    pub from_classes: Vec<String>,
    /// The shorthand class to use
    pub to_class: String,
}

impl Rule for UseTailwindShorthandClasses {
    type Query = Ast<AnyClassStringLike>;
    type State = ShorthandState;
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

        let classes: Vec<&str> = value_str.split_whitespace().collect();
        if classes.len() < 2 {
            return None;
        }

        // Group classes by variant and property type
        // Key: (variant, property_prefix), Value: (value, class_indices)
        let mut class_groups: FxHashMap<(String, &'static str), FxHashMap<String, Vec<usize>>> =
            FxHashMap::default();

        for (idx, class) in classes.iter().enumerate() {
            let (variants, utility) = extract_variants_and_utility(class);
            if let Some((prefix, value)) = extract_prefix_and_value(utility) {
                let key = (variants.to_string(), prefix);
                class_groups
                    .entry(key)
                    .or_default()
                    .entry(value.to_string())
                    .or_default()
                    .push(idx);
            }
        }

        // Find shorthand opportunities
        let mut suggestions: Vec<ShorthandSuggestion> = Vec::new();
        let mut classes_to_remove: Vec<usize> = Vec::new();
        let mut classes_to_add: Vec<(usize, String)> = Vec::new(); // (insert_after_idx, new_class)

        for (variants, prefix) in class_groups.keys() {
            // Check for px + py = p shorthand
            if (prefix == &"px" || prefix == &"py")
                && let Some(shorthand) =
                    find_xy_shorthand(&class_groups, variants, &classes, prefix)
            {
                for idx in &shorthand.indices {
                    classes_to_remove.push(*idx);
                }
                classes_to_add.push((shorthand.indices[0], shorthand.shorthand.clone()));
                suggestions.push(ShorthandSuggestion {
                    from_classes: shorthand
                        .indices
                        .iter()
                        .map(|i| classes[*i].to_string())
                        .collect(),
                    to_class: format!(
                        "{}{}",
                        if variants.is_empty() {
                            String::new()
                        } else {
                            variants.clone()
                        },
                        shorthand.shorthand
                    ),
                });
            }

            // Check for mx + my = m shorthand
            if (prefix == &"mx" || prefix == &"my")
                && let Some(shorthand) =
                    find_xy_shorthand(&class_groups, variants, &classes, prefix)
            {
                for idx in &shorthand.indices {
                    if !classes_to_remove.contains(idx) {
                        classes_to_remove.push(*idx);
                    }
                }
                if !classes_to_add
                    .iter()
                    .any(|(_, c)| c == &shorthand.shorthand)
                {
                    classes_to_add.push((shorthand.indices[0], shorthand.shorthand.clone()));
                    suggestions.push(ShorthandSuggestion {
                        from_classes: shorthand
                            .indices
                            .iter()
                            .map(|i| classes[*i].to_string())
                            .collect(),
                        to_class: format!(
                            "{}{}",
                            if variants.is_empty() {
                                String::new()
                            } else {
                                variants.clone()
                            },
                            shorthand.shorthand
                        ),
                    });
                }
            }

            // Check for pt + pr + pb + pl = p shorthand
            if (prefix == &"pt" || prefix == &"pr" || prefix == &"pb" || prefix == &"pl")
                && let Some(shorthand) =
                    find_four_side_shorthand(&class_groups, variants, &classes, "p")
            {
                for idx in &shorthand.indices {
                    if !classes_to_remove.contains(idx) {
                        classes_to_remove.push(*idx);
                    }
                }
                if !classes_to_add
                    .iter()
                    .any(|(_, c)| c == &shorthand.shorthand)
                {
                    classes_to_add.push((shorthand.indices[0], shorthand.shorthand.clone()));
                    suggestions.push(ShorthandSuggestion {
                        from_classes: shorthand
                            .indices
                            .iter()
                            .map(|i| classes[*i].to_string())
                            .collect(),
                        to_class: format!(
                            "{}{}",
                            if variants.is_empty() {
                                String::new()
                            } else {
                                variants.clone()
                            },
                            shorthand.shorthand
                        ),
                    });
                }
            }

            // Check for mt + mr + mb + ml = m shorthand
            if (prefix == &"mt" || prefix == &"mr" || prefix == &"mb" || prefix == &"ml")
                && let Some(shorthand) =
                    find_four_side_shorthand(&class_groups, variants, &classes, "m")
            {
                for idx in &shorthand.indices {
                    if !classes_to_remove.contains(idx) {
                        classes_to_remove.push(*idx);
                    }
                }
                if !classes_to_add
                    .iter()
                    .any(|(_, c)| c == &shorthand.shorthand)
                {
                    classes_to_add.push((shorthand.indices[0], shorthand.shorthand.clone()));
                    suggestions.push(ShorthandSuggestion {
                        from_classes: shorthand
                            .indices
                            .iter()
                            .map(|i| classes[*i].to_string())
                            .collect(),
                        to_class: format!(
                            "{}{}",
                            if variants.is_empty() {
                                String::new()
                            } else {
                                variants.clone()
                            },
                            shorthand.shorthand
                        ),
                    });
                }
            }

            // Check for w + h = size shorthand (Tailwind v3.4+)
            if (prefix == &"w" || prefix == &"h")
                && let Some(shorthand) =
                    find_size_shorthand(&class_groups, variants, &classes, prefix)
            {
                for idx in &shorthand.indices {
                    if !classes_to_remove.contains(idx) {
                        classes_to_remove.push(*idx);
                    }
                }
                let full_shorthand = format!(
                    "{}{}",
                    if variants.is_empty() {
                        String::new()
                    } else {
                        variants.clone()
                    },
                    shorthand.shorthand
                );
                if !classes_to_add.iter().any(|(_, c)| c == &full_shorthand) {
                    classes_to_add.push((shorthand.indices[0], full_shorthand.clone()));
                    suggestions.push(ShorthandSuggestion {
                        from_classes: shorthand
                            .indices
                            .iter()
                            .map(|i| classes[*i].to_string())
                            .collect(),
                        to_class: full_shorthand,
                    });
                }
            }
        }

        if suggestions.is_empty() {
            return None;
        }

        // Build fixed string
        classes_to_remove.sort_unstable();
        classes_to_remove.dedup();
        classes_to_add.sort_by_key(|(idx, _)| *idx);

        let mut result_classes: Vec<String> = Vec::new();
        let mut added_indices: Vec<usize> = Vec::new();

        for (idx, class) in classes.iter().enumerate() {
            // Add shorthand at the first removed index
            for (add_at, new_class) in &classes_to_add {
                if *add_at == idx && !added_indices.contains(add_at) {
                    result_classes.push(new_class.clone());
                    added_indices.push(*add_at);
                }
            }

            if !classes_to_remove.contains(&idx) {
                result_classes.push((*class).to_string());
            }
        }

        let fixed = result_classes.join(" ");

        Some(ShorthandState {
            fixed: fixed.into(),
            suggestions,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let suggestions_str = state
            .suggestions
            .iter()
            .map(|s| {
                let from = s.from_classes.join(" + ");
                format!("`{}` → `{}`", from, s.to_class)
            })
            .collect::<Vec<_>>()
            .join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Use shorthand Tailwind CSS classes."
                },
            )
            .note(markup! {
                "Replace: "{suggestions_str}
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
                "Replace with shorthand class."
            }
            .to_owned(),
            mutation,
        ))
    }
}

/// Extract variants and utility
fn extract_variants_and_utility(class: &str) -> (&str, &str) {
    if let Some(last_colon_idx) = class.rfind(':') {
        (&class[..=last_colon_idx], &class[last_colon_idx + 1..])
    } else {
        ("", class)
    }
}

/// Extract prefix (e.g., "px", "py", "pt") and value (e.g., "4", "2")
fn extract_prefix_and_value(utility: &str) -> Option<(&'static str, &str)> {
    // Remove important modifier
    let utility = utility.trim_start_matches('!').trim_end_matches('!');

    // Padding
    if let Some(rest) = utility.strip_prefix("px-") {
        return Some(("px", rest));
    }
    if let Some(rest) = utility.strip_prefix("py-") {
        return Some(("py", rest));
    }
    if let Some(rest) = utility.strip_prefix("pt-") {
        return Some(("pt", rest));
    }
    if let Some(rest) = utility.strip_prefix("pr-") {
        return Some(("pr", rest));
    }
    if let Some(rest) = utility.strip_prefix("pb-") {
        return Some(("pb", rest));
    }
    if let Some(rest) = utility.strip_prefix("pl-") {
        return Some(("pl", rest));
    }

    // Margin
    if let Some(rest) = utility.strip_prefix("mx-") {
        return Some(("mx", rest));
    }
    if let Some(rest) = utility.strip_prefix("my-") {
        return Some(("my", rest));
    }
    if let Some(rest) = utility.strip_prefix("mt-") {
        return Some(("mt", rest));
    }
    if let Some(rest) = utility.strip_prefix("mr-") {
        return Some(("mr", rest));
    }
    if let Some(rest) = utility.strip_prefix("mb-") {
        return Some(("mb", rest));
    }
    if let Some(rest) = utility.strip_prefix("ml-") {
        return Some(("ml", rest));
    }

    // Width and Height (for size-* shorthand)
    if let Some(rest) = utility.strip_prefix("w-") {
        return Some(("w", rest));
    }
    if let Some(rest) = utility.strip_prefix("h-") {
        return Some(("h", rest));
    }

    None
}

struct ShorthandResult {
    indices: Vec<usize>,
    shorthand: String,
}

/// Find xy shorthand (px + py = p, mx + my = m)
fn find_xy_shorthand(
    groups: &FxHashMap<(String, &'static str), FxHashMap<String, Vec<usize>>>,
    variants: &str,
    _classes: &[&str],
    current_prefix: &str,
) -> Option<ShorthandResult> {
    let (x_prefix, y_prefix, shorthand_prefix) = match current_prefix {
        "px" | "py" => ("px", "py", "p"),
        "mx" | "my" => ("mx", "my", "m"),
        _ => return None,
    };

    let x_key = (variants.to_string(), x_prefix);
    let y_key = (variants.to_string(), y_prefix);

    let x_values = groups.get(&x_key)?;
    let y_values = groups.get(&y_key)?;

    // Find matching values
    for (value, x_indices) in x_values {
        if let Some(y_indices) = y_values.get(value) {
            // Found matching x and y with same value
            let mut indices = x_indices.clone();
            indices.extend(y_indices.clone());
            return Some(ShorthandResult {
                indices,
                shorthand: format!("{}-{}", shorthand_prefix, value),
            });
        }
    }

    None
}

/// Find four-side shorthand (pt + pr + pb + pl = p, mt + mr + mb + ml = m)
fn find_four_side_shorthand(
    groups: &FxHashMap<(String, &'static str), FxHashMap<String, Vec<usize>>>,
    variants: &str,
    _classes: &[&str],
    shorthand_prefix: &str,
) -> Option<ShorthandResult> {
    let (t_prefix, r_prefix, b_prefix, l_prefix) = match shorthand_prefix {
        "p" => ("pt", "pr", "pb", "pl"),
        "m" => ("mt", "mr", "mb", "ml"),
        _ => return None,
    };

    let t_key = (variants.to_string(), t_prefix);
    let r_key = (variants.to_string(), r_prefix);
    let b_key = (variants.to_string(), b_prefix);
    let l_key = (variants.to_string(), l_prefix);

    let t_values = groups.get(&t_key)?;
    let r_values = groups.get(&r_key)?;
    let b_values = groups.get(&b_key)?;
    let l_values = groups.get(&l_key)?;

    // Find matching values across all four sides
    for (value, t_indices) in t_values {
        if let Some(r_indices) = r_values.get(value)
            && let Some(b_indices) = b_values.get(value)
            && let Some(l_indices) = l_values.get(value)
        {
            // Found all four sides with same value
            let mut indices = t_indices.clone();
            indices.extend(r_indices.clone());
            indices.extend(b_indices.clone());
            indices.extend(l_indices.clone());
            return Some(ShorthandResult {
                indices,
                shorthand: format!("{}-{}", shorthand_prefix, value),
            });
        }
    }

    None
}

/// Find size shorthand (w + h = size)
fn find_size_shorthand(
    groups: &FxHashMap<(String, &'static str), FxHashMap<String, Vec<usize>>>,
    variants: &str,
    _classes: &[&str],
    _current_prefix: &str,
) -> Option<ShorthandResult> {
    let w_key = (variants.to_string(), "w");
    let h_key = (variants.to_string(), "h");

    let w_values = groups.get(&w_key)?;
    let h_values = groups.get(&h_key)?;

    // Find matching values between width and height
    for (value, w_indices) in w_values {
        if let Some(h_indices) = h_values.get(value) {
            // Found matching w and h with same value
            let mut indices = w_indices.clone();
            indices.extend(h_indices.clone());
            return Some(ShorthandResult {
                indices,
                shorthand: format!("size-{}", value),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prefix_and_value() {
        assert_eq!(extract_prefix_and_value("px-4"), Some(("px", "4")));
        assert_eq!(extract_prefix_and_value("py-2"), Some(("py", "2")));
        assert_eq!(extract_prefix_and_value("pt-0"), Some(("pt", "0")));
        assert_eq!(extract_prefix_and_value("mx-auto"), Some(("mx", "auto")));
        assert_eq!(extract_prefix_and_value("w-4"), Some(("w", "4")));
        assert_eq!(extract_prefix_and_value("h-full"), Some(("h", "full")));
        assert_eq!(extract_prefix_and_value("flex"), None);
    }
}
