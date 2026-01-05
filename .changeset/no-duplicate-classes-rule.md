---
"@biomejs/biome": patch
---

Added the assist action [`noDuplicateClasses`](https://biomejs.dev/assist/actions/no-duplicate-classes/) to remove duplicate CSS classes from HTML and JSX files. Supports utility functions like `clsx`, `cn`, and `cva`.

```jsx
// Before
<div class="flex p-4 flex" />;

// After
<div class="flex p-4" />;
```
