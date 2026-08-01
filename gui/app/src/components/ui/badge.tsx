import * as React from "react";
import { Slot } from "radix-ui";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// [OPUS-4.8] sq-ixc3.9 — operational Badge, ported from the site design system so the
// honesty-tier dots + status pills read consistently with the rest of the engine's surfaces.
//
// [FABLE-5] sq-7i0xh — re-synced with the site's badge.tsx (max shared components, GH #836):
//   1. success text uses --success-on-tint (the site's sq-ymr2e.14 WCAG AA fix — --success on
//      the 15%-tinted bg was ~4.09:1, below AA; the on-tint token reads ~6.1:1);
//   2. the sq-vw3ax focus-visible ring so an asChild link/button badge is keyboard-visible
//      (inert for plain spans);
//   3. the sq-vw3ax `count` variant (min-w + tabular figures) for aligned numeric pills.
// Intentional GUI delta kept: rounded-full pill (vs the site's rounded-4xl — same look at h-5,
// no dependency on the site's radius scale).
const badgeVariants = cva(
  "inline-flex items-center justify-center rounded-full h-5 px-2 text-xs font-medium w-fit whitespace-nowrap shrink-0 gap-1 [&>svg]:size-3 transition-shadow outline-none focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
  {
    variants: {
      variant: {
        default: "bg-primary/10 text-primary",
        secondary: "bg-secondary text-secondary-foreground",
        outline: "text-foreground ring-1 ring-foreground/15",
        success:
          "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success-on-tint)]",
        warning:
          "bg-[color-mix(in_oklch,var(--warning)_18%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
        muted: "bg-muted text-muted-foreground",
      },
      count: {
        true: "min-w-5 px-1.5 tabular-nums",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

function Badge({
  className,
  variant,
  count,
  asChild = false,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot.Root : "span";
  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant, count }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
