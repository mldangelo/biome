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
    /// Disallow conflicting Tailwind CSS classes in the same element.
    ///
    /// Tailwind CSS classes that target the same CSS property will conflict with each other.
    /// Only the last class in the source order will take effect, making earlier classes redundant.
    /// This rule detects such conflicts and keeps only the last occurrence.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="p-2 p-4" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="text-red-500 text-blue-500" />;
    /// ```
    ///
    /// ```jsx,expect_diagnostic
    /// <div class="hidden block" />;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```jsx
    /// <div class="p-4 m-2" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="text-red-500 bg-blue-500" />;
    /// ```
    ///
    /// ```jsx
    /// <div class="hover:p-2 p-4" />;
    /// ```
    ///
    pub NoConflictingTailwindClasses {
        version: "next",
        name: "noConflictingTailwindClasses",
        language: "jsx",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

/// State containing the fixed class string and the conflicting groups
pub struct ConflictingClassState {
    /// The fixed class string with conflicts resolved (last one wins)
    pub fixed: Box<str>,
    /// List of conflicting class groups with their classes
    pub conflicts: Vec<ConflictGroup>,
}

/// A group of conflicting classes
pub struct ConflictGroup {
    /// The property group that conflicts (e.g., "padding", "display")
    pub property: &'static str,
    /// The classes that conflict (first N-1 are removed, last is kept)
    pub classes: Vec<String>,
}

impl Rule for NoConflictingTailwindClasses {
    type Query = Ast<AnyClassStringLike>;
    type State = ConflictingClassState;
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

        // Track property groups for each variant combination
        // Key: (variant, property_group), Value: (class_index, class_string)
        let mut property_map: FxHashMap<(String, &'static str), Vec<(usize, String)>> =
            FxHashMap::default();

        for (idx, class) in classes.iter().enumerate() {
            let (variants, utility) = extract_variants_and_utility(class);
            if let Some(property_group) = get_property_group(utility) {
                let key = (variants.to_string(), property_group);
                property_map
                    .entry(key)
                    .or_default()
                    .push((idx, (*class).to_string()));
            }
        }

        // Find conflicts (groups with more than one class)
        let mut conflicts: Vec<ConflictGroup> = Vec::new();
        let mut classes_to_remove: Vec<usize> = Vec::new();

        for ((_, property), group_classes) in &property_map {
            if group_classes.len() > 1 {
                // Keep the last one, remove the rest
                for (idx, _) in group_classes.iter().take(group_classes.len() - 1) {
                    classes_to_remove.push(*idx);
                }
                conflicts.push(ConflictGroup {
                    property,
                    classes: group_classes.iter().map(|(_, c)| c.clone()).collect(),
                });
            }
        }

        if conflicts.is_empty() {
            return None;
        }

        // Build fixed string (keeping only non-conflicting classes and last of each conflict)
        classes_to_remove.sort_unstable();
        let fixed_classes: Vec<&str> = classes
            .iter()
            .enumerate()
            .filter(|(idx, _)| !classes_to_remove.contains(idx))
            .map(|(_, c)| *c)
            .collect();
        let fixed = fixed_classes.join(" ");

        Some(ConflictingClassState {
            fixed: fixed.into(),
            conflicts,
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        let conflicts_str = state
            .conflicts
            .iter()
            .map(|c| {
                let classes = c
                    .classes
                    .iter()
                    .map(|s| format!("`{}`", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}: {}", c.property, classes)
            })
            .collect::<Vec<_>>()
            .join("; ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.range(),
                markup! {
                    "Conflicting Tailwind CSS classes detected."
                },
            )
            .note(markup! {
                "Conflicts: "{conflicts_str}
            })
            .note(markup! {
                "Only the last class in each conflict group will take effect."
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
                "Remove conflicting classes (keep last occurrence)."
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

/// Get the property group for a utility class
/// Returns None if the utility doesn't belong to a known conflict group
fn get_property_group(utility: &str) -> Option<&'static str> {
    // Remove any important modifier
    let utility = utility.trim_start_matches('!').trim_end_matches('!');

    // Padding
    if utility.starts_with("p-")
        || utility.starts_with("px-")
        || utility.starts_with("py-")
        || utility.starts_with("pt-")
        || utility.starts_with("pr-")
        || utility.starts_with("pb-")
        || utility.starts_with("pl-")
        || utility.starts_with("ps-")
        || utility.starts_with("pe-")
    {
        // More specific padding properties
        if utility.starts_with("px-") {
            return Some("padding-x");
        }
        if utility.starts_with("py-") {
            return Some("padding-y");
        }
        if utility.starts_with("pt-") {
            return Some("padding-top");
        }
        if utility.starts_with("pr-") {
            return Some("padding-right");
        }
        if utility.starts_with("pb-") {
            return Some("padding-bottom");
        }
        if utility.starts_with("pl-") {
            return Some("padding-left");
        }
        if utility.starts_with("ps-") {
            return Some("padding-inline-start");
        }
        if utility.starts_with("pe-") {
            return Some("padding-inline-end");
        }
        return Some("padding");
    }

    // Margin
    if utility.starts_with("m-")
        || utility.starts_with("mx-")
        || utility.starts_with("my-")
        || utility.starts_with("mt-")
        || utility.starts_with("mr-")
        || utility.starts_with("mb-")
        || utility.starts_with("ml-")
        || utility.starts_with("ms-")
        || utility.starts_with("me-")
    {
        if utility.starts_with("mx-") {
            return Some("margin-x");
        }
        if utility.starts_with("my-") {
            return Some("margin-y");
        }
        if utility.starts_with("mt-") {
            return Some("margin-top");
        }
        if utility.starts_with("mr-") {
            return Some("margin-right");
        }
        if utility.starts_with("mb-") {
            return Some("margin-bottom");
        }
        if utility.starts_with("ml-") {
            return Some("margin-left");
        }
        if utility.starts_with("ms-") {
            return Some("margin-inline-start");
        }
        if utility.starts_with("me-") {
            return Some("margin-inline-end");
        }
        return Some("margin");
    }

    // Display
    if matches!(
        utility,
        "block"
            | "inline-block"
            | "inline"
            | "flex"
            | "inline-flex"
            | "table"
            | "inline-table"
            | "table-caption"
            | "table-cell"
            | "table-column"
            | "table-column-group"
            | "table-footer-group"
            | "table-header-group"
            | "table-row-group"
            | "table-row"
            | "flow-root"
            | "grid"
            | "inline-grid"
            | "contents"
            | "list-item"
            | "hidden"
    ) {
        return Some("display");
    }

    // Text color
    if let Some(rest) = utility.strip_prefix("text-") {
        // Check if it's a color (has a dash or is a known color)
        if is_color_value(rest) {
            return Some("text-color");
        }
        // Text size
        if matches!(
            rest,
            "xs" | "sm"
                | "base"
                | "lg"
                | "xl"
                | "2xl"
                | "3xl"
                | "4xl"
                | "5xl"
                | "6xl"
                | "7xl"
                | "8xl"
                | "9xl"
        ) {
            return Some("font-size");
        }
    }

    // Background color
    if let Some(rest) = utility.strip_prefix("bg-")
        && is_color_value(rest)
    {
        return Some("background-color");
    }

    // Border color
    if let Some(rest) = utility.strip_prefix("border-")
        && is_color_value(rest)
    {
        return Some("border-color");
    }

    // Width
    if utility.starts_with("w-") {
        return Some("width");
    }

    // Height
    if utility.starts_with("h-") {
        return Some("height");
    }

    // Font weight
    if matches!(
        utility,
        "font-thin"
            | "font-extralight"
            | "font-light"
            | "font-normal"
            | "font-medium"
            | "font-semibold"
            | "font-bold"
            | "font-extrabold"
            | "font-black"
    ) {
        return Some("font-weight");
    }

    // Flex direction
    if matches!(
        utility,
        "flex-row" | "flex-row-reverse" | "flex-col" | "flex-col-reverse"
    ) {
        return Some("flex-direction");
    }

    // Justify content
    if matches!(
        utility,
        "justify-normal"
            | "justify-start"
            | "justify-end"
            | "justify-center"
            | "justify-between"
            | "justify-around"
            | "justify-evenly"
            | "justify-stretch"
    ) {
        return Some("justify-content");
    }

    // Align items
    if matches!(
        utility,
        "items-start" | "items-end" | "items-center" | "items-baseline" | "items-stretch"
    ) {
        return Some("align-items");
    }

    // Position
    if matches!(
        utility,
        "static" | "fixed" | "absolute" | "relative" | "sticky"
    ) {
        return Some("position");
    }

    // Z-index
    if utility.starts_with("z-") {
        return Some("z-index");
    }

    // Opacity
    if utility.starts_with("opacity-") {
        return Some("opacity");
    }

    // Rounded (border-radius)
    if utility == "rounded"
        || utility.starts_with("rounded-")
        || matches!(
            utility,
            "rounded-none"
                | "rounded-sm"
                | "rounded-md"
                | "rounded-lg"
                | "rounded-xl"
                | "rounded-2xl"
                | "rounded-3xl"
                | "rounded-full"
        )
    {
        return Some("border-radius");
    }

    // Gap
    if utility.starts_with("gap-") {
        if utility.starts_with("gap-x-") {
            return Some("column-gap");
        }
        if utility.starts_with("gap-y-") {
            return Some("row-gap");
        }
        return Some("gap");
    }

    // Flex wrap
    if matches!(utility, "flex-wrap" | "flex-wrap-reverse" | "flex-nowrap") {
        return Some("flex-wrap");
    }

    // Flex grow
    if matches!(utility, "grow" | "grow-0") || utility.starts_with("grow-") {
        return Some("flex-grow");
    }

    // Flex shrink
    if matches!(utility, "shrink" | "shrink-0") || utility.starts_with("shrink-") {
        return Some("flex-shrink");
    }

    // Flex basis
    if utility.starts_with("basis-") {
        return Some("flex-basis");
    }

    // Order
    if utility.starts_with("order-")
        || matches!(utility, "order-first" | "order-last" | "order-none")
    {
        return Some("order");
    }

    // Overflow
    if matches!(
        utility,
        "overflow-auto"
            | "overflow-hidden"
            | "overflow-clip"
            | "overflow-visible"
            | "overflow-scroll"
    ) {
        return Some("overflow");
    }
    if matches!(
        utility,
        "overflow-x-auto"
            | "overflow-x-hidden"
            | "overflow-x-clip"
            | "overflow-x-visible"
            | "overflow-x-scroll"
    ) {
        return Some("overflow-x");
    }
    if matches!(
        utility,
        "overflow-y-auto"
            | "overflow-y-hidden"
            | "overflow-y-clip"
            | "overflow-y-visible"
            | "overflow-y-scroll"
    ) {
        return Some("overflow-y");
    }

    // Visibility
    if matches!(utility, "visible" | "invisible" | "collapse") {
        return Some("visibility");
    }

    // Cursor
    if utility.starts_with("cursor-") {
        return Some("cursor");
    }

    // Pointer events
    if matches!(utility, "pointer-events-none" | "pointer-events-auto") {
        return Some("pointer-events");
    }

    // User select
    if matches!(
        utility,
        "select-none" | "select-text" | "select-all" | "select-auto"
    ) {
        return Some("user-select");
    }

    // Text align
    if matches!(
        utility,
        "text-left" | "text-center" | "text-right" | "text-justify" | "text-start" | "text-end"
    ) {
        return Some("text-align");
    }

    // Vertical align
    if matches!(
        utility,
        "align-baseline"
            | "align-top"
            | "align-middle"
            | "align-bottom"
            | "align-text-top"
            | "align-text-bottom"
            | "align-sub"
            | "align-super"
    ) {
        return Some("vertical-align");
    }

    // Whitespace
    if matches!(
        utility,
        "whitespace-normal"
            | "whitespace-nowrap"
            | "whitespace-pre"
            | "whitespace-pre-line"
            | "whitespace-pre-wrap"
            | "whitespace-break-spaces"
    ) {
        return Some("whitespace");
    }

    // Word break
    if matches!(
        utility,
        "break-normal" | "break-words" | "break-all" | "break-keep"
    ) {
        return Some("word-break");
    }

    // Object fit
    if matches!(
        utility,
        "object-contain" | "object-cover" | "object-fill" | "object-none" | "object-scale-down"
    ) {
        return Some("object-fit");
    }

    // Object position
    if matches!(
        utility,
        "object-bottom"
            | "object-center"
            | "object-left"
            | "object-left-bottom"
            | "object-left-top"
            | "object-right"
            | "object-right-bottom"
            | "object-right-top"
            | "object-top"
    ) {
        return Some("object-position");
    }

    // Min/Max width
    if utility.starts_with("min-w-") {
        return Some("min-width");
    }
    if utility.starts_with("max-w-") {
        return Some("max-width");
    }

    // Min/Max height
    if utility.starts_with("min-h-") {
        return Some("min-height");
    }
    if utility.starts_with("max-h-") {
        return Some("max-height");
    }

    // Inset (top, right, bottom, left)
    if utility.starts_with("inset-") {
        if utility.starts_with("inset-x-") {
            return Some("inset-x");
        }
        if utility.starts_with("inset-y-") {
            return Some("inset-y");
        }
        return Some("inset");
    }
    if utility.starts_with("top-") {
        return Some("top");
    }
    if utility.starts_with("right-") {
        return Some("right");
    }
    if utility.starts_with("bottom-") {
        return Some("bottom");
    }
    if utility.starts_with("left-") {
        return Some("left");
    }
    if utility.starts_with("start-") {
        return Some("inset-inline-start");
    }
    if utility.starts_with("end-") {
        return Some("inset-inline-end");
    }

    // Grid template columns
    if utility.starts_with("grid-cols-") {
        return Some("grid-template-columns");
    }

    // Grid template rows
    if utility.starts_with("grid-rows-") {
        return Some("grid-template-rows");
    }

    // Grid column span
    if utility.starts_with("col-span-") || matches!(utility, "col-auto") {
        return Some("grid-column");
    }
    if utility.starts_with("col-start-") {
        return Some("grid-column-start");
    }
    if utility.starts_with("col-end-") {
        return Some("grid-column-end");
    }

    // Grid row span
    if utility.starts_with("row-span-") || matches!(utility, "row-auto") {
        return Some("grid-row");
    }
    if utility.starts_with("row-start-") {
        return Some("grid-row-start");
    }
    if utility.starts_with("row-end-") {
        return Some("grid-row-end");
    }

    // Grid auto flow
    if matches!(
        utility,
        "grid-flow-row"
            | "grid-flow-col"
            | "grid-flow-dense"
            | "grid-flow-row-dense"
            | "grid-flow-col-dense"
    ) {
        return Some("grid-auto-flow");
    }

    // Align content
    if matches!(
        utility,
        "content-normal"
            | "content-center"
            | "content-start"
            | "content-end"
            | "content-between"
            | "content-around"
            | "content-evenly"
            | "content-baseline"
            | "content-stretch"
    ) {
        return Some("align-content");
    }

    // Align self
    if matches!(
        utility,
        "self-auto" | "self-start" | "self-end" | "self-center" | "self-stretch" | "self-baseline"
    ) {
        return Some("align-self");
    }

    // Justify items
    if matches!(
        utility,
        "justify-items-start"
            | "justify-items-end"
            | "justify-items-center"
            | "justify-items-stretch"
    ) {
        return Some("justify-items");
    }

    // Justify self
    if matches!(
        utility,
        "justify-self-auto"
            | "justify-self-start"
            | "justify-self-end"
            | "justify-self-center"
            | "justify-self-stretch"
    ) {
        return Some("justify-self");
    }

    // Place content
    if matches!(
        utility,
        "place-content-center"
            | "place-content-start"
            | "place-content-end"
            | "place-content-between"
            | "place-content-around"
            | "place-content-evenly"
            | "place-content-baseline"
            | "place-content-stretch"
    ) {
        return Some("place-content");
    }

    // Place items
    if matches!(
        utility,
        "place-items-start"
            | "place-items-end"
            | "place-items-center"
            | "place-items-baseline"
            | "place-items-stretch"
    ) {
        return Some("place-items");
    }

    // Place self
    if matches!(
        utility,
        "place-self-auto"
            | "place-self-start"
            | "place-self-end"
            | "place-self-center"
            | "place-self-stretch"
    ) {
        return Some("place-self");
    }

    // Float
    if matches!(
        utility,
        "float-right" | "float-left" | "float-none" | "float-start" | "float-end"
    ) {
        return Some("float");
    }

    // Clear
    if matches!(
        utility,
        "clear-left" | "clear-right" | "clear-both" | "clear-none" | "clear-start" | "clear-end"
    ) {
        return Some("clear");
    }

    // Isolation
    if matches!(utility, "isolate" | "isolation-auto") {
        return Some("isolation");
    }

    // Box sizing
    if matches!(utility, "box-border" | "box-content") {
        return Some("box-sizing");
    }

    // Border width
    if utility == "border"
        || utility.starts_with("border-")
            && matches!(utility, "border-0" | "border-2" | "border-4" | "border-8")
    {
        return Some("border-width");
    }

    // Border style
    if matches!(
        utility,
        "border-solid"
            | "border-dashed"
            | "border-dotted"
            | "border-double"
            | "border-hidden"
            | "border-none"
    ) {
        return Some("border-style");
    }

    // Outline width
    if utility == "outline"
        || matches!(
            utility,
            "outline-0" | "outline-1" | "outline-2" | "outline-4" | "outline-8"
        )
    {
        return Some("outline-width");
    }

    // Outline style
    if matches!(
        utility,
        "outline-none" | "outline-solid" | "outline-dashed" | "outline-dotted" | "outline-double"
    ) {
        return Some("outline-style");
    }

    // Outline offset
    if utility.starts_with("outline-offset-") {
        return Some("outline-offset");
    }

    // Shadow
    if utility == "shadow"
        || utility.starts_with("shadow-")
        || matches!(
            utility,
            "shadow-sm"
                | "shadow-md"
                | "shadow-lg"
                | "shadow-xl"
                | "shadow-2xl"
                | "shadow-inner"
                | "shadow-none"
        )
    {
        return Some("box-shadow");
    }

    // Transition property
    if matches!(
        utility,
        "transition"
            | "transition-all"
            | "transition-colors"
            | "transition-opacity"
            | "transition-shadow"
            | "transition-transform"
            | "transition-none"
    ) {
        return Some("transition-property");
    }

    // Transition duration
    if utility.starts_with("duration-") {
        return Some("transition-duration");
    }

    // Transition timing
    if utility.starts_with("ease-")
        || matches!(
            utility,
            "ease-linear" | "ease-in" | "ease-out" | "ease-in-out"
        )
    {
        return Some("transition-timing-function");
    }

    // Transition delay
    if utility.starts_with("delay-") {
        return Some("transition-delay");
    }

    // Animation
    if utility.starts_with("animate-")
        || matches!(
            utility,
            "animate-none" | "animate-spin" | "animate-ping" | "animate-pulse" | "animate-bounce"
        )
    {
        return Some("animation");
    }

    // Transform origin
    if utility.starts_with("origin-") {
        return Some("transform-origin");
    }

    // Scale
    if utility.starts_with("scale-") {
        if utility.starts_with("scale-x-") {
            return Some("scale-x");
        }
        if utility.starts_with("scale-y-") {
            return Some("scale-y");
        }
        return Some("scale");
    }

    // Rotate
    if utility.starts_with("rotate-") {
        return Some("rotate");
    }

    // Translate
    if utility.starts_with("translate-x-") {
        return Some("translate-x");
    }
    if utility.starts_with("translate-y-") {
        return Some("translate-y");
    }

    // Skew
    if utility.starts_with("skew-x-") {
        return Some("skew-x");
    }
    if utility.starts_with("skew-y-") {
        return Some("skew-y");
    }

    // Aspect ratio
    if utility.starts_with("aspect-")
        || matches!(utility, "aspect-auto" | "aspect-square" | "aspect-video")
    {
        return Some("aspect-ratio");
    }

    // Line height
    if utility.starts_with("leading-") {
        return Some("line-height");
    }

    // Letter spacing
    if utility.starts_with("tracking-") {
        return Some("letter-spacing");
    }

    // Text decoration
    if matches!(
        utility,
        "underline" | "overline" | "line-through" | "no-underline"
    ) {
        return Some("text-decoration-line");
    }

    // Text decoration style
    if matches!(
        utility,
        "decoration-solid"
            | "decoration-double"
            | "decoration-dotted"
            | "decoration-dashed"
            | "decoration-wavy"
    ) {
        return Some("text-decoration-style");
    }

    // Text decoration thickness
    if matches!(
        utility,
        "decoration-auto"
            | "decoration-from-font"
            | "decoration-0"
            | "decoration-1"
            | "decoration-2"
            | "decoration-4"
            | "decoration-8"
    ) {
        return Some("text-decoration-thickness");
    }

    // Text transform
    if matches!(
        utility,
        "uppercase" | "lowercase" | "capitalize" | "normal-case"
    ) {
        return Some("text-transform");
    }

    // Text overflow
    if matches!(utility, "truncate" | "text-ellipsis" | "text-clip") {
        return Some("text-overflow");
    }

    // Font style
    if matches!(utility, "italic" | "not-italic") {
        return Some("font-style");
    }

    // List style type
    if matches!(utility, "list-none" | "list-disc" | "list-decimal") {
        return Some("list-style-type");
    }

    // List style position
    if matches!(utility, "list-inside" | "list-outside") {
        return Some("list-style-position");
    }

    None
}

/// Check if a value looks like a color (for text-*, bg-*, border-*)
fn is_color_value(value: &str) -> bool {
    // Check for common color names
    if matches!(
        value,
        "inherit"
            | "current"
            | "transparent"
            | "black"
            | "white"
            | "slate"
            | "gray"
            | "zinc"
            | "neutral"
            | "stone"
            | "red"
            | "orange"
            | "amber"
            | "yellow"
            | "lime"
            | "green"
            | "emerald"
            | "teal"
            | "cyan"
            | "sky"
            | "blue"
            | "indigo"
            | "violet"
            | "purple"
            | "fuchsia"
            | "pink"
            | "rose"
    ) {
        return true;
    }

    // Check for color with shade (e.g., red-500, blue-100)
    if value.contains('-') {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() >= 2 {
            let color = parts[0];
            let shade = parts.last().unwrap();
            if matches!(
                color,
                "slate"
                    | "gray"
                    | "zinc"
                    | "neutral"
                    | "stone"
                    | "red"
                    | "orange"
                    | "amber"
                    | "yellow"
                    | "lime"
                    | "green"
                    | "emerald"
                    | "teal"
                    | "cyan"
                    | "sky"
                    | "blue"
                    | "indigo"
                    | "violet"
                    | "purple"
                    | "fuchsia"
                    | "pink"
                    | "rose"
            ) && matches!(
                *shade,
                "50" | "100"
                    | "200"
                    | "300"
                    | "400"
                    | "500"
                    | "600"
                    | "700"
                    | "800"
                    | "900"
                    | "950"
            ) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_group() {
        assert_eq!(get_property_group("p-4"), Some("padding"));
        assert_eq!(get_property_group("px-2"), Some("padding-x"));
        assert_eq!(get_property_group("m-2"), Some("margin"));
        assert_eq!(get_property_group("block"), Some("display"));
        assert_eq!(get_property_group("hidden"), Some("display"));
        assert_eq!(get_property_group("text-red-500"), Some("text-color"));
        assert_eq!(get_property_group("text-lg"), Some("font-size"));
        assert_eq!(get_property_group("bg-blue-500"), Some("background-color"));
        assert_eq!(get_property_group("flex"), Some("display"));
        assert_eq!(get_property_group("flex-row"), Some("flex-direction"));
    }

    #[test]
    fn test_is_color_value() {
        assert!(is_color_value("red-500"));
        assert!(is_color_value("blue-100"));
        assert!(is_color_value("black"));
        assert!(is_color_value("white"));
        assert!(!is_color_value("lg"));
        assert!(!is_color_value("center"));
    }
}
