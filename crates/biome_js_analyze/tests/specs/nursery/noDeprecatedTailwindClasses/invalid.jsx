// Invalid: deprecated flex-shrink
<div class="flex-shrink" />;
<div class="flex-shrink-0" />;

// Invalid: deprecated flex-grow
<div class="flex-grow" />;
<div class="flex-grow-0" />;

// Invalid: deprecated overflow utilities
<div class="overflow-ellipsis" />;
<div class="overflow-clip" />;

// Invalid: deprecated decoration utilities
<div class="decoration-slice" />;
<div class="decoration-clone" />;

// Invalid: with variants
<div class="hover:flex-shrink" />;
<div class="focus:hover:flex-grow-0" />;

// Invalid: mixed deprecated and valid classes
<div class="flex-shrink p-4 flex-grow" />;
