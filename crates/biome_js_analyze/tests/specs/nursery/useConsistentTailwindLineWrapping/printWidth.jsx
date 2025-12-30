// Test printWidth option - classes should be wrapped at 40 characters

// Long class strings that should be wrapped
<div class="flex items-center justify-between p-4 m-2 bg-blue-500" />;

// Already wrapped correctly (within width)
<div class="flex items-center" />;

// Single line that's too long
<div class="bg-blue-500 text-white p-4 rounded-lg shadow-md hover:bg-blue-600 focus:ring-2" />;
