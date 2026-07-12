"use client";
// [GPT-5.6] sq-lsp7k.1.2 — operational shared Forms panel consuming F1 JSON.
import * as React from "react";
import { Braces, FilePenLine, LoaderCircle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { FormRenderer } from "@/components/workbench/form-renderer";
import { FORMS_DEMO_DESCRIPTION } from "@/data/forms-demo";
import { parseFormDescription, type FormDescription, type FormMode, type TermRef } from "@/lib/form-description";

export function FormsTool() {
  const [description, setDescription] = React.useState<FormDescription>(FORMS_DEMO_DESCRIPTION);
  const [json, setJson] = React.useState(() => JSON.stringify(FORMS_DEMO_DESCRIPTION, null, 2));
  const [error, setError] = React.useState<string | null>(null);
  const [request, setRequest] = React.useState<string | null>(null);
  const loadJson = () => { try { setDescription(parseFormDescription(json)); setError(null); setRequest(null); } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); } };
  const onModeRequest = (mode: FormMode) => setRequest(mode === description.mode ? null : `Previewing ${mode} mode. A workspace host re-derives FormDescription JSON for a permanent mode change.`);
  const onShapeRequest = (shape: TermRef) => setRequest(`Shape ${shape.value} requested. The host must re-derive its groups through sparq-forms.`);
  return <div className="flex h-full flex-col" data-tool-panel="forms">
    <div className="flex shrink-0 items-center gap-2 border-b bg-card px-3 py-1.5"><FilePenLine className="size-3.5 text-muted-foreground" aria-hidden /><span className="text-xs font-medium text-muted-foreground">Shape-directed form</span><Badge variant="outline" className="h-5 text-[10px]">FormDescription JSON</Badge><Badge variant="muted" className="h-5 text-[10px]">Draft only · F4 commits</Badge></div>
    <div className="min-h-0 flex-1 overflow-y-auto p-3"><div className="mx-auto max-w-5xl space-y-3">
      <details className="rounded-md border bg-card" data-form-json-source><summary className="flex cursor-pointer list-none items-center gap-2 px-3 py-2 text-xs font-medium [&::-webkit-details-marker]:hidden"><Braces className="size-3.5 text-muted-foreground" aria-hidden />Load FormDescription JSON<span className="ml-auto text-[10px] font-normal text-muted-foreground">Desktop and hosted web consume the same contract</span></summary><div className="space-y-2 border-t p-3"><textarea value={json} onChange={(event) => setJson(event.currentTarget.value)} className="h-48 w-full resize-y rounded-md border bg-background p-2 font-mono text-[10px] outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-label="FormDescription JSON" spellCheck={false} /><div className="flex items-center gap-2"><Button type="button" size="sm" onClick={loadJson} data-load-form-json>Render JSON</Button>{error ? <p role="alert" className="text-xs text-destructive">{error}</p> : null}</div></div></details>
      {request ? <p className="flex items-start gap-2 rounded-md border border-primary/25 bg-primary/5 p-2 text-xs text-muted-foreground" role="status"><LoaderCircle className="mt-0.5 size-3.5 shrink-0" aria-hidden />{request}</p> : null}
      <FormRenderer description={description} onDescriptionChange={setDescription} onModeRequest={onModeRequest} onShapeRequest={onShapeRequest} />
    </div></div>
  </div>;
}
