import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import Script from "next/script";
import { Toaster } from "sonner";

import "./globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppShell } from "@/components/layout/app-shell";
import { withBasePath } from "@/lib/base-path";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
});

export const metadata: Metadata = {
  title: {
    default: "sparq — a SOTA RDF + SPARQL engine, live in your tab",
    template: "%s · sparq",
  },
  description:
    "sparq is a state-of-the-art Rust RDF + SPARQL engine with a browser WASM port. This showcase runs real sparq — SPARQL, ZK proofs, MPC, Solid — live in your tab, honestly labelled where it cannot.",
  // [OPUS-4.8] sq-9vw5 — basePath-switched so the favicon resolves under both the Pages
  // `/sparq` prefix AND the Tauri root-relative build (a hardcoded `/sparq/…` is NOT
  // rewritten by Next, so it must read the env-derived base path).
  icons: { icon: withBasePath("/logo.svg") },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f7fbfc" },
    { media: "(prefers-color-scheme: dark)", color: "#13181c" },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        {/* [OPUS-4.8] coi-serviceworker shim — synthesises COOP/COEP so
            `window.crossOriginIsolated` becomes true on GitHub Pages (which cannot
            set those headers itself). That unlocks SharedArrayBuffer, so @aztec/bb.js
            can spin worker threads and the in-tab ZK prover multithreads. Loaded
            beforeInteractive with an absolute, basePath-prefixed src so the service
            worker registers from `${basePath}/coi-serviceworker.js` with that scope.
            [OPUS-4.8] sq-9vw5 — basePath-switched (was hardcoded `/sparq/…`) so it
            resolves under both Pages and the Tauri root-relative build. It changes only
            thread count — never what any proof proves or its ZK property. */}
        <Script
          src={withBasePath("/coi-serviceworker.js")}
          strategy="beforeInteractive"
        />
      </head>
      <body className={`${inter.variable} font-sans antialiased`}>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <TooltipProvider>
            <AppShell>{children}</AppShell>
          </TooltipProvider>
          <Toaster richColors position="bottom-right" />
        </ThemeProvider>
      </body>
    </html>
  );
}
