import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft, ExternalLink } from "lucide-react";

// [OPUS-4.8] sq-3p0z (#822) — the Verifiable-Credentials import-and-query surface.
//
// Built DEMO-FIRST (research/website-redesign.md §4): a minimal header → the live import +
// query workbench immediately → a one-line caption → the import-vs-verify honesty behind a
// closed <details>. The "live · in your tab" tier and the "imported, not verified" qualifier
// stay VISIBLE in the header badges (privacy-claims gate); the full detail relocates into the
// disclosure. The realised proposal lives in research/gui-design.md §2b.
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { VerifiableCredentials } from "@/components/verifiable-credentials";

export const metadata: Metadata = {
  title: "Import & query Verifiable Credentials",
  description:
    "Drag-drop, paste or fetch a W3C Verifiable Credential and run SPARQL over its claims — the sparq engine compiled to WebAssembly, entirely in your browser tab. Importing and querying is live; cryptographic verification and zero-knowledge proofs are the separate research-grade ZK estate (not externally audited).",
};

const REPO_URL = "https://github.com/sparq-org/sparq";

export default function VerifiableCredentialsPage() {
  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/examples">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to examples
        </Link>
      </Button>

      {/* Minimal header: title + the live tier + the import-vs-verify qualifier (the full
          honesty hedge moves into the caveat disclosure below). */}
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold tracking-tight">
          Import &amp; query your Verifiable Credentials
        </h1>
        <div className="flex flex-wrap gap-2">
          <Badge variant="success">Live in your tab — real SPARQL over your VCs</Badge>
          <Badge variant="warning">Imported &amp; queried — not cryptographically verified</Badge>
        </div>
      </header>

      {/* The workbench, immediately. */}
      <VerifiableCredentials />

      {/* One-line caption. */}
      <p className="measure text-sm text-muted-foreground">
        Drag in a credential (JSON-LD VC, Turtle, N-Quads), or paste / fetch one, and run real
        SPARQL over its claims — joins across multiple credentials included — entirely in your
        browser, with nothing sent to a server.
      </p>

      {/* The import-vs-verify honesty — behind a closed disclosure. */}
      <details className="rounded-xl border bg-card px-4 py-3 text-sm">
        <summary className="cursor-pointer select-none font-medium">
          Details &amp; honest limits
        </summary>
        <div className="mt-3 space-y-3 text-muted-foreground">
          <p>
            A W3C Verifiable Credential is an RDF document — a JSON-LD document that maps to RDF
            (parsed in-tab by the engine&rsquo;s JSON-LD reader), or one already serialised as
            Turtle / N-Quads. So &ldquo;load it into a graph and query it&rdquo; is just the
            existing ingest path, and runs live in your browser via the sparq engine compiled to
            WebAssembly. Every imported credential is queried together, so you can join across
            them (e.g. &ldquo;is the holder of this licence the same subject as this degree?&rdquo;).
          </p>
          <p>
            <strong className="text-foreground">In-tab JSON-LD limit.</strong> The lean
            browser bundle does not fetch a remote{" "}
            <code className="font-mono">@context</code> — a JSON-LD credential that references the
            W3C VC context only by URL must carry that context inline, be pre-expanded, or be
            imported as Turtle / N-Quads. The desktop / server engine has no such limit.
          </p>
          <p>
            <strong className="text-foreground">Querying is not verifying.</strong> This surface
            does <strong className="text-foreground">not</strong> check a credential&rsquo;s
            signature, its issuer, or its revocation status, and it produces no zero-knowledge
            proof. SD-JWT-VC / JWT-VC envelopes are recognised but not decoded (they are not plain
            RDF). VC <em>verification</em> and zero-knowledge proofs over credentials are the
            separate sparq ZK estate, which is <strong className="text-foreground">research-grade
            and not externally audited</strong> — an external accredited-cryptographer audit is
            required before any ZK security, privacy or attestation claim may be relied upon
            (tracked as <code className="font-mono">sq-qhy4</code>). Treat any future
            &ldquo;verified&rdquo; affordance as a research demonstration, not a production
            guarantee.
          </p>
          {/* [FABLE-5] persistent underline: distinguishable without color (link-in-text-block, sq-0rbfn) */}
          <p className="flex flex-wrap gap-4">
            <a
              className="inline-flex items-center gap-1 text-primary underline underline-offset-4"
              href={`${REPO_URL}/tree/main/crates/sparq-zk`}
              target="_blank"
              rel="noopener noreferrer"
            >
              sparq-zk on GitHub <ExternalLink className="size-3.5" />
            </a>
            <a
              className="inline-flex items-center gap-1 text-primary underline underline-offset-4"
              href={`${REPO_URL}/blob/main/SECURITY.md`}
              target="_blank"
              rel="noopener noreferrer"
            >
              SECURITY.md <ExternalLink className="size-3.5" />
            </a>
          </p>
        </div>
      </details>
    </div>
  );
}
