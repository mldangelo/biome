/* should not generate diagnostics */
// Valid - standard Tailwind utilities
<div class="w-24" />;
<div class="bg-red-500" />;
<div class="text-sm p-2" />;
<div class="hover:bg-blue-500" />;
<div className="m-4 p-4 flex" />;

// Empty and whitespace-only are valid
<div class="" />;
<div class="   " />;
