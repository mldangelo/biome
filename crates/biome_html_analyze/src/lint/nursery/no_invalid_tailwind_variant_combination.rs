use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_html_syntax::{HtmlAttribute, inner_string_text};
use biome_rowan::AstNode;
use rustc_hash::FxHashSet;

declare_lint_rule! {
    /// Disallow invalid or redundant Tailwind CSS variant combinations.
    ///
    /// Tailwind CSS variants (like `hover:`, `sm:`, `first:`) can be combined to create
    /// conditional styles. However, some combinations are invalid or redundant:
    ///
    /// - **Duplicate variants**: Using the same variant twice (e.g., `hover:hover:`)
    /// - **Conflicting responsive variants**: Using multiple breakpoints in one class (e.g., `sm:md:`)
    /// - **Mutually exclusive variants**: Combining variants that can't both be true (e.g., `first:last:`)
    ///
    /// These combinations either produce no CSS output or indicate a logical error in the code.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```html,expect_diagnostic
    /// <div class="hover:hover:bg-red-500"></div>
    /// ```
    ///
    /// ```html,expect_diagnostic
    /// <div class="sm:md:flex"></div>
    /// ```
    ///
    /// ```html,expect_diagnostic
    /// <div class="first:last:text-bold"></div>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```html
    /// <div class="hover:bg-red-500"></div>
    /// ```
    ///
    /// ```html
    /// <div class="sm:flex md:block"></div>
    /// ```
    ///
    /// ```html
    /// <div class="hover:focus:bg-red-500"></div>
    /// ```
    ///
    pub NoInvalidTailwindVariantCombination {
        version: "next",
        name: "noInvalidTailwindVariantCombination",
        language: "html",
        recommended: false,
    }
}

/// Responsive breakpoint variants that are mutually exclusive within a single class
const RESPONSIVE_VARIANTS: &[&str] = &["sm", "md", "lg", "xl", "2xl"];

/// Max-width responsive variants that are mutually exclusive within a single class
const MAX_RESPONSIVE_VARIANTS: &[&str] = &["max-sm", "max-md", "max-lg", "max-xl", "max-2xl"];

/// Groups of positional variants where only one can be true at a time
const POSITIONAL_EXCLUSIVE_GROUPS: &[&[&str]] = &[
    &["first", "last", "only", "odd", "even"],
    &["first-of-type", "last-of-type", "only-of-type"],
];

/// Describes the type of invalid variant combination found
#[derive(Debug, Clone)]
pub enum InvalidVariantKind {
    /// Same variant used multiple times (e.g., `hover:hover:`)
    Duplicate(Box<str>),
    /// Multiple responsive breakpoints (e.g., `sm:md:`)
    ConflictingResponsive(Box<str>, Box<str>),
    /// Multiple max-width breakpoints (e.g., `max-sm:max-md:`)
    ConflictingMaxResponsive(Box<str>, Box<str>),
    /// Variants that can't both be true (e.g., `first:last:`)
    MutuallyExclusive(Box<str>, Box<str>),
}

/// State containing invalid variant combinations found
pub struct InvalidVariantState {
    /// List of (class_name, invalid_kind) pairs
    pub invalid_classes: Vec<(Box<str>, InvalidVariantKind)>,
}

impl Rule for NoInvalidTailwindVariantCombination {
    type Query = Ast<HtmlAttribute>;
    type State = InvalidVariantState;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let attribute = ctx.query();

        // Only check "class" attribute
        let name = attribute.name().ok()?;
        let name_token = name.value_token().ok()?;
        if name_token.text_trimmed() != "class" {
            return None;
        }

        // Get the attribute value
        let initializer = attribute.initializer()?;
        let value = initializer.value().ok()?;
        let html_string = value.as_html_string()?;
        let value_token = html_string.value_token().ok()?;

        // Get the inner string (without quotes)
        let inner_text = inner_string_text(&value_token);

        let mut invalid_classes = Vec::new();

        for class in inner_text.text().split_whitespace() {
            if let Some(kind) = check_variant_combination(class) {
                invalid_classes.push((class.into(), kind));
            }
        }

        if invalid_classes.is_empty() {
            return None;
        }

        Some(InvalidVariantState { invalid_classes })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let diagnostic = if state.invalid_classes.len() == 1 {
            let (class, kind) = &state.invalid_classes[0];
            let reason = format_invalid_reason(kind);

            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "This "<Emphasis>"class"</Emphasis>" attribute contains an invalid variant combination."
                },
            )
            .note(markup! {
                "The class "<Emphasis>{class.as_ref()}</Emphasis>" has "{reason}"."
            })
        } else {
            // Multiple invalid classes - format as a list
            let issues: Vec<String> = state
                .invalid_classes
                .iter()
                .map(|(class, kind)| {
                    let reason = format_invalid_reason(kind);
                    format!("`{}`: {}", class, reason)
                })
                .collect();

            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "This "<Emphasis>"class"</Emphasis>" attribute contains invalid variant combinations."
                },
            )
            .note(markup! {
                {issues.join("; ")}
            })
        };

        Some(diagnostic.note(markup! {
            "Invalid variant combinations produce unexpected CSS or no CSS at all."
        }))
    }
}

/// Format a human-readable reason for why the variant combination is invalid
fn format_invalid_reason(kind: &InvalidVariantKind) -> String {
    match kind {
        InvalidVariantKind::Duplicate(variant) => {
            format!("a duplicate `{}` variant", variant)
        }
        InvalidVariantKind::ConflictingResponsive(a, b) => {
            format!("conflicting responsive breakpoints `{}` and `{}`", a, b)
        }
        InvalidVariantKind::ConflictingMaxResponsive(a, b) => {
            format!("conflicting max-width breakpoints `{}` and `{}`", a, b)
        }
        InvalidVariantKind::MutuallyExclusive(a, b) => {
            format!("mutually exclusive variants `{}` and `{}`", a, b)
        }
    }
}

/// Check a class for invalid variant combinations
/// Returns Some(InvalidVariantKind) if invalid, None if valid
fn check_variant_combination(class: &str) -> Option<InvalidVariantKind> {
    // Extract variants (everything before the last colon-separated segment)
    let parts: Vec<&str> = class.split(':').collect();

    if parts.len() < 2 {
        // No variants, nothing to check
        return None;
    }

    // All but the last part are variants
    let variants: Vec<&str> = parts[..parts.len() - 1].to_vec();

    // Check for duplicate variants
    let mut seen: FxHashSet<&str> = FxHashSet::default();
    for variant in &variants {
        // Normalize variant (strip arbitrary parts for comparison)
        let normalized = normalize_variant(variant);
        if !seen.insert(normalized) {
            return Some(InvalidVariantKind::Duplicate((*variant).into()));
        }
    }

    // Check for conflicting responsive variants (sm, md, lg, xl, 2xl)
    let responsive: Vec<&str> = variants
        .iter()
        .filter(|v| RESPONSIVE_VARIANTS.contains(*v))
        .copied()
        .collect();
    if responsive.len() > 1 {
        return Some(InvalidVariantKind::ConflictingResponsive(
            responsive[0].into(),
            responsive[1].into(),
        ));
    }

    // Check for conflicting max-width responsive variants
    let max_responsive: Vec<&str> = variants
        .iter()
        .filter(|v| MAX_RESPONSIVE_VARIANTS.contains(*v))
        .copied()
        .collect();
    if max_responsive.len() > 1 {
        return Some(InvalidVariantKind::ConflictingMaxResponsive(
            max_responsive[0].into(),
            max_responsive[1].into(),
        ));
    }

    // Check for mutually exclusive positional variants
    for group in POSITIONAL_EXCLUSIVE_GROUPS {
        let found: Vec<&str> = variants
            .iter()
            .filter(|v| group.contains(*v))
            .copied()
            .collect();
        if found.len() > 1 {
            return Some(InvalidVariantKind::MutuallyExclusive(
                found[0].into(),
                found[1].into(),
            ));
        }
    }

    None
}

/// Normalize a variant for comparison
/// - Strips named group suffixes (e.g., `group-hover/sidebar` -> `group-hover`)
fn normalize_variant(variant: &str) -> &str {
    // For things like `group-hover/name`, strip the name part
    if let Some(pos) = variant.find('/') {
        &variant[..pos]
    } else {
        variant
    }
}
