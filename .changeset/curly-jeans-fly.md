---
"@biomejs/biome": patch
---

Added the new nursery rule [`noVueArrowFuncInWatch`](https://biomejs.dev/linter/rules/no-vue-arrow-func-in-watch/). This rule forbids using arrow functions in watchers in Vue components, because in an arrow function `this` won't reference the Vue instance.
