"use client";

// [OPUS-4.8] sq-vw3ax (bold /try redesign) — the local "panel" chrome the IDE workbench is
// built from (the mockup's `.panel` / `.panel-h` / `.panel-b`). A titled, elevated card with an
// uppercase mono header label, an optional teal-glow focus ring (the editor pane), and a header
// trailing slot. Local to the /try surface (NOT a shared primitive — the foundation owns those);
// it composes the merged tokens (--elevation-*, --teal-glow, border, muted) rather than inventing
// any new hue. Reused by the rail panels, the editor pane, and the results panes so the three
// columns read as one family.

import * as React from "react";

import { cn } from "@/lib/utils";

export function ReplPanel({
  title,
  icon: Icon,
  trailing,
  glow,
  bodyClassName,
  className,
  children,
  ...rest
}: {
  /** Uppercase mono header label. */
  title: React.ReactNode;
  /** Leading lucide icon (tinted teal in the header). */
  icon?: React.ComponentType<{ className?: string }>;
  /** Right-aligned header slot (a badge, a count, a view-toggle). */
  trailing?: React.ReactNode;
  /** Add the teal focus glow + brighter border (the central editor pane). */
  glow?: boolean;
  /** Override the body padding/scroll (e.g. a flush table that scrolls itself). */
  bodyClassName?: string;
  className?: string;
  children: React.ReactNode;
} & Omit<React.HTMLAttributes<HTMLElement>, "title">) {
  return (
    <section
      className={cn(
        "overflow-hidden rounded-lg border bg-card shadow-elevation-2",
        glow && "border-primary/30 shadow-elevation-glow",
        className,
      )}
      {...rest}
    >
      <div className="flex items-center gap-2 border-b bg-muted/25 px-3.5 py-3">
        <span className="flex items-center gap-2 text-[11.5px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
          {Icon && <Icon className="size-3.5 text-primary" />}
          {title}
        </span>
        {trailing && <span className="ml-auto flex items-center gap-1.5">{trailing}</span>}
      </div>
      <div className={cn("p-3.5", bodyClassName)}>{children}</div>
    </section>
  );
}
