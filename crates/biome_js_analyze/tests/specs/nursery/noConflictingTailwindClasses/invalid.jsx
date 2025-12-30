// ==========================================
// Spacing conflicts
// ==========================================
// Padding
<div class="p-2 p-4" />;
<div class="px-2 px-4" />;
<div class="py-2 py-4" />;
<div class="pt-2 pt-4" />;

// Margin
<div class="m-2 m-4" />;
<div class="mx-2 mx-4" />;
<div class="my-2 my-4" />;
<div class="mt-2 mt-4" />;

// Gap
<div class="gap-2 gap-4" />;
<div class="gap-x-2 gap-x-4" />;
<div class="gap-y-2 gap-y-4" />;

// ==========================================
// Display conflicts
// ==========================================
<div class="hidden block" />;
<div class="flex grid" />;
<div class="inline block" />;
<div class="inline-flex inline-block" />;

// ==========================================
// Flexbox conflicts
// ==========================================
<div class="flex-row flex-col" />;
<div class="flex-wrap flex-nowrap" />;
<div class="flex-grow flex-grow-0" />;
<div class="shrink shrink-0" />;
<div class="grow grow-0" />;
<div class="basis-0 basis-full" />;

// ==========================================
// Grid conflicts
// ==========================================
<div class="grid-cols-1 grid-cols-2" />;
<div class="grid-rows-1 grid-rows-2" />;
<div class="grid-flow-row grid-flow-col" />;
<div class="auto-cols-min auto-cols-max" />;
<div class="auto-rows-min auto-rows-max" />;

// ==========================================
// Alignment conflicts
// ==========================================
<div class="justify-start justify-center" />;
<div class="justify-items-start justify-items-center" />;
<div class="justify-self-start justify-self-center" />;
<div class="items-start items-center" />;
<div class="content-start content-center" />;
<div class="self-start self-center" />;
<div class="place-content-start place-content-center" />;
<div class="place-items-start place-items-center" />;
<div class="place-self-start place-self-center" />;

// ==========================================
// Size conflicts
// ==========================================
<div class="w-full w-1/2" />;
<div class="h-full h-1/2" />;
<div class="min-w-0 min-w-full" />;
<div class="max-w-sm max-w-lg" />;
<div class="min-h-0 min-h-full" />;
<div class="max-h-screen max-h-full" />;
<div class="size-4 size-8" />;

// ==========================================
// Color conflicts
// ==========================================
<div class="text-red-500 text-blue-500" />;
<div class="bg-red-500 bg-blue-500" />;
<div class="border-red-500 border-blue-500" />;
<div class="ring-red-500 ring-blue-500" />;
<div class="outline-red-500 outline-blue-500" />;
<div class="accent-red-500 accent-blue-500" />;
<div class="caret-red-500 caret-blue-500" />;
<div class="fill-red-500 fill-blue-500" />;
<div class="stroke-red-500 stroke-blue-500" />;

// ==========================================
// Typography conflicts
// ==========================================
<div class="font-bold font-normal" />;
<div class="font-sans font-serif" />;
<div class="text-sm text-lg" />;
<div class="text-left text-center" />;
<div class="tracking-tight tracking-wide" />;
<div class="leading-tight leading-loose" />;
<div class="underline line-through" />;
<div class="uppercase lowercase" />;
<div class="whitespace-normal whitespace-nowrap" />;
<div class="break-normal break-all" />;
<div class="align-top align-middle" />;

// ==========================================
// Position conflicts
// ==========================================
<div class="static absolute" />;
<div class="relative fixed" />;
<div class="top-0 top-4" />;
<div class="right-0 right-4" />;
<div class="bottom-0 bottom-4" />;
<div class="left-0 left-4" />;
<div class="inset-0 inset-4" />;
<div class="inset-x-0 inset-x-4" />;
<div class="inset-y-0 inset-y-4" />;
<div class="float-left float-right" />;
<div class="clear-left clear-right" />;

// ==========================================
// Z-index conflicts
// ==========================================
<div class="z-0 z-10" />;
<div class="z-auto z-50" />;

// ==========================================
// Visibility conflicts
// ==========================================
<div class="visible invisible" />;

// ==========================================
// Overflow conflicts
// ==========================================
<div class="overflow-auto overflow-hidden" />;
<div class="overflow-x-auto overflow-x-hidden" />;
<div class="overflow-y-auto overflow-y-hidden" />;

// ==========================================
// Cursor conflicts
// ==========================================
<div class="cursor-pointer cursor-wait" />;
<div class="pointer-events-none pointer-events-auto" />;
<div class="select-none select-all" />;

// ==========================================
// Object fit/position conflicts
// ==========================================
<div class="object-contain object-cover" />;
<div class="object-center object-top" />;

// ==========================================
// Border conflicts
// ==========================================
<div class="border-0 border-2" />;
<div class="border-t-0 border-t-2" />;
<div class="rounded-none rounded-lg" />;
<div class="rounded-t-none rounded-t-lg" />;
<div class="border-solid border-dashed" />;
<div class="outline-none outline-1" />;

// ==========================================
// Shadow conflicts
// ==========================================
<div class="shadow-sm shadow-lg" />;
<div class="shadow-none shadow-xl" />;

// ==========================================
// Opacity conflicts
// ==========================================
<div class="opacity-0 opacity-100" />;
<div class="opacity-50 opacity-75" />;

// ==========================================
// Transform conflicts
// ==========================================
<div class="rotate-45 rotate-90" />;
<div class="scale-50 scale-100" />;
<div class="translate-x-0 translate-x-4" />;
<div class="translate-y-0 translate-y-4" />;
<div class="skew-x-0 skew-x-12" />;
<div class="skew-y-0 skew-y-12" />;
<div class="origin-center origin-top" />;

// ==========================================
// Transition conflicts
// ==========================================
<div class="transition-none transition-all" />;
<div class="duration-75 duration-300" />;
<div class="ease-linear ease-in" />;
<div class="delay-75 delay-300" />;

// ==========================================
// Aspect ratio conflicts
// ==========================================
<div class="aspect-auto aspect-square" />;
<div class="aspect-video aspect-square" />;

// ==========================================
// Multiple conflicts in same attribute
// ==========================================
<div class="p-2 p-4 m-2 m-4" />;
<div class="flex flex-row hidden" />;
<div class="text-red-500 text-blue-500 bg-red-500 bg-blue-500" />;

// ==========================================
// With variants (should still detect conflicts)
// ==========================================
<div class="hover:p-2 hover:p-4" />;
<div class="md:flex md:grid" />;
<div class="dark:text-red-500 dark:text-blue-500" />;
<div class="focus:ring-2 focus:ring-4" />;
