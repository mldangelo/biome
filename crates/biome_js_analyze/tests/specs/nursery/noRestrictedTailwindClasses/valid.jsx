/* should not generate diagnostics */

// Valid: non-restricted classes (without options, no classes are restricted)
<div class="flex p-4" />;
<div class="bg-blue-500 text-white" />;
<div class="hidden" />;

// Valid: with variants
<div class="hover:bg-blue-500" />;
<div class="md:flex lg:block" />;
