import * as React from "react";
import { cn } from "@/lib/utils";

// Card uses `ring-1 ring-foreground/10` instead of a border + rounded-xl
// (solid-pod-manager convention).
//
// [OPUS-4.8] sq-vw3ax — bolder spec from the synced card (components/card.html): the flat
// `shadow-sm` becomes the teal-tinted `--elevation-1` token so depth reads consistently in
// BOTH themes (a soft drop in light, a hairline inset on the dark ground). Same ring, same
// radius, same API — every existing call site is unchanged. Pages that want the hover-lift
// treatment from the mockups add it via className (e.g. `hover:shadow-elevation-2`).
function Card({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card"
      className={cn(
        "bg-card text-card-foreground flex flex-col gap-4 rounded-xl py-5 ring-1 ring-foreground/10 shadow-[var(--elevation-1)]",
        className,
      )}
      {...props}
    />
  );
}

function CardHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-header"
      className={cn("flex flex-col gap-1.5 px-5", className)}
      {...props}
    />
  );
}

function CardTitle({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-title"
      className={cn("font-semibold leading-none tracking-tight", className)}
      {...props}
    />
  );
}

function CardDescription({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-description"
      className={cn("text-muted-foreground text-sm", className)}
      {...props}
    />
  );
}

function CardContent({ className, ...props }: React.ComponentProps<"div">) {
  return <div data-slot="card-content" className={cn("px-5", className)} {...props} />;
}

function CardFooter({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-footer"
      className={cn("flex items-center px-5 py-3 border-t bg-muted/50 rounded-b-xl -mb-5", className)}
      {...props}
    />
  );
}

export { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter };
