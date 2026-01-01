---
"@biomejs/biome": patch
---

Added the new nursery rule `noInvalidTailwindVariantCombination`.

Detects invalid variant combinations in Tailwind CSS classes:
- Duplicate variants (`hover:hover:bg-red-500`)
- Conflicting responsive breakpoints (`sm:md:flex`)
- Mutually exclusive positional variants (`first:last:text-bold`)

```jsx
// Invalid
<div class="hover:hover:bg-red-500" />;  // duplicate
<div class="sm:md:flex" />;              // conflicting breakpoints
<div class="first:last:text-bold" />;    // mutually exclusive

// Valid
<div class="hover:focus:bg-red-500" />;  // different state variants
<div class="sm:flex md:block" />;        // separate classes
```
