"use client";

// [GPT-5.6] sq-j4woz — the CLI surface walkthrough. The site is static and has no
// backend, so this component REPLAYS real, verbatim sparq-cli captures rather than
// pretending to execute commands. See src/lib/cli.ts for the declared fixture and the
// stdout/stderr honesty contract; site/test/cli.test.mjs pins their exact serialization.

import * as React from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CLI_CAPTURES } from "@/lib/cli";

export function CliWalkthrough() {
  const [selected, setSelected] = React.useState(0);
  const capture = CLI_CAPTURES[selected];

  return (
    <section className="space-y-4">
      <div className="space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="text-xl font-semibold">sparq CLI walkthrough</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          Pick a subcommand to replay its recorded stdout and stderr over the
          declared four-triple fixture. Nothing runs in your browser.
        </p>
      </div>

      <div
        className="flex flex-wrap gap-2"
        role="group"
        aria-label="CLI subcommand"
      >
        {CLI_CAPTURES.map((item, index) => (
          <Button
            key={item.id}
            size="sm"
            variant={index === selected ? "default" : "outline"}
            aria-pressed={index === selected}
            onClick={() => setSelected(index)}
          >
            {item.label}
          </Button>
        ))}
      </div>

      <Card>
        <CardHeader className="space-y-1">
          <div className="flex items-center justify-between gap-2">
            <CardTitle className="font-mono text-base">
              {capture.label}
            </CardTitle>
            <Badge
              variant="outline"
              className="font-mono text-[10px] uppercase"
            >
              captured replay
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">{capture.description}</p>
        </CardHeader>
        <CardContent className="space-y-3">
          <div>
            <div className="mb-1 text-xs font-medium text-muted-foreground">
              command
            </div>
            <pre className="overflow-x-auto whitespace-pre-wrap rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
              <span className="select-none text-muted-foreground">$ </span>
              {capture.command}
            </pre>
          </div>
          {capture.stdout && (
            <div>
              <div className="mb-1 text-xs font-medium text-muted-foreground">
                stdout
              </div>
              <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
                {capture.stdout}
              </pre>
            </div>
          )}
          {capture.stderr && (
            <div>
              <div className="mb-1 text-xs font-medium text-muted-foreground">
                stderr
              </div>
              <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed text-muted-foreground">
                {capture.stderr}
              </pre>
            </div>
          )}
        </CardContent>
      </Card>
    </section>
  );
}
