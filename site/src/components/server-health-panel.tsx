"use client";

// [OPUS-4.8] sq-he72 — GUI Phase 2 item 8: the server HEALTH / CAPABILITIES panel.
//
// In endpoint mode this panel reads the connected sparq-server's OPERATIONAL surface — its
// `/health` liveness, its Prometheus `/metrics`, and its opt-in VoID / SPARQL Service
// Description — and renders them as a readable "server health / capabilities" view.
//
// It REUSES the sq-2mke endpoint-mode connection + bearer posture (the SAME `EndpointConfig`
// the Connect panel owns), never reinventing it: the bearer token is sent only in the
// `Authorization: Bearer` header by the shared `@sparq/client` `fetchServerHealth`, the same
// channel the server's read gate (`--auth-token-read`) validates. This view never logs the
// token and never bypasses a server gate.
//
// HONESTY is load-bearing. The `/metrics` and VoID/SD surfaces are opt-in and OFF by default;
// a disabled feature answers `404`, which the client reports as a `"not-exposed"` outcome —
// rendered here as an explicit "not exposed by this server" note, NEVER a fabricated metric or
// capability. Metric VALUES are whatever the server reports at scrape time; they are server
// operational counters, NOT a benchmark claim (this repo's work box is non-canonical).
//
// All wire-protocol + parsing logic lives in the framework-agnostic `@sparq/client`
// server-health module; this component is just the React host that draws the metric tables,
// the capabilities lists, and the honest fetch lifecycle.

import * as React from "react";
import {
  HeartPulse,
  Gauge,
  Boxes,
  RefreshCw,
  Loader2,
  CircleCheck,
  CircleSlash,
  ShieldAlert,
  Network,
  Database,
  ServerCog,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  type EndpointConfig,
  type FetchOutcome,
  type MetricFamily,
  type ServerHealth,
  type ServiceDescriptionSummary,
  type VoidSummary,
  connectionSafetyWarnings,
  fetchServerHealth,
  formatMetricLabels,
  hasBlockingWarning,
  shortenIri,
} from "@sparq/client";

/** The panel's fetch lifecycle, surfaced honestly. */
type LoadState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "loaded"; health: ServerHealth }
  | { kind: "error"; message: string };

export interface ServerHealthPanelProps {
  /** The endpoint connection (URL + optional bearer token), shared with the REPL. */
  config: EndpointConfig;
  /** Whether endpoint mode is active — health is only read from a real server. */
  active: boolean;
}

/**
 * [OPUS-4.8] sq-he72 — the server health / capabilities panel. Renders a Refresh control, the
 * honest connection-safety findings (reused from sq-2mke), the server liveness, the parsed
 * Prometheus metrics grouped by family, and the VoID + Service-Description capabilities — each
 * gracefully showing "not exposed" when the server has the opt-in feature off (a `404`).
 */
export function ServerHealthPanel({ config, active }: ServerHealthPanelProps) {
  const [state, setState] = React.useState<LoadState>({ kind: "idle" });

  // Reuse the SAME honest classifier the Connect panel uses; a hard block (invalid URL /
  // mixed content) disables Refresh exactly as it disables turning endpoint mode on.
  const warnings = React.useMemo(() => connectionSafetyWarnings(config), [config]);
  const blocked = hasBlockingWarning(warnings);

  const refresh = React.useCallback(async () => {
    setState({ kind: "loading" });
    try {
      const health = await fetchServerHealth(config);
      setState({ kind: "loaded", health });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setState({ kind: "error", message });
    }
  }, [config]);

  // Reset the snapshot whenever endpoint mode is switched off, or the connection changes — a
  // stale snapshot from a previous server must never linger.
  React.useEffect(() => {
    setState({ kind: "idle" });
  }, [active, config.url, config.token]);

  if (!active) {
    return (
      <div className="space-y-2 rounded-lg border bg-muted/30 p-3">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <HeartPulse className="size-3.5" />
          Server health &amp; capabilities
          <span className="font-normal text-muted-foreground/80">
            — the connected server&apos;s metrics + VoID / Service Description
          </span>
        </div>
        <p className="text-[11.5px] leading-relaxed text-muted-foreground">
          Switch on <span className="font-medium">Endpoint mode</span> above and connect to a
          running sparq-server to read its <code className="font-mono">/health</code>, its
          Prometheus <code className="font-mono">/metrics</code>, and — when the operator
          enabled them — the VoID dataset description and SPARQL Service Description that
          advertise its capabilities.
        </p>
      </div>
    );
  }

  const loading = state.kind === "loading";

  return (
    <div className="space-y-3 rounded-lg border bg-muted/30 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <HeartPulse
            className={cn(
              "size-3.5",
              state.kind === "loaded" &&
                state.health.health.status === "ok" &&
                "text-primary",
            )}
          />
          Server health &amp; capabilities
          <span className="font-normal text-muted-foreground/80">
            — read from the connected server
          </span>
        </div>
        {state.kind === "loaded" ? (
          <LivenessBadge outcome={state.health.health} />
        ) : null}
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          disabled={blocked || loading}
          title={
            blocked
              ? "Fix the endpoint URL / transport warning before reading server health"
              : "Read the server's /health, /metrics and VoID / Service Description"
          }
        >
          {loading ? (
            <Loader2 className="size-3.5 animate-spin" />
          ) : (
            <RefreshCw className="size-3.5" />
          )}
          {state.kind === "loaded" ? "Refresh" : "Read server health"}
        </Button>
        <StatusLine state={state} />
      </div>

      <SafetyWarningList warnings={warnings} />

      {state.kind === "error" ? (
        <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-[12px] text-destructive">
          {state.message}
        </pre>
      ) : null}

      {state.kind === "loaded" ? (
        <div className="space-y-3" data-testid="server-health">
          <MetricsSection outcome={state.health.metrics} />
          <CapabilitiesSection
            voidOutcome={state.health.voidDescriptor}
            sdOutcome={state.health.serviceDescription}
          />
        </div>
      ) : null}

      <p className="text-[11px] leading-relaxed text-muted-foreground">
        Metric values are the server&apos;s own operational counters/gauges, read at scrape
        time — not a benchmark claim. The token is sent only in the{" "}
        <code className="font-mono">Authorization: Bearer</code> header; the VoID dataset
        description and Service Description are <span className="font-medium">opt-in</span> (the{" "}
        <code className="font-mono">federation-descriptors</code> feature) and shown as{" "}
        <span className="font-medium">not exposed</span> when the operator left them off.
      </p>
    </div>
  );
}

// [OPUS-4.8] sq-he72 — the liveness pill from the /health outcome.
function LivenessBadge({ outcome }: { outcome: FetchOutcome<{ body: string }> }) {
  if (outcome.status === "ok") {
    return (
      <Badge variant="success" aria-live="polite">
        <CircleCheck className="size-3" /> Live
      </Badge>
    );
  }
  if (outcome.status === "unauthorized") {
    return (
      <Badge variant="warning" aria-live="polite">
        <ShieldAlert className="size-3" /> Auth required
      </Badge>
    );
  }
  return (
    <Badge variant="warning" aria-live="polite">
      <CircleSlash className="size-3" /> Unreachable
    </Badge>
  );
}

function StatusLine({ state }: { state: LoadState }) {
  return (
    <p aria-live="polite" className="text-xs text-muted-foreground">
      {state.kind === "loading" && "Reading the server's operational surface…"}
      {state.kind === "loaded" &&
        state.health.health.status === "ok" &&
        "Server responded."}
      {state.kind === "loaded" &&
        state.health.health.status !== "ok" &&
        "Read complete — see the findings below."}
      {state.kind === "error" && (
        <span className="text-destructive">Could not read server health.</span>
      )}
    </p>
  );
}

// [OPUS-4.8] sq-he72 — render the SAME classified connection-safety findings the Connect
// panel uses (this panel rides the same transport as a query, so the same posture applies).
function SafetyWarningList({
  warnings,
}: {
  warnings: ReturnType<typeof connectionSafetyWarnings>;
}) {
  if (warnings.length === 0) return null;
  return (
    <ul className="space-y-1.5">
      {warnings.map((w) => (
        <li
          key={w.code}
          data-safety-code={w.code}
          data-safety-level={w.level}
          className={cn(
            "flex items-start gap-2 rounded-md border p-2 text-[11.5px] leading-relaxed",
            w.level === "error" &&
              "border-destructive/30 bg-destructive/5 text-destructive",
            w.level === "warning" &&
              "border-[color-mix(in_oklch,var(--warning)_35%,transparent)] bg-[color-mix(in_oklch,var(--warning)_10%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
            w.level === "info" && "border-border bg-muted/40 text-muted-foreground",
          )}
        >
          <span>{w.message}</span>
        </li>
      ))}
    </ul>
  );
}

// [OPUS-4.8] sq-he72 — a small "not exposed / unauthorized / error" note reused by every
// outcome-tagged section, so a disabled opt-in feature reads honestly everywhere.
function OutcomeNote({
  outcome,
  notExposedLabel,
}: {
  outcome: { status: "not-exposed" } | { status: "unauthorized"; message: string } | { status: "error"; message: string };
  notExposedLabel: string;
}) {
  if (outcome.status === "not-exposed") {
    return (
      <p className="flex items-start gap-2 rounded-md border bg-muted/40 p-2 text-[11.5px] leading-relaxed text-muted-foreground">
        <CircleSlash className="mt-0.5 size-3.5 shrink-0" />
        <span>{notExposedLabel}</span>
      </p>
    );
  }
  if (outcome.status === "unauthorized") {
    return (
      <p className="flex items-start gap-2 rounded-md border border-[color-mix(in_oklch,var(--warning)_35%,transparent)] bg-[color-mix(in_oklch,var(--warning)_10%,transparent)] p-2 text-[11.5px] leading-relaxed text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]">
        <ShieldAlert className="mt-0.5 size-3.5 shrink-0" />
        <span>{outcome.message}</span>
      </p>
    );
  }
  return (
    <p className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-2 text-[11.5px] leading-relaxed text-destructive">
      <ShieldAlert className="mt-0.5 size-3.5 shrink-0" />
      <span>{outcome.message}</span>
    </p>
  );
}

// [OPUS-4.8] sq-he72 — the Prometheus metrics section: one block per metric family, each with
// its help text, type, and a value table over its samples (with the label set rendered
// inline). No charting library — a formatted table is the honest, dependency-free shape.
function MetricsSection({ outcome }: { outcome: FetchOutcome<{ families: MetricFamily[] }> }) {
  return (
    <section className="space-y-2">
      <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        <Gauge className="size-3.5" />
        Prometheus metrics
        <span className="font-normal text-muted-foreground/70">
          (<code className="font-mono">/metrics</code>)
        </span>
      </div>
      {outcome.status !== "ok" ? (
        <OutcomeNote
          outcome={outcome}
          notExposedLabel="This server does not expose /metrics."
        />
      ) : outcome.data.families.length === 0 ? (
        <p className="rounded-md border bg-muted/40 p-2 text-[11.5px] text-muted-foreground">
          The server returned no metric families.
        </p>
      ) : (
        <ul className="space-y-2">
          {outcome.data.families.map((f) => (
            <MetricFamilyBlock key={f.name} family={f} />
          ))}
        </ul>
      )}
    </section>
  );
}

function MetricFamilyBlock({ family }: { family: MetricFamily }) {
  return (
    <li
      data-metric-family={family.name}
      className="rounded-md border bg-background/60 p-2.5"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <code className="font-mono text-[12px] font-medium">{family.name}</code>
        <Badge variant="muted" className="text-[10px]">
          {family.type}
        </Badge>
      </div>
      {family.help ? (
        <p className="mt-0.5 text-[11px] leading-relaxed text-muted-foreground">
          {family.help}
        </p>
      ) : null}
      <table className="mt-1.5 w-full text-left text-[11.5px]">
        <tbody>
          {family.samples.map((s, i) => {
            const labels = formatMetricLabels(s.labels);
            return (
              <tr key={i} className="border-t border-border/60">
                <td className="py-1 pr-3 font-mono text-muted-foreground">
                  {labels === "" ? (
                    <span className="text-muted-foreground/60">(no labels)</span>
                  ) : (
                    labels
                  )}
                </td>
                <td className="py-1 text-right font-mono tabular">
                  {Number.isFinite(s.value) ? s.value : String(s.value)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </li>
  );
}

// [OPUS-4.8] sq-he72 — the capabilities section: the VoID dataset summary + the SPARQL
// Service Description, each shown as "not exposed" when the opt-in feature is off.
function CapabilitiesSection({
  voidOutcome,
  sdOutcome,
}: {
  voidOutcome: FetchOutcome<VoidSummary>;
  sdOutcome: FetchOutcome<ServiceDescriptionSummary>;
}) {
  return (
    <section className="space-y-2">
      <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        <ServerCog className="size-3.5" />
        Capabilities
        <span className="font-normal text-muted-foreground/70">
          (VoID + SPARQL Service Description — opt-in)
        </span>
      </div>

      <div className="space-y-2 rounded-md border bg-background/60 p-2.5">
        <div className="flex items-center gap-1.5 text-[11.5px] font-medium">
          <Database className="size-3.5 text-muted-foreground" />
          VoID dataset (<code className="font-mono">/.well-known/void</code>)
        </div>
        {voidOutcome.status !== "ok" ? (
          <OutcomeNote
            outcome={voidOutcome}
            notExposedLabel="The VoID dataset description is not exposed (the federation-descriptors feature is off)."
          />
        ) : (
          <VoidView summary={voidOutcome.data} />
        )}
      </div>

      <div className="space-y-2 rounded-md border bg-background/60 p-2.5">
        <div className="flex items-center gap-1.5 text-[11.5px] font-medium">
          <Network className="size-3.5 text-muted-foreground" />
          Service Description (<code className="font-mono">GET /sparql</code>, no query)
        </div>
        {sdOutcome.status !== "ok" ? (
          <OutcomeNote
            outcome={sdOutcome}
            notExposedLabel="The SPARQL Service Description is not exposed (the federation-descriptors feature is off)."
          />
        ) : (
          <ServiceDescriptionView summary={sdOutcome.data} />
        )}
      </div>
    </section>
  );
}

// [OPUS-4.8] sq-he72 — the VoID dataset counts as a small fact list. A `null` count means the
// document did not carry it (honest "—"), never a zero we invented.
function VoidView({ summary }: { summary: VoidSummary }) {
  const rows: { label: string; value: number | null }[] = [
    { label: "Triples", value: summary.triples },
    { label: "Entities (typed subjects)", value: summary.entities },
    { label: "Distinct subjects", value: summary.distinctSubjects },
    { label: "Classes", value: summary.classes },
    { label: "Properties", value: summary.properties },
  ];
  return (
    <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-[11.5px] sm:grid-cols-3">
      {rows.map((r) => (
        <div key={r.label} className="flex flex-col">
          <dt className="text-muted-foreground">{r.label}</dt>
          <dd className="font-mono tabular font-medium">
            {r.value === null ? <span className="text-muted-foreground/60">—</span> : r.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}

// [OPUS-4.8] sq-he72 — the Service Description capabilities. The well-known sd:/void: IRIs are
// abbreviated for display; everything is rendered verbatim from the document (no fiction).
function ServiceDescriptionView({ summary }: { summary: ServiceDescriptionSummary }) {
  return (
    <div className="space-y-2 text-[11.5px]">
      {summary.endpoint ? (
        <FactRow label="Endpoint">
          <code className="font-mono">{summary.endpoint}</code>
        </FactRow>
      ) : null}
      <IriListRow label="Supported languages" iris={summary.supportedLanguages} />
      <IriListRow label="Features" iris={summary.features} />
      <IriListRow label="Result formats" iris={summary.resultFormats} />
      <IriListRow label="Input formats" iris={summary.inputFormats} />
      <IriListRow label="Extension functions" iris={summary.extensionFunctions} />
      {summary.namedGraphs.length > 0 ? (
        <FactRow label="Named graphs">
          <ul className="space-y-0.5">
            {summary.namedGraphs.map((ng) => (
              <li key={ng.name} className="flex items-baseline gap-2">
                <Boxes className="size-3 shrink-0 text-muted-foreground" />
                <code className="font-mono break-all">{ng.name}</code>
                {ng.triples !== null ? (
                  <span className="text-muted-foreground">· {ng.triples} triples</span>
                ) : null}
              </li>
            ))}
          </ul>
        </FactRow>
      ) : null}
    </div>
  );
}

function FactRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-muted-foreground">{label}</span>
      <div className="font-medium">{children}</div>
    </div>
  );
}

// One labelled list of abbreviated IRIs, or an honest "none advertised" when the document
// carried no values for it.
function IriListRow({ label, iris }: { label: string; iris: string[] }) {
  return (
    <FactRow label={label}>
      {iris.length === 0 ? (
        <span className="font-normal text-muted-foreground/60">none advertised</span>
      ) : (
        <div className="flex flex-wrap gap-1">
          {iris.map((iri) => (
            <Badge key={iri} variant="muted" className="font-mono text-[10px]" title={iri}>
              {shortenIri(iri)}
            </Badge>
          ))}
        </div>
      )}
    </FactRow>
  );
}
