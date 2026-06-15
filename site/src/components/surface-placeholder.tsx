import Link from "next/link";
import { ArrowLeft, Construction } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { type Surface, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

// Honest "coming soon" placeholder for surfaces whose interactive page is not yet
// built. It states the planned execution tier up front so the page never overclaims.
export function SurfacePlaceholder({
  surface,
  flagship,
}: {
  surface: Surface;
  flagship?: boolean;
}) {
  const Icon = surface.icon;
  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" />
          Back to overview
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Icon className="size-5" />
          </span>
          <div>
            <h1 className="flex items-center gap-2 text-2xl font-semibold">
              {flagship && <span className="text-[var(--warning)]">★</span>}
              {surface.title}
            </h1>
            <p className="text-sm text-muted-foreground">{surface.blurb}</p>
          </div>
        </div>
        <Badge variant={TIER_VARIANT[surface.tier]}>
          Planned: {TIER_LABEL[surface.tier]}
        </Badge>
      </header>

      <Card>
        <CardHeader className="flex-row items-center gap-2 space-y-0">
          <Construction className="size-5 text-[var(--warning)]" />
          <CardTitle className="text-base">This demo is being built</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <CardDescription className="leading-relaxed">
            The site foundation — shell, theme, IA and the live SPARQL REPL — is
            live. This surface&rsquo;s interactive demo is tracked as a follow-up
            and will land here. When it does, it will run as labelled above, with
            real sparq output (never a mock).
          </CardDescription>
          <div className="flex flex-wrap gap-2">
            <Button asChild size="sm">
              <Link href="/try">Try the live REPL</Link>
            </Button>
            <Button asChild size="sm" variant="outline">
              <a
                href="https://github.com/jeswr/sparq"
                target="_blank"
                rel="noopener noreferrer"
              >
                View the source
              </a>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
