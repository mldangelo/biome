// Invalid: px + py can be p
<div class="px-4 py-4" />;

// Invalid: mx + my can be m
<div class="mx-2 my-2" />;

// Invalid: pt + pr + pb + pl can be p
<div class="pt-2 pr-2 pb-2 pl-2" />;

// Invalid: mt + mr + mb + ml can be m
<div class="mt-4 mr-4 mb-4 ml-4" />;

// Invalid: with other classes
<div class="flex px-4 py-4 items-center" />;

// Invalid: w + h can be size (Tailwind v3.4+)
<div class="w-4 h-4" />;

// Invalid: w + h with other values
<div class="w-full h-full" />;

// Invalid: w + h with variants
<div class="hover:w-8 hover:h-8" />;

// Invalid: w + h with other classes
<div class="flex w-10 h-10 items-center" />;
