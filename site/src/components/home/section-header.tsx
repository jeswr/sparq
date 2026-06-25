// [OPUS-4.8] sq-vw3ax.5 — the bold homepage redesign. Home-only presentational
// section header: a mono `.kicker` over a display-2 / section title, with an optional
// right-aligned note — the per-section rhythm the approved mockup
// (sparq-design-system/proposals/web-home.html, `.repl-head` / `.sec-kicker`) establishes.
// Pure presentation, no data; kept home-scoped so it never collides with shared primitives.

import * as React from "react";

import { cn } from "@/lib/utils";

export function SectionHeader({
  kicker,
  title,
  note,
  className,
  titleClassName,
}: {
  kicker: string;
  title: React.ReactNode;
  note?: React.ReactNode;
  className?: string;
  titleClassName?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between",
        className,
      )}
    >
      <div className="space-y-1.5">
        <div className="kicker">{kicker}</div>
        <h2
          className={cn(
            "text-2xl font-semibold tracking-tight sm:text-3xl",
            titleClassName,
          )}
        >
          {title}
        </h2>
      </div>
      {note ? (
        <p className="max-w-[46ch] text-sm text-muted-foreground">{note}</p>
      ) : null}
    </div>
  );
}
