// [OPUS-4.8] sq-rvgr2.1 — renders the build-time-generated spec HTML fragment, mirroring
// components/papers/paper-html.tsx.
//
// The fragment is the <body> inner HTML emitted by Typst's native HTML export and
// post-processed into ReSpec/TR conventions (heading ids + a linked ToC) by
// scripts/build-specs.mjs — built from the SAME .typ source as the downloadable PDF, so the
// in-site render and the PDF cannot disagree. We render it as a static asset (no
// ReSpec/puppeteer/Chromium, no client JS) inside a scoped `.spec-prose` wrapper (see
// globals.css). The content is project-authored + built in this repo (not user input), so
// dangerouslySetInnerHTML is safe here.
export function SpecHtml({ html }: { html: string }) {
  return (
    <article
      className="spec-prose"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
