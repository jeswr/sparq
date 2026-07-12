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
    typeof value.value === "string" &&
    (value.datatype === undefined || typeof value.datatype === "string") &&
    (value.language === undefined || typeof value.language === "string");
}
function invalid(path: string, expectation: string): never {
  throw new Error(`${path} ${expectation}`);
}
function assertOptionalString(value: unknown, path: string): void {
  if (value !== undefined && typeof value !== "string") invalid(path, "must be a string");
}
function assertOptionalBoolean(value: unknown, path: string): void {
  if (value !== undefined && typeof value !== "boolean") invalid(path, "must be a boolean");
}
function assertOptionalNumber(value: unknown, path: string): void {
  if (value !== undefined && (typeof value !== "number" || !Number.isFinite(value))) {
    invalid(path, "must be a finite number");
  }
}
function assertTerm(value: unknown, path: string): asserts value is TermRef {
  if (!isTermRef(value)) invalid(path, "must be a term reference");
}
function assertStringArray(value: unknown, path: string): void {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    invalid(path, "must be an array of strings");
  }
}
function assertTermArray(value: unknown, path: string): void {
  if (!Array.isArray(value)) invalid(path, "must be an array of term references");
  value.forEach((entry, index) => assertTerm(entry, `${path}[${index}]`));
}
function assertConstraints(value: unknown, path: string): asserts value is Constraints {
  if (!isRecord(value)) invalid(path, "must be an object");
  for (const key of ["datatype", "node_kind", "language_in"] as const) {
    if (value[key] !== undefined) assertStringArray(value[key], `${path}.${key}`);
  }
  for (const key of ["class", "in_values"] as const) {
    if (value[key] !== undefined) assertTermArray(value[key], `${path}.${key}`);
  }
  for (const key of ["min_inclusive", "max_inclusive", "min_exclusive", "max_exclusive", "root_class", "node_shape"] as const) {
    if (value[key] !== undefined) assertTerm(value[key], `${path}.${key}`);
  }
  for (const key of ["min_count", "max_count", "min_length", "max_length"] as const) {
    assertOptionalNumber(value[key], `${path}.${key}`);
  }
  for (const key of ["pattern", "pattern_flags"] as const) {
    assertOptionalString(value[key], `${path}.${key}`);
  }
  for (const key of ["unique_lang", "single_line"] as const) {
    assertOptionalBoolean(value[key], `${path}.${key}`);
  }
  if (value.or !== undefined) {
    if (!Array.isArray(value.or)) invalid(`${path}.or`, "must be an array");
    value.or.forEach((branch, index) => assertConstraints(branch, `${path}.or[${index}]`));
  }
}
function assertWidget(value: unknown, path: string): asserts value is WidgetChoice {
  if (!isRecord(value)) invalid(path, "must be an object");
  assertOptionalString(value.editor, `${path}.editor`);
  assertOptionalString(value.viewer, `${path}.viewer`);
  assertOptionalBoolean(value.explicit, `${path}.explicit`);
  assertOptionalNumber(value.score, `${path}.score`);
  if (value.editor_alternatives !== undefined) {
    assertStringArray(value.editor_alternatives, `${path}.editor_alternatives`);
  }
  if (value.viewer_alternatives !== undefined) {
    assertStringArray(value.viewer_alternatives, `${path}.viewer_alternatives`);
  }
}
function assertDescription(value: unknown, path: string): asserts value is FormDescription {
  if (!isRecord(value)) invalid(path, "must be a JSON object");
  assertTerm(value.focus, `${path}.focus`);
  if (value.mode !== "view" && value.mode !== "edit") {
    invalid(`${path}.mode`, 'must be "view" or "edit"');
  }
  assertOptionalString(value.role, `${path}.role`);
  if (!Array.isArray(value.shapes)) invalid(`${path}.shapes`, "must be an array");
  value.shapes.forEach((shape, index) => {
    const shapePath = `${path}.shapes[${index}]`;
    if (!isRecord(shape)) invalid(shapePath, "must be an object");
    assertTerm(shape.shape, `${shapePath}.shape`);
    assertOptionalString(shape.label, `${shapePath}.label`);
    if (!(["target-node", "target-class", "applicable-to-class", "explicit"] as unknown[]).includes(shape.via)) {
      invalid(`${shapePath}.via`, "must be a supported shape-selection reason");
    }
  });
  if (value.shape !== undefined) assertTerm(value.shape, `${path}.shape`);
  if (!Array.isArray(value.groups)) invalid(`${path}.groups`, "must be an array");
  value.groups.forEach((group, groupIndex) => {
    const groupPath = `${path}.groups[${groupIndex}]`;
    if (!isRecord(group)) invalid(groupPath, "must be an object");
    if (group.kind !== "default" && group.kind !== "declared" && group.kind !== "other") {
      invalid(`${groupPath}.kind`, "must be default, declared, or other");
    }
    if (group.group !== undefined) assertTerm(group.group, `${groupPath}.group`);
    assertOptionalString(group.label, `${groupPath}.label`);
    assertOptionalNumber(group.order, `${groupPath}.order`);
    if (!Array.isArray(group.fields)) invalid(`${groupPath}.fields`, "must be an array");
    group.fields.forEach((field, fieldIndex) => {
      const fieldPath = `${groupPath}.fields[${fieldIndex}]`;
      if (!isRecord(field)) invalid(fieldPath, "must be an object");
      if (field.property_shape !== undefined) assertTerm(field.property_shape, `${fieldPath}.property_shape`);
      if (typeof field.path !== "string") invalid(`${fieldPath}.path`, "must be a string");
      if (typeof field.label !== "string") invalid(`${fieldPath}.label`, "must be a string");
      assertOptionalString(field.description, `${fieldPath}.description`);
      assertOptionalNumber(field.order, `${fieldPath}.order`);
      assertOptionalBoolean(field.inverse, `${fieldPath}.inverse`);
      assertOptionalBoolean(field.required, `${fieldPath}.required`);
      if (typeof field.multi !== "boolean") invalid(`${fieldPath}.multi`, "must be a boolean");
      if (typeof field.editable !== "boolean") invalid(`${fieldPath}.editable`, "must be a boolean");
      assertWidget(field.widget, `${fieldPath}.widget`);
      assertConstraints(field.constraints, `${fieldPath}.constraints`);
      if (!Array.isArray(field.values)) invalid(`${fieldPath}.values`, "must be an array");
      field.values.forEach((formValue, valueIndex) => {
        const valuePath = `${fieldPath}.values[${valueIndex}]`;
        if (!isRecord(formValue)) invalid(valuePath, "must be an object");
        assertTerm(formValue.term, `${valuePath}.term`);
        if (formValue.nested !== undefined) {
          assertDescription(formValue.nested, `${valuePath}.nested`);
        }
      });
    });
  });
}

/** Parses the exact JSON emitted by `sparq_forms::FormDescription`. */
export function parseFormDescription(json: string): FormDescription {
  const value: unknown = JSON.parse(json);
  assertDescription(value, "FormDescription");
  return value;
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
