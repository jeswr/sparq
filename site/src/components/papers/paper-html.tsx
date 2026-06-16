// [OPUS-4.8] sq-gum8 — renders the build-time-generated paper HTML fragment.
//
// The fragment is the <body> inner HTML emitted by Typst's native HTML export
// (scripts/build-papers.mjs), built from the SAME .typ + the SAME injected evidence JSON as
// the downloadable PDF — so the in-site render and the PDF cannot show different numbers. We
// render it as a static asset (no WASM compiler shipped to the browser) inside a scoped
// `.paper-prose` wrapper (see globals.css). The content is project-authored + built in this
// repo (not user input), so dangerouslySetInnerHTML is safe here.
export function PaperHtml({ html }: { html: string }) {
  return (
    <article
      className="paper-prose"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
