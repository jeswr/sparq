import * as React from "react";
import { Slot } from "radix-ui";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// Pill badge (rounded-4xl, h-5) — the tier badges ("Live in your tab" / "Hosted" /
// "Walkthrough") reuse the success/warning/muted tokens.
//
// [OPUS-4.8] sq-vw3ax — bolder spec from the synced design-system card
// (sparq-design-system/components/badge.html). Two purely-additive upgrades, NO API break:
//   1. a focus-visible ring (ring-3 ring-ring/50, offset by the background) so the asChild
//      link/button case is keyboard-visible — the one consistency gap vs Button, which the
//      card flagged. Inert for non-focusable spans.
//   2. a `count` variant (min-w + centred padding + tabular figures) for numeric pills, so
//      single/double-digit counts stay aligned. Default variant unchanged.
//
// [SONNET-4.6] sq-ymr2e.14 — success text uses --success-on-tint (the WCAG AA accessible
// dark green) instead of --success for the text-on-tinted-bg context. The badge background
// keeps --success (15%-tinted light green). Light mode: --success-on-tint = oklch(0.46 0.12
// 155) → ~6.1:1 on the tinted bg (was 4.09:1 with --success). Dark mode: same as --success
// (already ~5.7:1). Defined in globals.css.
const badgeVariants = cva(
  "inline-flex items-center justify-center rounded-4xl h-5 px-2 text-xs font-medium w-fit whitespace-nowrap shrink-0 gap-1 [&>svg]:size-3 transition-shadow outline-none focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
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
