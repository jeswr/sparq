import * as React from "react";
import { Slot } from "radix-ui";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// [OPUS-4.8] sq-ixc3.9 — operational Button. Same variants as the site's design system
// (reuse, don't reinvent) so the GUI reads as a sibling tool; sizes lean compact for the
// dense workbench chrome.
//
// [FABLE-5] sq-7i0xh — re-synced the sq-vw3ax focus ring from the site's button.tsx (ring-3 +
// 2px background offset — the crisper keyboard halo from the synced design-system card).
// Intentional GUI deltas kept, per the operational-vs-marketing split (GH #836): compact
// sizes + rounded-md + the extra icon-sm size for the dense workbench chrome, and NO
// default-variant elevation glow — in the workbench the glow is an opt-in brand accent
// (`.sq-glow-btn`, e.g. the top-bar Import action), not every primary button.
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all outline-none focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background active:translate-y-px disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive:
          "bg-destructive/10 text-destructive hover:bg-destructive/20",
        outline:
          "border bg-background hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-8 px-3 py-2 has-[>svg]:px-2.5",
        sm: "h-7 rounded-md px-2.5 text-xs",
        lg: "h-9 rounded-md px-5",
        icon: "size-8",
        "icon-sm": "size-7",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  },
);

function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot.Root : "button";
  return (
    <Comp
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
