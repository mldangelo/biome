use biome_js_syntax::{
    AnyJsExpression, AnyJsObjectMemberName, JsCallArguments, JsCallExpression, JsLiteralMemberName,
    JsPropertyObjectMember, JsStringLiteralExpression, JsSyntaxNode, JsTemplateChunkElement,
    JsTemplateExpression, JsxAttribute, JsxString,
};
use biome_rowan::{AstNode, TokenText, declare_node_union};

/// A trait for options that can be used with `AnyClassStringLike::should_visit`.
///
/// This trait abstracts over the attribute/function matching logic so that
/// multiple rule options types can share the same `should_visit` implementation.
pub trait TailwindClassOptions {
    /// Check if an attribute name should be inspected for Tailwind classes.
    fn has_attribute(&self, name: &str) -> bool;

    /// Check if a function name should be inspected for Tailwind classes.
    fn has_function(&self, name: &str) -> bool;

    /// Check if a function name matches using wildcard patterns.
    fn match_function(&self, name: &str) -> bool;

    /// Check if an object key should be ignored (its values won't be inspected).
    fn has_ignored_key(&self, name: &str) -> bool;
}

fn get_callee_name(call_expression: &JsCallExpression) -> Option<TokenText> {
    call_expression
        .callee()
        .ok()?
        .as_js_identifier_expression()?
        .name()
        .ok()?
        .name()
        .ok()
}

fn is_call_expression_of_target_function<O: TailwindClassOptions>(
    call_expression: &JsCallExpression,
    options: &O,
) -> bool {
    get_callee_name(call_expression).is_some_and(|name| options.has_function(name.text()))
}

fn get_attribute_name(attribute: &JsxAttribute) -> Option<TokenText> {
    Some(
        attribute
            .name()
            .ok()?
            .as_jsx_name()?
            .value_token()
            .ok()?
            .token_text_trimmed(),
    )
}

declare_node_union! {
    /// A string literal, JSX string, or template chunk representing a CSS class string.
    pub AnyClassStringLike = JsStringLiteralExpression | JsxString | JsTemplateChunkElement | JsLiteralMemberName
}

/// Get the name of an object property key
fn get_property_key_name(property: &JsPropertyObjectMember) -> Option<TokenText> {
    match property.name().ok()? {
        AnyJsObjectMemberName::JsLiteralMemberName(name) => name.name().ok(),
        AnyJsObjectMemberName::JsComputedMemberName(_) => None, // Can't determine at compile time
        AnyJsObjectMemberName::JsMetavariable(_) => None,
    }
}

fn inspect_string_literal<O: TailwindClassOptions>(
    node: &JsSyntaxNode,
    options: &O,
) -> Option<bool> {
    let mut in_arguments = false;
    let mut in_function = false;
    for ancestor in node.ancestors().skip(1) {
        // Check if we're inside an ignored object property
        if let Some(property) = JsPropertyObjectMember::cast_ref(&ancestor)
            && let Some(key_name) = get_property_key_name(&property)
            && options.has_ignored_key(key_name.text())
        {
            return None; // Skip values under ignored keys
        }

        if let Some(jsx_attribute) = JsxAttribute::cast_ref(&ancestor) {
            let attribute_name = get_attribute_name(&jsx_attribute)?;
            if options.has_attribute(attribute_name.text()) {
                return Some(true);
            }
        }

        if let Some(call_expression) = JsCallExpression::cast_ref(&ancestor) {
            in_function = is_call_expression_of_target_function(&call_expression, options);
        }

        if JsCallArguments::can_cast(ancestor.kind()) {
            in_arguments = true;
        }

        if in_function && in_arguments {
            return Some(true);
        }
    }

    None
}

impl AnyClassStringLike {
    pub(crate) fn should_visit<O: TailwindClassOptions>(&self, options: &O) -> Option<bool> {
        match self {
            Self::JsStringLiteralExpression(string_literal) => {
                inspect_string_literal(string_literal.syntax(), options)
            }
            Self::JsLiteralMemberName(literal_name) => {
                inspect_string_literal(literal_name.syntax(), options)
            }
            Self::JsxString(jsx_string) => {
                let jsx_attribute = jsx_string
                    .syntax()
                    .ancestors()
                    .skip(1)
                    .find_map(JsxAttribute::cast)?;
                let name = get_attribute_name(&jsx_attribute)?;
                if options.has_attribute(name.text()) {
                    return Some(true);
                }

                None
            }
            Self::JsTemplateChunkElement(template) => {
                for ancestor in template.syntax().ancestors().skip(1) {
                    if let Some(template_expression) = JsTemplateExpression::cast_ref(&ancestor) {
                        if let Some(AnyJsExpression::JsIdentifierExpression(tag)) =
                            template_expression.tag()
                        {
                            let name = tag.name().ok()?.name().ok()?;
                            if options.has_function(name.text()) {
                                return Some(true);
                            }
                        }
                        if let Some(AnyJsExpression::JsStaticMemberExpression(tag)) =
                            template_expression.tag()
                            && options.match_function(tag.to_string().as_ref())
                        {
                            return Some(true);
                        }
                    } else if let Some(jsx_attribute) = JsxAttribute::cast_ref(&ancestor) {
                        let attribute_name = get_attribute_name(&jsx_attribute)?;
                        if options.has_attribute(attribute_name.text()) {
                            return Some(true);
                        }
                    }
                }

                None
            }
        }
    }

    pub fn value(&self) -> Option<TokenText> {
        match &self {
            Self::JsStringLiteralExpression(node) => node.inner_string_text().ok(),
            Self::JsxString(node) => node.inner_string_text().ok(),
            Self::JsTemplateChunkElement(template_chunk) => {
                Some(template_chunk.template_chunk_token().ok()?.token_text())
            }
            Self::JsLiteralMemberName(node) => node.name().ok(),
        }
    }
}

// Implement the trait for UseSortedClassesOptions
impl TailwindClassOptions for biome_rule_options::use_sorted_classes::UseSortedClassesOptions {
    fn has_attribute(&self, name: &str) -> bool {
        self.has_attribute(name)
    }

    fn has_function(&self, name: &str) -> bool {
        self.has_function(name)
    }

    fn match_function(&self, name: &str) -> bool {
        self.match_function(name)
    }

    fn has_ignored_key(&self, name: &str) -> bool {
        self.has_ignored_key(name)
    }
}

// Implement the trait for UseConsistentTailwindImportantPositionOptions
impl TailwindClassOptions
    for biome_rule_options::use_consistent_tailwind_important_position::UseConsistentTailwindImportantPositionOptions
{
    fn has_attribute(&self, name: &str) -> bool {
        self.has_attribute(name)
    }

    fn has_function(&self, name: &str) -> bool {
        self.has_function(name)
    }

    fn match_function(&self, name: &str) -> bool {
        self.match_function(name)
    }

    fn has_ignored_key(&self, name: &str) -> bool {
        self.has_ignored_key(name)
    }
}

// Implement the trait for UseConsistentTailwindLineWrappingOptions
impl TailwindClassOptions
    for biome_rule_options::use_consistent_tailwind_line_wrapping::UseConsistentTailwindLineWrappingOptions
{
    fn has_attribute(&self, name: &str) -> bool {
        self.has_attribute(name)
    }

    fn has_function(&self, name: &str) -> bool {
        self.has_function(name)
    }

    fn match_function(&self, name: &str) -> bool {
        self.match_function(name)
    }

    fn has_ignored_key(&self, name: &str) -> bool {
        self.has_ignored_key(name)
    }
}

// Implement the trait for NoUnregisteredTailwindClassesOptions
impl TailwindClassOptions
    for biome_rule_options::no_unregistered_tailwind_classes::NoUnregisteredTailwindClassesOptions
{
    fn has_attribute(&self, name: &str) -> bool {
        self.has_attribute(name)
    }

    fn has_function(&self, name: &str) -> bool {
        self.has_function(name)
    }

    fn match_function(&self, name: &str) -> bool {
        self.match_function(name)
    }

    fn has_ignored_key(&self, name: &str) -> bool {
        self.has_ignored_key(name)
    }
}
