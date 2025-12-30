/* should not generate diagnostics */

// Valid: standard Tailwind utilities
<div class="flex items-center justify-between p-4" />;
<div class="mx-auto my-2 w-full h-screen" />;
<div class="text-red-500 bg-blue-100 border-gray-300" />;

// Valid: with variants
<div class="hover:bg-blue-500 dark:text-white" />;
<div class="sm:flex md:grid lg:hidden" />;
<div class="focus:outline-none focus:ring-2" />;

// Valid: arbitrary values
<div class="w-[100px] h-[50vh]" />;
<div class="bg-[#ff0000] text-[var(--color)]" />;
<div class="top-[calc(100%-20px)]" />;

// Valid: negative values
<div class="-mt-4 -translate-x-1/2" />;

// Valid: important modifier
<div class="!mt-0 !hidden" />;

// Valid: complex combinations
<div class="group-hover:opacity-100 peer-checked:bg-blue-500" />;
<div class="aria-checked:bg-blue-500 data-[state=open]:bg-red-500" />;
