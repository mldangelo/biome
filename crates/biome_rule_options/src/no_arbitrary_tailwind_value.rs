use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

/// Options for the `noArbitraryTailwindValue` rule.
#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoArbitraryTailwindValueOptions {
    /// Patterns to allow. Each pattern is treated as a regex.
    /// Use this to allow specific arbitrary value patterns that are
    /// commonly needed, such as `bg-\[url\(.*\)\]` for background images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Box<[Box<str>]>>,
}
