"use client";

// [OPUS-4.8] sq-vjn4 — the left sidebar inside /benchmarks: one entry per benchmark TYPE
// (capability family). Active-link highlighting mirrors the main SidebarNav. Each entry
// shows the family's metric count (0 -> "soon" badge, honest coverage).
import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

export interface TypeNavItem {
  key: string;
  title: string;
  count: number;
}

export function TypeSidebar({ items }: { items: TypeNavItem[] }) {
  const pathname = usePathname();
  const base = "/benchmarks";
  return (
    <nav aria-label="Benchmark types" className="flex flex-col gap-0.5 text-sm">
      <NavLink href={base} active={pathname === base || pathname === `${base}/`}>
        Overview
      </NavLink>
      {items.map((it) => {
        const href = `${base}/${it.key}`;
        return (
          <NavLink key={it.key} href={href} active={pathname.startsWith(href)}>
            <span className="flex-1 truncate">{it.title}</span>
            {it.count === 0 ? (
              <Badge variant="muted" className="h-4 px-1.5 text-[10px]">
                soon
              </Badge>
            ) : (
              <span className="text-xs text-muted-foreground">{it.count}</span>
            )}
          </NavLink>
        );
      })}
    </nav>
  );
}

function NavLink({
  href,
  active,
  children,
}: {
  href: string;
  active: boolean;
  children: React.ReactNode;
}) {
  return (
    <Link
      href={href}
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
