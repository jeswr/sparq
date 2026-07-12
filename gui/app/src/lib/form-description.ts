// [GPT-5.6] sq-lsp7k.1.2 — TypeScript mirror of sparq-forms' serde JSON contract.
// This module only validates/consumes F1 output. Widget selection remains owned by
// crates/sparq-forms; the renderer must never infer a widget from constraints again.

export type FormMode = "view" | "edit";
export type TermKind = "iri" | "bnode" | "literal" | "triple";
export type GroupKind = "default" | "declared" | "other";
export type ShapeVia = "target-node" | "target-class" | "applicable-to-class" | "explicit";

export interface TermRef { kind: TermKind; value: string; datatype?: string; language?: string }
export interface ShapeChoice { shape: TermRef; label?: string; via: ShapeVia }
export interface WidgetChoice {
  editor?: string; viewer?: string; explicit?: boolean; score?: number;
  editor_alternatives?: string[]; viewer_alternatives?: string[];
}
export interface Constraints {
  min_count?: number; max_count?: number; datatype?: string[]; class?: TermRef[];
  node_kind?: string[]; in_values?: TermRef[]; pattern?: string; pattern_flags?: string;
  min_length?: number; max_length?: number; min_inclusive?: TermRef; max_inclusive?: TermRef;
  min_exclusive?: TermRef; max_exclusive?: TermRef; language_in?: string[];
  unique_lang?: boolean; single_line?: boolean; root_class?: TermRef;
  node_shape?: TermRef; or?: Constraints[];
}
export interface FormValue { term: TermRef; nested?: FormDescription }
export interface FormField {
  property_shape?: TermRef; path: string; inverse?: boolean; label: string; description?: string;
  order?: number; required?: boolean; multi: boolean; editable: boolean; widget: WidgetChoice;
  values: FormValue[]; constraints: Constraints;
}
export interface FormGroup {
  kind: GroupKind; group?: TermRef; label?: string; order?: number; fields: FormField[];
}
export interface FormDescription {
  focus: TermRef; mode: FormMode; role?: string; shapes: ShapeChoice[];
  shape?: TermRef; groups: FormGroup[];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function isTermRef(value: unknown): value is TermRef {
  return isRecord(value) &&
    (value.kind === "iri" || value.kind === "bnode" || value.kind === "literal" || value.kind === "triple") &&
    typeof value.value === "string";
}

/** Parses the exact JSON emitted by `sparq_forms::FormDescription`. */
export function parseFormDescription(json: string): FormDescription {
  const value: unknown = JSON.parse(json);
  if (!isRecord(value)) throw new Error("FormDescription must be a JSON object");
  if (!isTermRef(value.focus)) throw new Error("FormDescription.focus must be a term reference");
  if (value.mode !== "view" && value.mode !== "edit") throw new Error('FormDescription.mode must be "view" or "edit"');
  if (!Array.isArray(value.shapes)) throw new Error("FormDescription.shapes must be an array");
  if (!Array.isArray(value.groups)) throw new Error("FormDescription.groups must be an array");
  for (const group of value.groups) {
    if (!isRecord(group) || !Array.isArray(group.fields)) {
      throw new Error("Every FormDescription group must contain a fields array");
    }
  }
  return value as unknown as FormDescription;
}

export function termKey(term: TermRef): string {
  return JSON.stringify([term.kind, term.value, term.datatype ?? null, term.language ?? null]);
}
export function termLabel(term: TermRef): string {
  if (term.kind !== "iri") return term.value;
  const clean = term.value.replace(/\/$/, "");
  const boundary = Math.max(clean.lastIndexOf("#"), clean.lastIndexOf("/"));
  return boundary >= 0 ? clean.slice(boundary + 1) || term.value : term.value;
}
