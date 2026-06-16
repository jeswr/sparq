"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { FLAGSHIPS, GROUPS, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

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

      {GROUPS.map((group) => (
        <Section key={group.label} label={group.label}>
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
  return (
    <Link
      href={href}
      onClick={onNavigate}
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
