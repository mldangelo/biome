// ==========================================
// Double negatives (dash prefix + negative value)
// ==========================================
<div class="-top-[-10px]" />;
<div class="-m-[-1rem]" />;
<div class="-mt-[-16px]" />;
<div class="-translate-x-[-50%]" />;

// ==========================================
// Dash prefix with positive values (should use negative inside brackets)
// ==========================================
<div class="-top-[10px]" />;
<div class="-m-[1rem]" />;
<div class="-mt-[16px]" />;
<div class="-translate-x-[50%]" />;
<div class="-translate-y-[25%]" />;
<div class="-inset-[1rem]" />;
<div class="-left-[16px]" />;
<div class="-right-[2rem]" />;

// ==========================================
// CSS variables (should wrap in calc)
// ==========================================
<div class="-left-[var(--spacing)]" />;
<div class="-top-[var(--offset)]" />;
<div class="-m-[var(--margin)]" />;

// ==========================================
// Calc expressions
// ==========================================
<div class="-left-[calc(100%-1rem)]" />;

// ==========================================
// With variants
// ==========================================
<div class="hover:-m-[1rem]" />;
<div class="hover:-m-[-1rem]" />;
<div class="dark:md:-mt-[16px]" />;
<div class="focus:-translate-x-[50%]" />;

// ==========================================
// Multiple violations
// ==========================================
<div class="-m-[1rem] -translate-x-[50%]" />;
<div class="-m-[-1rem] -translate-x-[-50%]" />;

// ==========================================
// With important modifier (prefix)
// ==========================================
<div class="!-top-[10px]" />;
<div class="!-m-[1rem]" />;
<div class="!-top-[-10px]" />;
<div class="hover:!-m-[1rem]" />;

// ==========================================
// With important modifier (suffix)
// ==========================================
<div class="-top-[10px]!" />;
<div class="-m-[1rem]!" />;
<div class="-top-[-10px]!" />;
<div class="hover:-m-[1rem]!" />;
