"use client";

import * as React from "react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";

import { CONCEPTS, type Concept } from "./logo-concepts";

// [OPUS-4.8] sq-jnh9 — client gallery so each concept can be eyeballed at
// favicon → header sizes on BOTH a light and a dark field, regardless of the
// page theme. The swatches set their own fg/bg locally (not the page theme), so
// the user can judge contrast on both backgrounds at once. Pure presentation;
// no engine, no fetch — static-export safe.

const SIZES = [16, 32, 64, 256] as const;

/** A fixed light or dark field that locally pins fg/bg so the mark's
 * `currentColor` + teal token render against THAT field, not the page theme. */
function Field({
  tone,
  children,
}: {
  tone: "light" | "dark";
  children: React.ReactNode;
}) {
  // `.dark` flips the CSS variables (--primary etc.) exactly as the real theme
  // toggle does, so the teal token resolves to its dark-mode value on the dark
  // field — a faithful preview of both themes side by side.
  return (
    <div
      className={cn(
        "flex flex-1 flex-col items-center gap-3 rounded-lg p-4 ring-1 ring-foreground/10",
        tone === "light"
          ? "bg-white text-[#0f1c20]"
          : "dark bg-[#13181c] text-[#e8edf0]",
      )}
    >
      <span
        className={cn(
          "text-[0.65rem] font-medium uppercase tracking-wider",
          tone === "light" ? "text-black/40" : "text-white/40",
        )}
      >
        {tone}
      </span>
      {children}
    </div>
  );
}

/** The favicon-size strip: the mark at 16/32/64/256px, to spot legibility loss. */
function SizeStrip({ Mark }: { Mark: Concept["Mark"] }) {
  return (
    <div className="flex flex-wrap items-end gap-5">
      {SIZES.map((s) => (
        <div key={s} className="flex flex-col items-center gap-1">
          <Mark style={{ width: s, height: s }} aria-hidden />
          <span className="text-[0.65rem] text-current/50">{s}px</span>
        </div>
      ))}
    </div>
  );
}

function ConceptCard({ concept, index }: { concept: Concept; index: number }) {
  const { name, tagline, idea, Mark, Lockup } = concept;
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Badge variant="default">Concept {index + 1}</Badge>
          {Lockup && <Badge variant="success">full lockup</Badge>}
        </div>
        <CardTitle className="text-lg">
          {name} <span className="text-muted-foreground">· {tagline}</span>
        </CardTitle>
        <CardDescription className="measure">{idea}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* mark on both fields, at a comfortable preview size */}
        <div className="flex flex-col gap-3 sm:flex-row">
          <Field tone="light">
            <Mark style={{ width: 96, height: 96 }} aria-hidden />
          </Field>
          <Field tone="dark">
            <Mark style={{ width: 96, height: 96 }} aria-hidden />
          </Field>
        </div>

        {/* favicon size check, on the current page field */}
        <div className="space-y-2">
          <p className="text-xs font-medium text-muted-foreground">
            Favicon size check (16 / 32 / 64 / 256px)
          </p>
          <SizeStrip Mark={Mark} />
        </div>

        {/* lockup, if this concept has one, on both fields */}
        {Lockup && (
          <div className="space-y-2">
            <p className="text-xs font-medium text-muted-foreground">
              Full lockup (mark + wordmark)
            </p>
            <div className="flex flex-col gap-3 sm:flex-row">
              <Field tone="light">
                <Lockup style={{ height: 44, width: "auto" }} aria-hidden />
              </Field>
              <Field tone="dark">
                <Lockup style={{ height: 44, width: "auto" }} aria-hidden />
              </Field>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function LogoGallery() {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      {CONCEPTS.map((c, i) => (
        <ConceptCard key={c.id} concept={c} index={i} />
      ))}
    </div>
  );
}
