"use client";

// [OPUS-4.8] sq-gl3cf / sq-ixc3 — the /download page client body.
//
// Why a client component: the page progressively ENHANCES with client-side OS
// detection (navigator.userAgent / userAgentData) to highlight the platform the
// visitor is most likely on. It is pure progressive enhancement — every platform
// is ALWAYS listed and linkable; detection only adds a "we think this is you"
// affordance and reorders the highlighted card. With JS off (or before hydration)
// the full unfiltered list renders, so nothing depends on the detection.
//
// HONESTY: there are no published GitHub Releases yet (gh release list is empty as
// of this commit), and the desktop installers are UNSIGNED "developer build"
// bundles (signing/notarization is the separate needs:user bead sq-v286.8). The
// copy below says so plainly — no overclaiming. Asset links point at the *latest
// release* page rather than per-asset deep links, because the release CI
// (release.yml, sq-8n1c via #808) names each asset
// `sparq-gui-<tag>-<label>-<tauri-name>` where `<tag>` is the dynamic version, so a
// static per-asset URL would be a dead link until a tag ships. Once releases exist,
// the "latest release" page resolves to the real assets.

import * as React from "react";
import Link from "next/link";
import {
  Apple,
  Cpu,
  Download,
  ExternalLink,
  MonitorSmartphone,
  Server,
  Terminal,
  TriangleAlert,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

const REPO_URL = "https://github.com/jeswr/sparq";
const LATEST_RELEASE_URL = `${REPO_URL}/releases/latest`;
const RELEASES_URL = `${REPO_URL}/releases`;

// The platforms the release matrix actually produces a desktop GUI installer for
// (release.yml `gui-bundle` job, sq-8n1c via #808). win-arm64 is deliberately
// absent (the Tauri bundler can't reliably cross-bundle an arm64 Windows installer
// from the x64 host) — an honest gap, not a silent drop.
type OsKey = "macos-arm" | "macos-intel" | "windows" | "linux";

interface Platform {
  /** Detection bucket(s) this card satisfies. */
  os: OsKey;
  label: string;
  /** The installer file kind shown in the release assets. */
  ext: string;
  /** The release-asset label this maps to (release.yml matrix `label`). */
  assetLabel: string;
  icon: React.ReactNode;
  /** Per-OS unsigned-bundle bypass instructions. */
  bypass: React.ReactNode;
}

const PLATFORMS: Platform[] = [
  {
    os: "macos-arm",
    label: "macOS · Apple Silicon",
    ext: ".dmg",
    assetLabel: "arm64-darwin",
    icon: <Apple className="size-5" aria-hidden />,
    bypass: (
      <>
        Gatekeeper quarantines unsigned apps. After opening the{" "}
        <code className="font-mono text-xs">.dmg</code> and dragging sparq to
        Applications, <strong>right-click the app → Open</strong> (don&rsquo;t
        double-click) and confirm once. If it still refuses, clear the quarantine
        flag from a terminal:
        <pre className="mt-2 overflow-x-auto rounded-md bg-muted px-3 py-2 text-xs">
          <code>xattr -dr com.apple.quarantine /Applications/sparq.app</code>
        </pre>
      </>
    ),
  },
  {
    os: "macos-intel",
    label: "macOS · Intel",
    ext: ".dmg",
    assetLabel: "x64-darwin",
    icon: <Apple className="size-5" aria-hidden />,
    bypass: (
      <>
        Same as Apple Silicon — <strong>right-click the app → Open</strong> on
        first launch, or clear the quarantine flag:
        <pre className="mt-2 overflow-x-auto rounded-md bg-muted px-3 py-2 text-xs">
          <code>xattr -dr com.apple.quarantine /Applications/sparq.app</code>
        </pre>
        Pick this build only on an Intel Mac; Apple Silicon Macs should use the
        arm64 build above.
      </>
    ),
  },
  {
    os: "windows",
    label: "Windows · x64",
    ext: ".msi",
    assetLabel: "win-x64",
    icon: <MonitorSmartphone className="size-5" aria-hidden />,
    bypass: (
      <>
        The installer isn&rsquo;t Authenticode-signed, so Microsoft SmartScreen
        shows a blue &ldquo;Windows protected your PC&rdquo; dialog. Click{" "}
        <strong>More info → Run anyway</strong> to proceed. There is no Windows
        on Arm installer yet (the bundler can&rsquo;t cross-build it from the x64
        runner).
      </>
    ),
  },
  {
    os: "linux",
    label: "Linux · x86-64 / arm64",
    ext: ".AppImage",
    assetLabel: "x64-linux",
    icon: <Cpu className="size-5" aria-hidden />,
    bypass: (
      <>
        The <code className="font-mono text-xs">.AppImage</code> is a portable,
        self-contained binary — no install needed. Mark it executable, then run
        it:
        <pre className="mt-2 overflow-x-auto rounded-md bg-muted px-3 py-2 text-xs">
          <code>chmod +x sparq-gui-*.AppImage{"\n"}./sparq-gui-*.AppImage</code>
        </pre>
        A <code className="font-mono text-xs">.deb</code> is also published for
        Debian/Ubuntu. Choose the asset matching your CPU (x86-64 or arm64).
      </>
    ),
  },
];

/**
 * Best-effort, progressive-enhancement OS detection. Returns the detection bucket
 * we think the visitor is on, or null when we can't tell (in which case nothing is
 * highlighted and the full list still renders). We never branch behaviour on this —
 * it only reorders/annotates.
 */
function detectOs(): OsKey | null {
  if (typeof navigator === "undefined") return null;

  // userAgentData is the modern, UA-string-independent surface (Chromium). It can
  // expose the platform without the increasingly-frozen UA string.
  const uaData = (
    navigator as Navigator & {
      userAgentData?: { platform?: string };
    }
  ).userAgentData;
  const platform = (uaData?.platform ?? "").toLowerCase();
  const ua = navigator.userAgent.toLowerCase();
  const hay = `${platform} ${ua}`;

  // Apple Silicon vs Intel is not reliably distinguishable from the browser, so we
  // default macOS to the Apple-Silicon card (the common case for new Macs) and let
  // the user pick Intel explicitly — both are always visible.
  if (/mac|iphone|ipad|ipod/.test(hay)) return "macos-arm";
  if (/win/.test(hay)) return "windows";
  if (/linux|android|cros/.test(hay)) return "linux";
  return null;
}

export function DownloadClient() {
  // null until hydrated → SSR renders the full list with nothing highlighted, so
  // there is no hydration mismatch (the server can't know the OS).
  const [detected, setDetected] = React.useState<OsKey | null>(null);

  React.useEffect(() => {
    setDetected(detectOs());
  }, []);

  // Highlighted-first ordering: the detected card floats to the top of the list,
  // but every platform stays present and linkable.
  const ordered = React.useMemo(() => {
    if (!detected) return PLATFORMS;
    const match = (p: Platform) => p.os === detected;
    return [...PLATFORMS].sort(
      (a, b) => Number(match(b)) - Number(match(a)),
    );
  }, [detected]);

  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <h1 className="text-2xl font-semibold">Download sparq</h1>
        <p className="measure text-muted-foreground">
          The sparq desktop GUI bundles the engine and the playground into a
          native app for macOS, Windows, and Linux. Prefer not to install
          anything? The full SPARQL playground runs entirely in your browser at{" "}
          <Link
            className="text-primary underline-offset-4 hover:underline"
            href="/try"
          >
            /try
          </Link>{" "}
          — no download required.
        </p>
      </header>

      {/* Unsigned-build honesty banner — front and centre, not buried. */}
      <div
        role="note"
        className="flex items-start gap-3 rounded-xl bg-[color-mix(in_oklch,var(--warning)_12%,transparent)] px-4 py-3 ring-1 ring-[color-mix(in_oklch,var(--warning)_30%,transparent)]"
      >
        <TriangleAlert
          className="mt-0.5 size-5 shrink-0 text-[var(--warning)]"
          aria-hidden
        />
        <div className="space-y-1 text-sm">
          <p className="font-medium">Developer builds — unsigned.</p>
          <p className="measure text-muted-foreground">
            These desktop bundles are <strong>not code-signed or
            notarized</strong>, so macOS Gatekeeper and Windows SmartScreen will
            warn the first time you open them. That&rsquo;s expected — see the
            per-OS instructions on each card to allow the app. Officially signed
            and notarized builds (Apple Developer ID + Windows Authenticode) are
            coming; until then, only install builds you fetched yourself from the{" "}
            <a
              className="text-primary underline-offset-4 hover:underline"
              href={RELEASES_URL}
              target="_blank"
              rel="noopener noreferrer"
            >
              official Releases page
            </a>
            . Each release attaches a{" "}
            <code className="font-mono text-xs">SHA256SUMS</code> manifest and
            SLSA build-provenance attestations you can verify with{" "}
            <code className="font-mono text-xs">gh attestation verify</code>.
          </p>
        </div>
      </div>

      {/* Desktop GUI downloads */}
      <section className="space-y-4">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-xl font-semibold">Desktop app</h2>
          {detected && (
            <Badge variant="success" className="h-6">
              Detected your platform
            </Badge>
          )}
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          {ordered.map((p) => {
            const isMatch = detected !== null && p.os === detected;
            return (
              <Card
                key={`${p.os}`}
                className={
                  isMatch
                    ? "ring-2 ring-primary/60"
                    : undefined
                }
              >
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    {p.icon}
                    <span className="flex-1">{p.label}</span>
                    {isMatch && (
                      <Badge variant="success" className="h-5">
                        You
                      </Badge>
                    )}
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3 text-sm">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="muted">{p.ext}</Badge>
                    <Badge variant="warning">Unsigned</Badge>
                  </div>
                  <Button asChild variant="outline" size="sm" className="w-full">
                    <a
                      href={LATEST_RELEASE_URL}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Download className="size-4" />
                      Get the {p.ext} from the latest release
                    </a>
                  </Button>
                  <details className="text-muted-foreground">
                    <summary className="cursor-pointer select-none font-medium text-foreground">
                      First-launch instructions
                    </summary>
                    <div className="measure pt-2 leading-relaxed">
                      {p.bypass}
                    </div>
                  </details>
                </CardContent>
              </Card>
            );
          })}
        </div>

        <p className="measure text-xs text-muted-foreground">
          Releases are published from the build matrix on each tagged version. If
          the latest-release page shows no assets yet, the first signed-off
          release hasn&rsquo;t shipped — check back, or watch the{" "}
          <a
            className="text-primary underline-offset-4 hover:underline"
            href={RELEASES_URL}
            target="_blank"
            rel="noopener noreferrer"
          >
            Releases page
          </a>
          . Assets are named{" "}
          <code className="font-mono text-xs">
            sparq-gui-&lt;version&gt;-&lt;platform&gt;-&hellip;
          </code>{" "}
          (e.g. <code className="font-mono text-xs">arm64-darwin</code>,{" "}
          <code className="font-mono text-xs">win-x64</code>).
        </p>
      </section>

      {/* CLI + server binaries */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">Command-line & server</h2>
        <p className="measure text-sm text-muted-foreground">
          The same releases also carry the standalone{" "}
          <code className="font-mono text-xs">sparq</code> CLI and the{" "}
          <code className="font-mono text-xs">sparq-server</code> HTTP server
          binaries, built across a broad OS &times; CPU matrix
          (macOS/Windows/Linux, x86-64 micro-architecture tiers + arm64). These
          are plain executables — no signing dialog — verified against the
          release&rsquo;s <code className="font-mono text-xs">SHA256SUMS</code>.
        </p>
        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Terminal className="size-5" aria-hidden />
                sparq CLI
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-muted-foreground">
              <p className="measure">
                Load, query, and convert RDF from the terminal. Download the
                archive for your platform, extract, and put the binary on your{" "}
                <code className="font-mono text-xs">PATH</code>.
              </p>
              <Button asChild variant="outline" size="sm" className="w-full">
                <a
                  href={LATEST_RELEASE_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Download className="size-4" />
                  CLI binaries
                </a>
              </Button>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Server className="size-5" aria-hidden />
                sparq-server
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-muted-foreground">
              <p className="measure">
                Serve a dataset over the SPARQL 1.1 Protocol with metrics and a
                Service Description. The desktop GUI can connect to a running
                server in endpoint mode.
              </p>
              <Button asChild variant="outline" size="sm" className="w-full">
                <a
                  href={LATEST_RELEASE_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Download className="size-4" />
                  Server binaries
                </a>
              </Button>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* No-install option */}
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">No install needed</h2>
        <p className="measure text-sm text-muted-foreground">
          The web playground runs the same engine compiled to WebAssembly,
          entirely in your browser tab — nothing is uploaded, nothing is
          installed.
        </p>
        <Button asChild variant="default" size="sm">
          <Link href="/try">
            Open the playground
            <ExternalLink className="size-4" />
          </Link>
        </Button>
      </section>
    </div>
  );
}
