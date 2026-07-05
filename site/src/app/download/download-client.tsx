"use client";

// [OPUS-4.8] sq-vw3ax.11 — the /download page client body: DIRECT per-asset downloads.
//
// WHAT CHANGED (vs the old "link to the latest-release page" body). Every button is now a
// one-click DIRECT download of the right file for the visitor's platform, using GitHub's
// version-stable download endpoint:
//
//     https://github.com/jeswr/sparq/releases/latest/download/<alias>
//
// GitHub 302-redirects that URL straight to the asset on its CDN (no interstitial), PROVIDED
// the asset name is version-STABLE. The release pipeline names its primary bundles with the
// version embedded (`sparq-gui_<version>_<arch>.<ext>`, `sparq-cli-<version>-<tier>.tar.gz`),
// which is NOT stable, so this page targets version-agnostic ALIAS names
// (`sparq-gui-arm64-darwin.dmg`, `sparq-cli-x64-linux.tar.gz`, …). Those aliases are published
// by the release pipeline alongside the versioned assets (release.yml — a CI-infra-lane change
// that is the companion to this site change; see the PR body). The upshot: the button hrefs are
// STATIC and never need a per-release site rebuild — they resolve to whatever the latest release
// carries the moment a release ships.
//
// METADATA (progressive enhancement, NOT required for the buttons to work). On mount we fetch
// `api.github.com/repos/jeswr/sparq/releases/latest` (CORS-open, unauthenticated) purely to
// ENRICH each card with the version tag, the asset byte-size, and a copyable sha256 (the asset
// `digest` field the API now returns, so we never need the CORS-blocked SHA256SUMS body). If that
// fetch FAILS for any reason other than a definitive 404, the buttons stay live — the
// latest/download URL needs no API.
//
// NO-RELEASE-YET STATE (current reality: `/releases/latest` 404s because only pre-releases exist,
// and `/releases/latest/download/*` likewise only resolves to a non-prerelease release). On a
// 404 we KNOW there is no published release, so every download control renders as a subdued
// "No release yet — watch releases" link to the Releases page — the ONLY GitHub-page link
// retained in that state. It flips to live direct-download buttons with ZERO code change the
// moment the first non-prerelease ships.
//
// HONESTY: the desktop bundles are UNSIGNED developer builds (signing/notarization is the
// separate needs:user bead sq-v286.8). The unsigned-builds banner stays front-and-centre in
// BOTH states — no overclaiming.

import * as React from "react";
import { toast } from "sonner";
import {
  Apple,
  Bell,
  Check,
  Copy,
  Cpu,
  Download,
  ExternalLink,
  Loader2,
  MonitorSmartphone,
  Server,
  ShieldCheck,
  Terminal,
  TriangleAlert,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { withBasePath } from "@/lib/base-path";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

const REPO_URL = "https://github.com/jeswr/sparq";
const RELEASES_URL = `${REPO_URL}/releases`;
// The version-STABLE download endpoint. GitHub 302s `<LATEST_DL>/<asset>` to the asset on its
// CDN with no interstitial, so an <alias> that the latest release carries downloads in one click.
const LATEST_DL = `${REPO_URL}/releases/latest/download`;
const API_LATEST =
  "https://api.github.com/repos/jeswr/sparq/releases/latest";

// The platforms the release matrix produces a desktop GUI installer for (release.yml `gui-bundle`
// job). win-arm64 is deliberately absent (the Tauri bundler can't reliably cross-bundle an arm64
// Windows installer from the x64 host) — an honest gap, not a silent drop.
type OsKey = "macos-arm" | "macos-intel" | "windows" | "linux";

interface SecondaryAsset {
  /** Version-agnostic alias file name served at `<LATEST_DL>/<file>`. */
  file: string;
  /** Short button label, e.g. ".deb (Debian/Ubuntu)". */
  label: string;
}

interface GuiPlatform {
  /** Detection bucket this card satisfies. */
  os: OsKey;
  /** Card title, e.g. "macOS · Apple Silicon". */
  label: string;
  /** Full-width primary-row heading when this is the detected platform. */
  forYou: string;
  /** Version-agnostic alias file name (release.yml publishes it), served at `<LATEST_DL>/<file>`. */
  file: string;
  /** Human-facing short file name shown on the button, e.g. "sparq-gui.dmg". */
  shortName: string;
  /** The installer file kind, e.g. ".dmg". */
  ext: string;
  /** Optional extra download offered on the same card (Linux ships a .deb too). */
  secondary?: SecondaryAsset;
  icon: React.ReactNode;
  /** Per-OS unsigned-bundle first-launch bypass instructions. */
  bypass: React.ReactNode;
}

const GUI_PLATFORMS: GuiPlatform[] = [
  {
    os: "macos-arm",
    label: "macOS · Apple Silicon",
    forYou: "For your Mac (Apple Silicon)",
    file: "sparq-gui-arm64-darwin.dmg",
    shortName: "sparq-gui.dmg",
    ext: ".dmg",
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
    forYou: "For your Mac (Intel)",
    file: "sparq-gui-x64-darwin.dmg",
    shortName: "sparq-gui.dmg",
    ext: ".dmg",
    icon: <Apple className="size-5" aria-hidden />,
    bypass: (
      <>
        Same as Apple Silicon — <strong>right-click the app → Open</strong> on
        first launch, or clear the quarantine flag:
        <pre className="mt-2 overflow-x-auto rounded-md bg-muted px-3 py-2 text-xs">
          <code>xattr -dr com.apple.quarantine /Applications/sparq.app</code>
        </pre>
        Pick this build only on an Intel Mac; Apple Silicon Macs should use the
        arm64 build.
      </>
    ),
  },
  {
    os: "windows",
    label: "Windows · x64",
    forYou: "For your Windows PC",
    file: "sparq-gui-win-x64.msi",
    shortName: "sparq-gui.msi",
    ext: ".msi",
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
    label: "Linux · x86-64",
    forYou: "For your Linux machine",
    file: "sparq-gui-x64-linux.AppImage",
    shortName: "sparq-gui.AppImage",
    ext: ".AppImage",
    secondary: { file: "sparq-gui-x64-linux.deb", label: ".deb (Debian/Ubuntu)" },
    icon: <Cpu className="size-5" aria-hidden />,
    bypass: (
      <>
        The <code className="font-mono text-xs">.AppImage</code> is a portable,
        self-contained binary — no install needed. Mark it executable, then run
        it:
        <pre className="mt-2 overflow-x-auto rounded-md bg-muted px-3 py-2 text-xs">
          <code>chmod +x sparq-gui.AppImage{"\n"}./sparq-gui.AppImage</code>
        </pre>
        A <code className="font-mono text-xs">.deb</code> is published for
        Debian/Ubuntu (button above). arm64 Linux builds are attached to each
        release on the Releases page.
      </>
    ),
  },
];

// ---- GitHub Releases API metadata (progressive enhancement) ------------------------------------

/** One release asset, trimmed to what the cards render. */
interface ReleaseAsset {
  name: string;
  size: number;
  /** e.g. "sha256:<hex>" — GitHub attaches this per asset (CORS-open on api.github.com). */
  digest: string | null;
}

/** The metadata fetch state machine. */
type ReleaseState =
  | { kind: "loading" }
  // A published (non-prerelease) release exists — enrich buttons with version/size/sha.
  | { kind: "ready"; version: string; assets: Map<string, ReleaseAsset> }
  // HTTP 404: there is definitively no published release yet.
  | { kind: "none" }
  // Network/other failure: we can't confirm, so buttons stay live (latest/download needs no API).
  | { kind: "error" };

interface ApiReleaseShape {
  tag_name?: unknown;
  assets?: unknown;
}

function parseRelease(
  json: unknown,
): { version: string; assets: Map<string, ReleaseAsset> } | null {
  if (typeof json !== "object" || json === null) return null;
  const { tag_name, assets } = json as ApiReleaseShape;
  if (typeof tag_name !== "string" || !Array.isArray(assets)) return null;
  const map = new Map<string, ReleaseAsset>();
  for (const a of assets) {
    if (typeof a !== "object" || a === null) continue;
    const { name, size, digest } = a as {
      name?: unknown;
      size?: unknown;
      digest?: unknown;
    };
    if (typeof name !== "string" || typeof size !== "number") continue;
    map.set(name, {
      name,
      size,
      digest: typeof digest === "string" ? digest : null,
    });
  }
  return { version: tag_name, assets: map };
}

/**
 * Fetch the latest published release once on mount. A definitive 404 → `none` (no release yet);
 * a 200 with a parseable body → `ready`; anything else (network error, rate limit, unparseable)
 * → `error`, which keeps the direct-download buttons live but without metadata.
 */
function useLatestRelease(): ReleaseState {
  const [state, setState] = React.useState<ReleaseState>({ kind: "loading" });

  React.useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch(API_LATEST, {
          headers: { Accept: "application/vnd.github+json" },
        });
        if (cancelled) return;
        if (res.status === 404) {
          setState({ kind: "none" });
          return;
        }
        if (!res.ok) {
          setState({ kind: "error" });
          return;
        }
        const parsed = parseRelease(await res.json());
        if (cancelled) return;
        setState(parsed ? { kind: "ready", ...parsed } : { kind: "error" });
      } catch {
        if (!cancelled) setState({ kind: "error" });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return state;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const mb = bytes / (1024 * 1024);
  if (mb < 1) return `${(bytes / 1024).toFixed(0)} KB`;
  if (mb < 1024) return `${mb.toFixed(1)} MB`;
  return `${(mb / 1024).toFixed(2)} GB`;
}

/** The asset metadata for a given alias, if the ready release carries it. */
function assetOf(state: ReleaseState, file: string): ReleaseAsset | undefined {
  return state.kind === "ready" ? state.assets.get(file) : undefined;
}

// ---- OS detection (progressive enhancement) ----------------------------------------------------

/**
 * Best-effort OS detection. Returns the bucket we think the visitor is on, or null when we can't
 * tell (in which case nothing is promoted and the full grid renders). We never branch behaviour
 * on this — it only promotes/annotates.
 */
function detectOs(): OsKey | null {
  if (typeof navigator === "undefined") return null;
  const uaData = (
    navigator as Navigator & { userAgentData?: { platform?: string } }
  ).userAgentData;
  const platform = (uaData?.platform ?? "").toLowerCase();
  const ua = navigator.userAgent.toLowerCase();
  const hay = `${platform} ${ua}`;
  // Apple Silicon vs Intel is not reliably distinguishable from the browser, so we default macOS
  // to the Apple-Silicon card (the common case for new Macs); both are always visible.
  if (/mac|iphone|ipad|ipod/.test(hay)) return "macos-arm";
  if (/win/.test(hay)) return "windows";
  if (/linux|android|cros/.test(hay)) return "linux";
  return null;
}

// ---- small presentational pieces ---------------------------------------------------------------

/** A copyable sha256 line — click to copy the full hex to the clipboard. */
function Sha256Line({ digest }: { digest: string }) {
  const hex = digest.replace(/^sha256:/, "");
  const [copied, setCopied] = React.useState(false);
  // [OPUS-4.8] track the reset timer id so it can be cancelled on unmount / re-arm to avoid leaks.
  // `window.setTimeout` returns `number` in the DOM lib; type the ref as `number` to avoid the
  // @types/node global-augmentation ambiguity where `ReturnType<typeof window.setTimeout>`
  // resolves to Node's `Timeout` and the assignment fails typecheck (TS2322).
  const timerRef = React.useRef<number | null>(null);

  React.useEffect(() => {
    return () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, []);

  const copy = React.useCallback(async () => {
    try {
      await navigator.clipboard.writeText(hex);
      setCopied(true);
      toast.success("sha256 copied");
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => setCopied(false), 1500);
    } catch {
      toast.error("Couldn't copy to the clipboard");
    }
  }, [hex]);

  return (
    <div className="mt-2 space-y-1">
      <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
        <ShieldCheck className="size-3.5 text-[var(--success)]" aria-hidden />
        Verify the download (sha256)
      </div>
      <div className="flex items-start gap-2">
        {/* [FABLE-5] sq-ymr2e.10 — data-vr-mask: the checksum hex is release-specific. */}
        <code
          data-vr-mask
          className="min-w-0 flex-1 overflow-x-auto rounded-md bg-muted px-2 py-1 font-mono text-[11px] leading-relaxed break-all"
        >
          {hex}
        </code>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-7 shrink-0"
          onClick={copy}
          aria-label="Copy the sha256 checksum"
        >
          {copied ? (
            <Check className="size-4 text-[var(--success)]" />
          ) : (
            <Copy className="size-4" />
          )}
        </Button>
      </div>
    </div>
  );
}

/**
 * A direct-download control for one alias. In the `none` state it degrades to a subdued
 * "watch releases" link (the only GitHub-page link retained). Otherwise it is a live
 * one-click download of `<LATEST_DL>/<file>`; `<meta>` (version · size) is appended when known.
 */
function DownloadButton({
  file,
  children,
  state,
  variant = "outline",
  size = "sm",
  className,
}: {
  file: string;
  children: React.ReactNode;
  state: ReleaseState;
  variant?: React.ComponentProps<typeof Button>["variant"];
  size?: React.ComponentProps<typeof Button>["size"];
  className?: string;
}) {
  if (state.kind === "none") {
    return (
      <Button
        asChild
        variant="outline"
        size={size}
        className={cn("w-full text-muted-foreground", className)}
      >
        <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer">
          <Bell className="size-4" />
          No release yet — watch releases
        </a>
      </Button>
    );
  }
  const asset = assetOf(state, file);
  const meta =
    state.kind === "ready"
      ? [state.version, asset ? formatBytes(asset.size) : null]
          .filter(Boolean)
          .join(" · ")
      : null;
  return (
    <Button asChild variant={variant} size={size} className={cn("w-full", className)}>
      {/* The `download` attribute hints a save; on a cross-origin 302 the browser downloads the
          target regardless. The file name is version-stable, so no rebuild per release. */}
      <a href={`${LATEST_DL}/${file}`} download>
        {state.kind === "loading" ? (
          <Loader2 className="size-4 animate-spin" aria-hidden />
        ) : (
          <Download className="size-4" aria-hidden />
        )}
        <span className="truncate">
          {children}
          {/* [FABLE-5] sq-ymr2e.10 — data-vr-mask: version + byte-size churn with every
              release, so the visual-regression baselines mask them (mask, don't chase). */}
          {meta ? (
            <span data-vr-mask className="opacity-80">
              {" "}
              · {meta}
            </span>
          ) : null}
        </span>
      </a>
    </Button>
  );
}

/** The collapsed first-launch / verify details shared by every GUI card. */
function LaunchDetails({
  bypass,
  file,
  state,
}: {
  bypass: React.ReactNode;
  file: string;
  state: ReleaseState;
}) {
  const asset = assetOf(state, file);
  return (
    <details className="text-muted-foreground">
      <summary className="cursor-pointer select-none font-medium text-foreground">
        First launch &amp; verification
      </summary>
      <div className="measure pt-2 leading-relaxed">
        {bypass}
        {asset?.digest ? <Sha256Line digest={asset.digest} /> : null}
      </div>
    </details>
  );
}

// ---- CLI / server aliases ----------------------------------------------------------------------

/** The CLI/server alias platform token + archive extension for the detected OS. */
function cliTarget(os: OsKey | null): { token: string; ext: string; label: string } {
  switch (os) {
    case "macos-arm":
      return { token: "arm64-darwin", ext: "tar.gz", label: "macOS · Apple Silicon" };
    case "macos-intel":
      return { token: "x64-darwin", ext: "tar.gz", label: "macOS · Intel" };
    case "windows":
      return { token: "win-x64", ext: "zip", label: "Windows · x64" };
    case "linux":
      return { token: "x64-linux", ext: "tar.gz", label: "Linux · x86-64" };
    default:
      // Undetected: default to the portable Linux x86-64 archive; every platform's build is on
      // the Releases page (linked from the banner).
      return { token: "x64-linux", ext: "tar.gz", label: "Linux · x86-64" };
  }
}

// ---- page --------------------------------------------------------------------------------------

export function DownloadClient() {
  // null until hydrated → SSR renders the full grid with nothing promoted (no hydration mismatch;
  // the server can't know the OS).
  const [detected, setDetected] = React.useState<OsKey | null>(null);
  const release = useLatestRelease();

  React.useEffect(() => {
    setDetected(detectOs());
  }, []);

  const primary = React.useMemo(
    () => (detected ? GUI_PLATFORMS.find((p) => p.os === detected) ?? null : null),
    [detected],
  );
  const rest = React.useMemo(
    () => (primary ? GUI_PLATFORMS.filter((p) => p !== primary) : GUI_PLATFORMS),
    [primary],
  );

  const cli = React.useMemo(() => cliTarget(detected), [detected]);
  const cliArchive = `sparq-cli-${cli.token}.${cli.ext}`;
  const serverArchive = `sparq-server-${cli.token}.${cli.ext}`;

  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <h1 className="text-2xl font-semibold">Download sparq</h1>
        <p className="measure text-muted-foreground">
          The sparq desktop GUI bundles the engine and the workbench into a
          native app for macOS, Windows, and Linux. Prefer not to install
          anything? The full SPARQL workbench runs entirely in your browser at{" "}
          {/* [OPUS-4.8] sq-ymr2e.4 — inline prose links carry a PERSISTENT underline (WCAG 2.1
              §1.4.1 Use of Color / axe `link-in-text-block`): distinguishable without relying on
              the teal colour alone, which fails the 3:1 contrast vs the surrounding muted text.
              [OPUS-4.8] sq-4hiqe — /app is the single workbench (the /try REPL was removed); it is a
              separate overlaid Next app, so this is a HARD anchor, not a next/link soft nav. */}
          <a
            className="text-primary underline underline-offset-4"
            href={withBasePath("/app/")}
          >
            /app
          </a>{" "}
          — no download required.
        </p>
      </header>

      {/* Unsigned-build honesty banner — front and centre in BOTH states, never buried. */}
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
              className="text-primary underline underline-offset-4"
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
        <div className="flex flex-wrap items-center justify-between gap-3">
          <h2 className="text-xl font-semibold">Desktop app</h2>
          {release.kind === "ready" && (
            // [FABLE-5] sq-ymr2e.10 — data-vr-mask: the version tag churns per release.
            <Badge variant="muted" className="h-6 font-mono" data-vr-mask>
              {release.version}
            </Badge>
          )}
        </div>

        {/* Detected platform → full-width primary row. */}
        {primary && (
          <Card className="ring-2 ring-primary/60">
            <CardContent className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
              <div className="min-w-0 space-y-1">
                <div className="flex items-center gap-2">
                  {primary.icon}
                  <span className="text-base font-semibold">{primary.forYou}</span>
                  <Badge variant="success" className="h-5">
                    Detected
                  </Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  Recommended installer for your platform — {primary.label},{" "}
                  <code className="font-mono text-xs">{primary.ext}</code>.
                </p>
              </div>
              <div className="w-full space-y-2 md:w-80">
                <DownloadButton
                  file={primary.file}
                  state={release}
                  variant="default"
                  size="lg"
                >
                  Download {primary.shortName}
                </DownloadButton>
                {primary.secondary && release.kind !== "none" && (
                  <DownloadButton file={primary.secondary.file} state={release}>
                    {primary.secondary.label}
                  </DownloadButton>
                )}
              </div>
            </CardContent>
            <CardContent className="text-sm">
              <LaunchDetails
                bypass={primary.bypass}
                file={primary.file}
                state={release}
              />
            </CardContent>
          </Card>
        )}

        {/* Remaining platforms → compact grid (3-up when a platform is promoted, 2×2 otherwise). */}
        <div
          className={cn(
            "grid gap-4",
            primary ? "md:grid-cols-3" : "sm:grid-cols-2",
          )}
        >
          {rest.map((p) => (
            <Card key={p.os}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  {p.icon}
                  <span className="flex-1">{p.label}</span>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-sm">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant="muted">{p.ext}</Badge>
                  <Badge variant="warning">Unsigned</Badge>
                </div>
                <DownloadButton file={p.file} state={release}>
                  Download {p.shortName}
                </DownloadButton>
                {p.secondary && release.kind !== "none" && (
                  <DownloadButton file={p.secondary.file} state={release}>
                    {p.secondary.label}
                  </DownloadButton>
                )}
                <LaunchDetails bypass={p.bypass} file={p.file} state={release} />
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      {/* CLI + server binaries */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">Command-line &amp; server</h2>
        <p className="measure text-sm text-muted-foreground">
          The same releases also carry the standalone{" "}
          <code className="font-mono text-xs">sparq</code> CLI and the{" "}
          <code className="font-mono text-xs">sparq-server</code> HTTP server
          binaries, built across a broad OS &times; CPU matrix
          (macOS/Windows/Linux, x86-64 micro-architecture tiers + arm64). These
          are plain executables — no signing dialog. The buttons below fetch the
          build for <strong>{cli.label}</strong>; every other platform&rsquo;s
          archive is attached to the{" "}
          <a
            className="text-primary underline underline-offset-4"
            href={RELEASES_URL}
            target="_blank"
            rel="noopener noreferrer"
          >
            latest release
          </a>
          .
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
                archive, extract, and put the binary on your{" "}
                <code className="font-mono text-xs">PATH</code>.
              </p>
              <DownloadButton file={cliArchive} state={release}>
                Download sparq CLI
              </DownloadButton>
              <LaunchDetails
                bypass={
                  <>
                    Portable archive for <strong>{cli.label}</strong> (
                    <code className="font-mono text-xs">.{cli.ext}</code>).
                    Extract and run{" "}
                    <code className="font-mono text-xs">sparq</code> — no
                    installer, no signing dialog.
                  </>
                }
                file={cliArchive}
                state={release}
              />
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
              <DownloadButton file={serverArchive} state={release}>
                Download sparq-server
              </DownloadButton>
              <LaunchDetails
                bypass={
                  <>
                    Portable archive for <strong>{cli.label}</strong> (
                    <code className="font-mono text-xs">.{cli.ext}</code>).
                    Extract and run{" "}
                    <code className="font-mono text-xs">sparq-server</code>.
                  </>
                }
                file={serverArchive}
                state={release}
              />
            </CardContent>
          </Card>
        </div>
      </section>

      {/* No-install closer */}
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">No install needed</h2>
        <p className="measure text-sm text-muted-foreground">
          The web playground runs the same engine compiled to WebAssembly,
          entirely in your browser tab — nothing is uploaded, nothing is
          installed.
        </p>
        <div className="flex flex-wrap items-center gap-3">
          <Button asChild variant="default" size="sm">
            <a href={withBasePath("/app/")}>
              Open the workbench
              <ExternalLink className="size-4" />
            </a>
          </Button>
          <Badge variant="success" className="h-6 gap-1.5">
            <ShieldCheck className="size-3.5" aria-hidden />
            Runs locally — nothing uploaded
          </Badge>
        </div>
      </section>
    </div>
  );
}
