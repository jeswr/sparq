import * as React from "react";

import { cn } from "@/lib/utils";

// [OPUS-4.8] sq-4296 — a minimal pulsing placeholder block (shadcn/ui convention), used as
// the layout-stable loading state for code-split client components (e.g. the lazily-loaded
// capability demos, src/components/capabilities/lazy-demo.tsx) so the page shell can render
// before the heavy chunk arrives. Purely presentational; no client state, no wasm. `bg-muted`
// + `animate-pulse` are
// both already in the design system's token set.
function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn("animate-pulse rounded-md bg-muted", className)}
      {...props}
    />
  );
}

export { Skeleton };
