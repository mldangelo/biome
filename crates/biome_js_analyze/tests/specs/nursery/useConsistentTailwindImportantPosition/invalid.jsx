// Invalid: important at end (should be at start)
<div class="text-red-500!" />;
<div class="bg-blue-500!" />;

// Invalid: important at end with variants
<div class="hover:bg-blue-500!" />;
<div class="focus:hover:text-white!" />;

// Invalid: mixed important positions
<div class="!text-red-500 bg-blue-500!" />;
