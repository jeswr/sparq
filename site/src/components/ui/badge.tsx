import * as React from "react";
import { Slot } from "radix-ui";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// Pill badge (rounded-4xl, h-5) — the tier badges ("Live in your tab" / "Hosted" /
// "Walkthrough") reuse the success/warning/muted tokens.
const badgeVariants = cva(
  "inline-flex items-center justify-center rounded-4xl h-5 px-2 text-xs font-medium w-fit whitespace-nowrap shrink-0 gap-1 [&>svg]:size-3",
  {
    variants: {
      variant: {
        default: "bg-primary/10 text-primary",
        secondary: "bg-secondary text-secondary-foreground",
        outline: "text-foreground ring-1 ring-foreground/15",
        success:
          "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success)]",
        warning:
          "bg-[color-mix(in_oklch,var(--warning)_18%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
        muted: "bg-muted text-muted-foreground",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

function Badge({
  className,
  variant,
  asChild = false,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot.Root : "span";
  return (
    <Comp data-slot="badge" className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
