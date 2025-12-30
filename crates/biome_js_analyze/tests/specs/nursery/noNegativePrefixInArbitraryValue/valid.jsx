/* should not generate diagnostics */

// ==========================================
// Negative value inside brackets (preferred)
// ==========================================
<div class="top-[-10px]" />;
<div class="m-[-1rem]" />;
<div class="translate-x-[-50%]" />;
<div class="translate-y-[calc(-50%-1rem)]" />;
<div class="inset-[-1rem]" />;
<div class="indent-[-7px]" />;
<div class="left-[calc(var(--spacing)*-1)]" />;

// ==========================================
// Dash prefix with standard values (no arbitrary)
// ==========================================
<div class="-m-4" />;
<div class="-mt-4" />;
<div class="-mx-8" />;
<div class="-translate-x-4" />;
<div class="-translate-y-1/2" />;
<div class="-top-4" />;
<div class="-left-8" />;
<div class="-inset-4" />;
<div class="-scroll-m-4" />;
<div class="-indent-4" />;
<div class="-space-x-4" />;
<div class="-z-10" />;

// ==========================================
// Positive arbitrary values without dash prefix
// ==========================================
<div class="m-[1rem]" />;
<div class="translate-x-[50%]" />;
<div class="top-[10px]" />;
<div class="left-[var(--spacing)]" />;

// ==========================================
// Non-negatable utilities (should be ignored)
// ==========================================
<div class="w-[-100px]" />;
<div class="h-[-50vh]" />;
<div class="p-[-1rem]" />;
<div class="text-[-16px]" />;

// ==========================================
// Standard positive utilities
// ==========================================
<div class="m-4 p-4 w-full" />;
<div class="translate-x-4 translate-y-1/2" />;

// ==========================================
// Non-Tailwind classes
// ==========================================
<div class="custom-class" />;
<div class="my-component" />;

// ==========================================
// Mixed valid classes
// ==========================================
<div class="flex items-center -m-4 translate-x-[-50%]" />;

// ==========================================
// With variants (valid patterns)
// ==========================================
<div class="hover:m-[-1rem]" />;
<div class="hover:-m-4" />;
<div class="dark:md:translate-x-[-50%]" />;
