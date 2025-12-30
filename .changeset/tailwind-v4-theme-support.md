---
"@biomejs/biome": patch
---

Added comprehensive Tailwind CSS v4 support with 13 new nursery lint rules and configuration options.

#### New Lint Rules

- [`noDuplicateTailwindClasses`](https://biomejs.dev/linter/rules/no-duplicate-tailwind-classes/): Detect duplicate Tailwind classes.
- [`noUnnecessaryTailwindWhitespace`](https://biomejs.dev/linter/rules/no-unnecessary-tailwind-whitespace/): Remove excessive whitespace in class strings.
- [`useConsistentTailwindImportantPosition`](https://biomejs.dev/linter/rules/use-consistent-tailwind-important-position/): Enforce consistent `!` placement for important modifiers.
- [`noDeprecatedTailwindClasses`](https://biomejs.dev/linter/rules/no-deprecated-tailwind-classes/): Detect deprecated Tailwind v3 classes.
- [`noRestrictedTailwindClasses`](https://biomejs.dev/linter/rules/no-restricted-tailwind-classes/): Configure banned classes with optional auto-fix replacements.
- [`noConflictingTailwindClasses`](https://biomejs.dev/linter/rules/no-conflicting-tailwind-classes/): Detect conflicting utility classes.
- [`useTailwindShorthandClasses`](https://biomejs.dev/linter/rules/use-tailwind-shorthand-classes/): Suggest shorthand variants like `p-*` instead of `px-*/py-*`.
- [`useConsistentTailwindVariableSyntax`](https://biomejs.dev/linter/rules/use-consistent-tailwind-variable-syntax/): Enforce `var()` vs bracket syntax.
- [`noUnregisteredTailwindClasses`](https://biomejs.dev/linter/rules/no-unregistered-tailwind-classes/): Detect classes not in the Tailwind preset.
- [`useConsistentTailwindLineWrapping`](https://biomejs.dev/linter/rules/use-consistent-tailwind-line-wrapping/): Auto-wrap long class lists at a configurable width.
- [`noUnnecessaryArbitraryValue`](https://biomejs.dev/linter/rules/no-unnecessary-arbitrary-value/): Suggest theme values over arbitrary values.
- [`noNegativePrefixInArbitraryValue`](https://biomejs.dev/linter/rules/no-negative-prefix-in-arbitrary-value/): Enforce `m-[-10px]` over `-m-[10px]`.
- [`noArbitraryTailwindValue`](https://biomejs.dev/linter/rules/no-arbitrary-tailwind-value/): Disallow arbitrary values entirely.

#### New Rule Options

Added shared options for Tailwind rules:

- `classRegex`: Match custom attribute patterns (e.g., `.*[Cc]lass.*`).
- `ignoredKeys`: Skip specific object keys in class detection.
- `functions`: Configure function calls that contain class strings.
- `attributes`: Additional JSX attributes to check beyond `class` and `className`.

#### Tailwind v4 Configuration

Added `tailwind.cssPath` configuration option to specify a Tailwind v4 CSS file:

```json
{
  "tailwind": {
    "cssPath": "./src/app.css"
  }
}
```

Biome parses `@theme` blocks to extract custom theme values for use by lint rules.

#### New CLI Command

Added `biome migrate tailwind` command to migrate Tailwind v3 JavaScript configs to Biome's configuration format.

#### Known Limitations

Biome's Tailwind v4 support uses static CSS analysis rather than loading the full Tailwind design system at runtime. This provides fast, zero-dependency linting but has some limitations:

- **Plugin utilities not detected**: Classes from plugins like DaisyUI, Flowbite, or custom plugins won't be recognized automatically by `noUnregisteredTailwindClasses`. Use the `whitelist` option to allow these classes.
- **Complex presets**: Dynamic presets that compute values at build time may not be fully captured.
- **`@import` resolution**: Only local CSS files referenced via `@import` are resolved. Remote URLs and npm packages are not followed.

If you use third-party plugins, configure the `whitelist` option on relevant rules:

```json
{
  "linter": {
    "rules": {
      "nursery": {
        "noUnregisteredTailwindClasses": {
          "level": "warn",
          "options": {
            "whitelist": ["btn", "btn-*", "card", "card-*", "daisy-*"]
          }
        }
      }
    }
  }
}
```
