import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";

import "./globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { EngineProvider } from "@/lib/engine-context";
import { WorkspaceProvider } from "@/lib/workspace-context";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
});

// [OPUS-4.8] sq-ixc3.8 / sq-ixc3.9 — the operational GUI is a DISTINCT app, not the marketing
// site. Its metadata names the workbench, never the showcase.
export const metadata: Metadata = {
  title: "sparq workbench",
  description:
    "The sparq operational workbench — query, validate, and inspect RDF over a live in-tab engine. A distinct desktop + web tool, not the marketing site.",
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
      <body className={`${inter.variable} font-sans antialiased`}>
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          {/* The engine context provides the ONE live store the whole workbench shares; the
              workspace context tracks the active workspace's imported sources + persists the
              save/open snapshot (sq-atb0) the Import drawer (sq-ixc3.13) writes on each import. */}
          <EngineProvider>
            <WorkspaceProvider>{children}</WorkspaceProvider>
          </EngineProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
