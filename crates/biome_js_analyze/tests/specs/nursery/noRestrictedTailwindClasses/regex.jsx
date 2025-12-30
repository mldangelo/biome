// Test regex pattern matching

// Match bg-red-* classes
<div class="bg-red-500" />;
<div class="bg-red-100" />;
<div class="hover:bg-red-500" />;

// Match text-xs and text-sm
<div class="text-xs" />;
<div class="text-sm" />;

// Should not match (doesn't match pattern)
<div class="bg-blue-500 text-lg" />;

// Mixed: some match, some don't
<div class="flex bg-red-500 text-xs p-4" />;
