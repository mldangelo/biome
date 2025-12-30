// Test auto-fix with replacement
// hidden -> invisible (with replacement)
<div class="hidden" />;
<div class="hover:hidden" />;
<div class="md:hover:hidden" />;

// float-left -> flex (with replacement)
<div class="float-left" />;

// multiple restricted classes with replacements
<div class="hidden flex float-left p-4" />;

// deprecated -> removed (no replacement, just removal)
<div class="deprecated text-lg" />;

// Template literal
const cls = `hidden px-4`;

// JavaScript string
const className = "hidden float-left";
