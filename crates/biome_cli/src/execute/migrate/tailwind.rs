//! Tailwind CSS configuration migration module.
//!
//! This module handles migrating Tailwind CSS configuration from tailwind.config.js/ts
//! to the biome.json tailwind section.

use crate::CliDiagnostic;
use crate::diagnostics::MigrationDiagnostic;
use biome_configuration::TailwindConfiguration;
use biome_console::{Console, ConsoleExt, markup};
use biome_fs::FileSystem;
use biome_js_parser::{JsParserOptions, parse};
use biome_js_syntax::{
    AnyJsExpression, AnyJsLiteralExpression, AnyJsModuleItem, AnyJsName, AnyJsObjectMember,
    AnyJsRoot, JsFileSource, JsObjectExpression,
};
use std::collections::BTreeMap;

/// Config file names to search for
const CONFIG_FILES: &[&str] = &[
    "tailwind.config.js",
    "tailwind.config.mjs",
    "tailwind.config.cjs",
    "tailwind.config.ts",
    "tailwind.config.mts",
];

/// Result of parsing a Tailwind config file
#[derive(Debug)]
pub(crate) struct Config {
    /// Path of the Tailwind config file
    pub(crate) path: &'static str,
    /// Extracted Tailwind configuration
    pub(crate) data: TailwindConfiguration,
}

/// Find and read the Tailwind config file
pub(crate) fn read_config_file(
    fs: &dyn FileSystem,
    console: &mut dyn Console,
) -> Result<Config, CliDiagnostic> {
    let working_directory = fs.working_directory().unwrap_or_default();

    for config_file in CONFIG_FILES {
        let path = working_directory.join(config_file);
        if let Ok(mut file) =
            fs.open_with_options(path.as_path(), biome_fs::OpenOptions::default().read(true))
        {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                console.log(markup! {
                    <Info>"Found Tailwind config: "<Emphasis>{config_file}</Emphasis></Info>
                });

                let config = parse_tailwind_config(&content, config_file, console);
                return Ok(Config {
                    path: config_file,
                    data: config,
                });
            }
        }
    }

    Err(CliDiagnostic::MigrateError(MigrationDiagnostic {
        reason: "Could not find a Tailwind CSS configuration file. Searched for: tailwind.config.js, tailwind.config.ts, etc.".to_string(),
    }))
}

/// Parse a Tailwind config file and extract theme values
fn parse_tailwind_config(
    content: &str,
    path: &str,
    console: &mut dyn Console,
) -> TailwindConfiguration {
    let source_type = if path.ends_with(".ts") || path.ends_with(".mts") {
        JsFileSource::ts()
    } else {
        JsFileSource::js_module()
    };

    let parsed = parse(content, source_type, JsParserOptions::default());
    let root = parsed.tree();

    let mut config = TailwindConfiguration::default();
    let mut found_any = false;

    // Get the items from the module
    let items: Box<dyn Iterator<Item = AnyJsModuleItem>> = match root {
        AnyJsRoot::JsModule(module) => Box::new(module.items().into_iter()),
        AnyJsRoot::JsScript(_) => {
            // Scripts have statements, not module items, so we can't extract exports
            console.log(markup! {
                <Warn>"Tailwind config appears to be a script, not a module. Cannot extract exports."</Warn>
            });
            return config;
        }
        _ => return config,
    };

    // Try to find the exported object (module.exports = { ... } or export default { ... })
    for item in items {
        if let Some(export_object) = find_export_object(&item) {
            // Look for theme property
            if let Some(theme_object) = find_object_property(&export_object, "theme") {
                // Look for extend property within theme
                if let Some(extend_object) = find_object_property(&theme_object, "extend") {
                    // Extract theme values from extend
                    if let Some(spacing) = extract_string_map(&extend_object, "spacing")
                        && !spacing.is_empty()
                    {
                        config.spacing = Some(spacing);
                        found_any = true;
                    }
                    if let Some(opacity) = extract_string_map(&extend_object, "opacity")
                        && !opacity.is_empty()
                    {
                        config.opacity = Some(opacity);
                        found_any = true;
                    }
                    if let Some(z_index) = extract_string_map(&extend_object, "zIndex")
                        && !z_index.is_empty()
                    {
                        config.z_index = Some(z_index);
                        found_any = true;
                    }
                    if let Some(font_size) = extract_string_map(&extend_object, "fontSize")
                        && !font_size.is_empty()
                    {
                        config.font_size = Some(font_size);
                        found_any = true;
                    }
                    if let Some(border_radius) = extract_string_map(&extend_object, "borderRadius")
                        && !border_radius.is_empty()
                    {
                        config.border_radius = Some(border_radius);
                        found_any = true;
                    }
                }

                // Also check for direct theme properties (not in extend)
                if config.spacing.is_none()
                    && let Some(spacing) = extract_string_map(&theme_object, "spacing")
                    && !spacing.is_empty()
                {
                    config.spacing = Some(spacing);
                    found_any = true;
                }
            }
        }
    }

    if !found_any {
        console.log(markup! {
            <Warn>"Could not extract theme values from the Tailwind config. The config may use dynamic values or a format that cannot be statically analyzed."</Warn>
        });
        console.log(markup! {
            <Info>"You can manually configure Tailwind theme values in your biome.json:"</Info>
        });
        console.log(markup! {
            <Info>"  \"tailwind\": {{ \"spacing\": {{ \"3.25rem\": \"13\" }} }}"</Info>
        });
    }

    config
}

/// Find the exported object in a module (module.exports = { ... } or export default { ... })
fn find_export_object(item: &biome_js_syntax::AnyJsModuleItem) -> Option<JsObjectExpression> {
    use biome_js_syntax::AnyJsModuleItem;

    match item {
        // Handle: export default { ... }
        AnyJsModuleItem::JsExport(export) => {
            let clause = export.export_clause().ok()?;
            if let biome_js_syntax::AnyJsExportClause::JsExportDefaultExpressionClause(
                default_clause,
            ) = clause
            {
                let expr = default_clause.expression().ok()?;
                return expr.as_js_object_expression().cloned();
            }
            None
        }
        // Handle: module.exports = { ... }
        AnyJsModuleItem::AnyJsStatement(stmt) => {
            if let biome_js_syntax::AnyJsStatement::JsExpressionStatement(expr_stmt) = stmt {
                let expr = expr_stmt.expression().ok()?;
                if let AnyJsExpression::JsAssignmentExpression(assign) = expr {
                    let left = assign.left().ok()?;
                    // Check if it's module.exports
                    if let biome_js_syntax::AnyJsAssignmentPattern::AnyJsAssignment(
                        biome_js_syntax::AnyJsAssignment::JsStaticMemberAssignment(member),
                    ) = left
                    {
                        let object = member.object().ok()?;
                        let member_name = member.member().ok()?;
                        if let AnyJsExpression::JsIdentifierExpression(ident) = object {
                            let name = ident.name().ok()?;
                            let name_token = name.value_token().ok()?;
                            if name_token.text_trimmed() == "module"
                                && let AnyJsName::JsName(member_js_name) = member_name
                                && let Ok(member_token) = member_js_name.value_token()
                                && member_token.text_trimmed() == "exports"
                            {
                                let right = assign.right().ok()?;
                                return right.as_js_object_expression().cloned();
                            }
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Find a property in an object expression
fn find_object_property(object: &JsObjectExpression, name: &str) -> Option<JsObjectExpression> {
    for member in object.members() {
        if let Ok(AnyJsObjectMember::JsPropertyObjectMember(prop)) = member
            && let Ok(member_name) = prop.name()
        {
            let key_text = match member_name {
                biome_js_syntax::AnyJsObjectMemberName::JsLiteralMemberName(lit) => {
                    lit.name().ok()?.text().to_string()
                }
                biome_js_syntax::AnyJsObjectMemberName::JsComputedMemberName(_) => continue,
                biome_js_syntax::AnyJsObjectMemberName::JsMetavariable(_) => continue,
            };

            if key_text == name
                && let Ok(value) = prop.value()
            {
                return value.as_js_object_expression().cloned();
            }
        }
    }
    None
}

/// Extract a string map from an object property (e.g., spacing: { '3.25rem': '13' })
fn extract_string_map(object: &JsObjectExpression, name: &str) -> Option<BTreeMap<String, String>> {
    let target = find_object_property(object, name)?;
    let mut map = BTreeMap::new();

    for member in target.members() {
        if let Ok(AnyJsObjectMember::JsPropertyObjectMember(prop)) = member
            && let (Ok(key), Ok(value)) = (prop.name(), prop.value())
        {
            let key_text = match key {
                biome_js_syntax::AnyJsObjectMemberName::JsLiteralMemberName(lit) => {
                    let name = lit.name().ok()?;
                    // Remove quotes if present
                    name.text()
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string()
                }
                _ => continue,
            };

            // Try to get string value
            if let AnyJsExpression::AnyJsLiteralExpression(
                AnyJsLiteralExpression::JsStringLiteralExpression(ref str_lit),
            ) = value
                && let Ok(token) = str_lit.value_token()
            {
                let value_text = token
                    .text()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string();
                map.insert(key_text, value_text);
            }
            // Also handle number literals (e.g., zIndex: { 60: '60' } might be zIndex: { 60: 60 })
            else if let AnyJsExpression::AnyJsLiteralExpression(
                AnyJsLiteralExpression::JsNumberLiteralExpression(num_lit),
            ) = value
                && let Ok(token) = num_lit.value_token()
            {
                let value_text = token.text().to_string();
                map.insert(key_text, value_text);
            }
        }
    }

    if map.is_empty() { None } else { Some(map) }
}
