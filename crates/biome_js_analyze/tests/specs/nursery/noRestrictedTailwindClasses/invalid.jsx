// Invalid: restricted class "hidden"
<div class="hidden" />;

// Invalid: restricted class with variants
<div class="hover:hidden" />;
<div class="md:hover:hidden" />;

// Invalid: restricted class "float-left"
<div class="float-left" />;

// Invalid: multiple restricted classes
<div class="hidden flex float-left" />;
