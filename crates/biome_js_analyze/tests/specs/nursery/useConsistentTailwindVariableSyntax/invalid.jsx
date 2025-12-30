// Invalid: using var() syntax when simpler syntax available
<div class="bg-[var(--my-color)]" />;
<div class="text-[var(--brand-primary)]" />;
<div class="border-[var(--border-color)]" />;

// Invalid: with variants
<div class="hover:bg-[var(--hover-color)]" />;
<div class="dark:text-[var(--dark-text)]" />;

// Invalid: multiple classes
<div class="bg-[var(--bg)] text-[var(--text)]" />;
