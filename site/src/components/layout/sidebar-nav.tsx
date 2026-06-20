"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import {
  ABOUT_SURFACE,
  FLAGSHIPS,
  GROUPS,
  TIER_LABEL,
  TIER_VARIANT,
} from "@/data/surfaces";

// [OPUS-4.8] Hover/focus-intent prefetch of the in-browser ZK prover.
//
// The prover's cold start (the ~MB @noir-lang/noir_js + @aztec/bb.js WASM glue +
// the ACIR circuit fetch + the Barretenberg WASM instantiate) is deliberately
// route-scoped: it warms only after the user lands on a ZK page and the demo
// component mounts (prewarmProver in src/lib/zk-prover.ts, invoked from the
// zk-car-hire useEffect). This adds an EARLIER, intent-gated hook so the warm can
// begin while the pointer is still on the nav link — before navigation, before
// first paint of the ZK page — without ever pulling those assets onto routes that
// have nothing to do with ZK (/try, /papers, /benchmarks, …).
//
// It is purely an optimisation layered on top of the existing in-route warm-up:
//   * fire-and-forget — never awaited on any render / interaction path;
//   * idempotent — `prewarmProver` shares the single module-level promise, so a
//     hover, a later hover, and the in-route useEffect all reuse ONE cold start;
//   * one-shot per session — `zkPrefetchScheduled` guards repeat scheduling;
//   * idle-gated — scheduled via requestIdleCallback (setTimeout fallback) so it
//     never contends with first paint or an active interaction;
//   * graceful — failures are swallowed and re-arm the guard; the cold-start cache
//     self-resets, so the on-demand proving path stays correct even if this never
//     ran.
// The dynamic import keeps the bb.js / noir_js chunks out of every other route's
// bundle, preserving the route-scoping documented in zk-car-hire.tsx.
const ZK_PREFETCH_HREFS = new Set(["/showcase/zk-car-hire", "/surface/zk"]);

let zkPrefetchScheduled = false;

function prefetchZkProver() {
  if (typeof window === "undefined" || zkPrefetchScheduled) return;
  zkPrefetchScheduled = true;

  const warm = () => {
    // Dynamic import so the bb.js / noir_js chunks never enter non-ZK route bundles.
    void import("@/lib/zk-prover")
      .then((m) => m.prewarmProver())
      .catch(() => {
        // Optimisation only — if the warm import / instantiate fails, the in-route
        // useEffect and the proveAgeEligibility safety net still drive a fresh cold
        // start on demand. Re-arm the guard so a later hover can retry.
        zkPrefetchScheduled = false;
      });
  };

  const ric = (
    window as Window & {
      requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number;
    }
  ).requestIdleCallback;
  if (typeof ric === "function") {
    ric(warm, { timeout: 2000 });
  } else {
    // Safari / older browsers lack requestIdleCallback: defer past first paint.
    window.setTimeout(warm, 200);
  }
}

// Shared nav body used by BOTH the persistent desktop sidebar and the mobile Sheet.
// Every item is a real <a href> (Next Link) per the accessible-html-links rule.
export function SidebarNav({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();

  const isActive = (href: string) =>
    href === "/" ? pathname === "/" : pathname.startsWith(href);

  return (
    <nav aria-label="Feature surfaces" className="flex flex-col gap-5 text-sm">
      <NavLink href="/" active={pathname === "/"} onNavigate={onNavigate}>
        Overview
      </NavLink>

      <NavLink
        href="/benchmarks"
        active={isActive("/benchmarks")}
        onNavigate={onNavigate}
      >
        Benchmarks
      </NavLink>

      <NavLink
        href="/papers"
        active={isActive("/papers")}
        onNavigate={onNavigate}
      >
        Papers
      </NavLink>

      <Section label="Showcase">
        {FLAGSHIPS.map((f) => (
          <NavLink
            key={f.href}
            href={f.href}
            active={isActive(f.href)}
            onNavigate={onNavigate}
          >
            <span className="text-[var(--warning)]" aria-hidden>
              ★
            </span>{" "}
            {f.title}
          </NavLink>
        ))}
      </Section>

      {/* [OPUS-4.8] sq-vw3ax.2 — the 5 capability THEMES, derived from the single
          GROUPS source. Re-grouping there restructures this sidebar (and the Home grid,
          the future /capabilities, and Cmd-K) in one edit. */}
      {GROUPS.map((group) => (
        <Section key={group.id} label={group.label}>
          {group.surfaces.map((s) => (
            <NavLink
              key={s.href}
              href={s.href}
              active={isActive(s.href)}
              onNavigate={onNavigate}
            >
              <span className="flex-1 truncate">{s.title}</span>
              {(s.tier === "live" || s.tier === "live-new-wasm" || s.tier === "live-bbjs" || s.tier === "live-sim") && (
                <Badge
                  variant={TIER_VARIANT[s.tier]}
                  className="h-4 px-1.5 text-[10px]"
                  title={TIER_LABEL[s.tier]}
                >
                  live
                </Badge>
              )}
            </NavLink>
          ))}
        </Section>
      ))}

      {/* [OPUS-4.8] sq-vw3ax.2 — /about is a utility destination, not a capability
          theme, so it is kept out of GROUPS and rendered here so the link is retained. */}
      <NavLink
        href={ABOUT_SURFACE.href}
        active={isActive(ABOUT_SURFACE.href)}
        onNavigate={onNavigate}
      >
        {ABOUT_SURFACE.title}
      </NavLink>
    </nav>
  );
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-0.5">
      <h2 className="px-2 pb-1 text-xs font-medium uppercase tracking-wide text-muted-foreground/80">
        {label}
      </h2>
      {children}
    </div>
  );
}

function NavLink({
  href,
  active,
  onNavigate,
  children,
}: {
  href: string;
  active: boolean;
  onNavigate?: () => void;
  children: React.ReactNode;
}) {
  // [OPUS-4.8] On ZK nav entries only, hover / focus intent eagerly (idle-gated,
  // fire-and-forget) warms the in-browser prover before navigation. No-op on every
  // other link, so non-ZK routes never pull the prover WASM.
  const onZkIntent = ZK_PREFETCH_HREFS.has(href) ? prefetchZkProver : undefined;
  return (
    <Link
      href={href}
      onClick={onNavigate}
      onMouseEnter={onZkIntent}
      onFocus={onZkIntent}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex items-center gap-2 rounded-md px-2 py-1.5 transition-colors",
        active
          ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
          : "text-sidebar-foreground/80 hover:bg-sidebar-accent/60 hover:text-sidebar-foreground",
      )}
    >
      {children}
    </Link>
  );
}
