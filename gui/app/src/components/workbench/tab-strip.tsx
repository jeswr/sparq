"use client";

// [OPUS-4.8] sq-ixc3.9 — the IDE-style TAB STRIP (research/gui-design.md §A.2). Each open tool
// is a tab; the active tab's panel fills the full-bleed work area below. The last tab is not
// closable (the work area never empties).

import { X } from "lucide-react";

import { cn } from "@/lib/utils";
import { toolById, TIER_META } from "@/data/tools";
import type { OpenTab } from "@/components/workbench/workbench";

interface TabStripProps {
  tabs: OpenTab[];
  activeId: string;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
}

export function TabStrip({ tabs, activeId, onSelect, onClose }: TabStripProps) {
  return (
    // [OPUS-4.8] sq-vw3ax (#820 redesign) — IDE tab strip with a subtle card gradient + a
    // teal-glow underline on the active tab (the proposal's accent-lit spine).
    <div
      role="tablist"
      className="flex h-9 shrink-0 items-stretch overflow-x-auto border-b bg-gradient-to-b from-card to-[color-mix(in_oklch,var(--card)_70%,var(--background))]"
    >
      {tabs.map((tab) => {
        const tool = toolById(tab.id);
        const tier = tool ? TIER_META[tool.tier] : null;
        const isActive = tab.id === activeId;
        const Icon = tool?.icon;
        return (
          <div
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            className={cn(
              "group relative flex cursor-pointer items-center gap-1.5 border-r px-3 text-xs",
              isActive
                ? "bg-background font-medium text-foreground after:absolute after:inset-x-2.5 after:-bottom-px after:h-0.5 after:rounded-full after:bg-gradient-to-r after:from-primary after:to-transparent after:shadow-[0_0_8px_var(--primary)]"
                : "text-muted-foreground hover:bg-accent/40",
            )}
            onClick={() => onSelect(tab.id)}
          >
            {Icon ? <Icon className="size-3.5" /> : null}
            <span>{tab.label}</span>
            {tier ? (
              <span className={cn("size-1.5 rounded-full", tier.dot)} aria-hidden />
            ) : null}
            {tabs.length > 1 ? (
              <button
                className="ml-1 rounded p-0.5 opacity-0 hover:bg-accent group-hover:opacity-100"
                aria-label={`Close ${tab.label}`}
                onClick={(e) => {
                  e.stopPropagation();
                  onClose(tab.id);
                }}
              >
                <X className="size-3" />
              </button>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
