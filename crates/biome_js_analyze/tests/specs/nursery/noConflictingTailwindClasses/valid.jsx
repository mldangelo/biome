/* should not generate diagnostics */

// Valid: non-conflicting classes
<div class="p-4 m-2" />;
<div class="text-red-500 bg-blue-500" />;
<div class="flex items-center justify-between" />;

// Valid: different variants don't conflict
<div class="p-2 hover:p-4" />;
<div class="text-red-500 hover:text-blue-500" />;

// Valid: different specific properties
<div class="pt-2 pb-4" />;
<div class="px-2 py-4" />;
<div class="mt-2 mb-4" />;

// Valid: single class
<div class="p-4" />;
<div class="flex" />;
