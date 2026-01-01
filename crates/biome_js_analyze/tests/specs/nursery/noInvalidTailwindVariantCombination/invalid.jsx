// Invalid - duplicate variants
<div class="hover:hover:bg-red-500" />;
<div class="focus:focus:text-white" />;
<div class="sm:sm:flex" />;

// Invalid - conflicting responsive variants
<div class="sm:md:flex" />;
<div class="lg:xl:hidden" />;

// Invalid - conflicting max-width variants
<div class="max-sm:max-md:flex" />;

// Invalid - mutually exclusive positional variants
<div class="first:last:text-bold" />;
<div class="odd:even:bg-gray-100" />;
<div class="first-of-type:last-of-type:text-lg" />;

// Invalid - duplicate with named groups (should normalize)
<div class="group-hover/a:group-hover/b:text-white" />;

// Invalid - multiple issues in one class string
<div class="hover:hover:bg-red-500 sm:md:flex first:last:text-bold" />;
