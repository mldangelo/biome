/* should not generate diagnostics */

// Valid: important at start (default position)
<div class="!text-red-500" />;
<div class="!bg-blue-500" />;

// Valid: important at start with variants
<div class="hover:!bg-blue-500" />;
<div class="focus:hover:!text-white" />;

// Valid: no important modifier
<div class="text-red-500 bg-blue-500" />;
<div class="hover:bg-blue-500" />;
