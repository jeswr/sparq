import * as React from "react";

import { cn } from "@/lib/utils";

// The sparq mark, theme-aware: the graph nodes/edges/wordmark use `currentColor`
// (so they follow light/dark via the surrounding text colour) and the spark keeps
// its amber accent. Mirrors assets/logo.svg but swaps the prefers-color-scheme rule
// for currentColor so the class-based theme toggle drives it.
export function Logo({ className, ...props }: React.ComponentProps<"svg">) {
  return (
    <svg
      viewBox="0 0 360 96"
      role="img"
      aria-label="sparq"
      className={cn("text-foreground", className)}
      {...props}
    >
      <g transform="translate(20,24)" fill="currentColor">
        <line x1="12" y1="36" x2="44" y2="12" stroke="currentColor" strokeWidth={5} />
        <line x1="44" y1="12" x2="76" y2="36" stroke="currentColor" strokeWidth={5} />
        <circle cx="12" cy="36" r="9" />
        <circle cx="44" cy="12" r="9" />
        <circle cx="76" cy="36" r="9" />
        <path d="M76 -2 l-8 18 h7 l-5 16 14 -22 h-7 z" fill="#f5a623" />
      </g>
      <text
        x="120"
        y="64"
        fill="currentColor"
        fontFamily="ui-monospace,'JetBrains Mono','SF Mono',Menlo,monospace"
        fontSize="56"
        fontWeight="700"
        letterSpacing="-1"
      >
        sparq
      </text>
    </svg>
  );
}
