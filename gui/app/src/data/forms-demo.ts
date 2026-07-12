// [GPT-5.6] sq-lsp7k.1.2 — static F1 FormDescription JSON input; no browser derivation.
import type { FormDescription, FormField, TermRef } from "@/lib/form-description";
const DASH = "http://datashapes.org/dash#";
const XSD = "http://www.w3.org/2001/XMLSchema#";
const SH = "http://www.w3.org/ns/shacl#";
const EX = "http://example.org/";
const iri = (value: string): TermRef => ({ kind: "iri", value });
const lit = (value: string, datatype?: string): TermRef => ({ kind: "literal", value, ...(datatype ? { datatype } : {}) });
function field(partial: Partial<FormField> & Pick<FormField, "path" | "label">): FormField {
  return { property_shape: { kind: "bnode", value: `demo-${partial.label.replaceAll(" ", "-")}` }, multi: false, editable: true, widget: {}, values: [], constraints: {}, ...partial };
}
const address: FormDescription = {
  focus: { kind: "bnode", value: "address-1" }, mode: "edit",
  shapes: [{ shape: iri(`${EX}AddressShape`), label: "Address", via: "explicit" }], shape: iri(`${EX}AddressShape`),
  groups: [{ kind: "default", fields: [
    field({ path: `<${EX}street>`, label: "Street", required: true, widget: { editor: `${DASH}TextFieldEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("1 Coyote Way") }], constraints: { min_count: 1, max_count: 1, datatype: [`${XSD}string`] } }),
    field({ path: `<${EX}city>`, label: "City", required: true, widget: { editor: `${DASH}TextFieldEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("Tumbleweed") }], constraints: { min_count: 1, max_count: 1, datatype: [`${XSD}string`] } }),
  ] }],
};
export const FORMS_DEMO_DESCRIPTION: FormDescription = {
  focus: iri(`${EX}order42`), mode: "edit",
  shapes: [
    { shape: iri(`${EX}OrderShape`), label: "Order", via: "target-class" },
    { shape: iri(`${EX}AuditableShape`), label: "Auditable item", via: "applicable-to-class" },
  ], shape: iri(`${EX}OrderShape`), groups: [
    { kind: "declared", group: iri(`${EX}SummaryGroup`), label: "Summary", order: 0, fields: [
      field({ path: `<${EX}title>`, label: "Title", description: "A required single-line literal.", required: true,
        widget: { editor: `${DASH}TextFieldEditor`, viewer: `${DASH}LiteralViewer`, score: 10, editor_alternatives: [`${DASH}TextAreaEditor`], viewer_alternatives: [`${DASH}ValueTableViewer`] },
        values: [{ term: lit("Order 42") }], constraints: { min_count: 1, max_count: 1, datatype: [`${XSD}string`] } }),
      field({ path: `<${EX}notes>`, label: "Notes", multi: true, widget: { editor: `${DASH}TextAreaEditor`, viewer: `${DASH}LiteralViewer`, score: 20 }, values: [{ term: lit("Leave at reception.\nCall on arrival.") }], constraints: { datatype: [`${XSD}string`], single_line: false } }),
      field({ path: `<${EX}status>`, label: "Status", required: true, widget: { editor: `${DASH}EnumSelectEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("submitted") }], constraints: { min_count: 1, max_count: 1, in_values: [lit("draft"), lit("submitted"), lit("shipped")] } }),
      field({ path: `<${EX}active>`, label: "Active", widget: { editor: `${DASH}BooleanSelectEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("true", `${XSD}boolean`) }], constraints: { max_count: 1, datatype: [`${XSD}boolean`] } }),
    ] },
    { kind: "declared", group: iri(`${EX}ScheduleGroup`), label: "Schedule", order: 1, fields: [
      field({ path: `<${EX}dueDate>`, label: "Due date", widget: { editor: `${DASH}DatePickerEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("2026-07-31", `${XSD}date`) }], constraints: { max_count: 1, datatype: [`${XSD}date`] } }),
      field({ path: `<${EX}reviewedAt>`, label: "Reviewed at", widget: { editor: `${DASH}DateTimePickerEditor`, viewer: `${DASH}LiteralViewer`, score: 10 }, values: [{ term: lit("2026-07-12T09:30:00Z", `${XSD}dateTime`) }], constraints: { max_count: 1, datatype: [`${XSD}dateTime`] } }),
    ] },
    { kind: "declared", group: iri(`${EX}LinksGroup`), label: "Links and details", order: 2, fields: [
      field({ path: `<${EX}customer>`, label: "Customer", required: true,
        widget: { editor: `${DASH}InstancesSelectEditor`, viewer: `${DASH}LabelViewer`, score: 12, editor_alternatives: [`${DASH}AutoCompleteEditor`, `${DASH}URIEditor`], viewer_alternatives: [`${DASH}HyperlinkViewer`, `${DASH}ValueTableViewer`] },
        values: [{ term: iri(`${EX}acme`) }], constraints: { min_count: 1, max_count: 1, class: [iri(`${EX}Customer`)], node_kind: [`${SH}IRI`] } }),
      field({ path: `<${EX}shippingAddress>`, label: "Shipping address", widget: { editor: `${DASH}DetailsEditor`, viewer: `${DASH}DetailsViewer`, score: 15 }, values: [{ term: { kind: "bnode", value: "address-1" }, nested: address }], constraints: { max_count: 1, node_shape: iri(`${EX}AddressShape`) } }),
      field({ path: `<${EX}relatedBlankNode>`, label: "Related blank node", widget: { editor: `${DASH}BlankNodeEditor`, viewer: `${DASH}LabelViewer` }, values: [{ term: { kind: "bnode", value: "related-1" } }], constraints: { max_count: 1, node_kind: [`${SH}BlankNode`] } }),
    ] },
    { kind: "other", label: "Other properties", fields: [
      field({ property_shape: undefined, path: `<${EX}homepage>`, label: "Homepage", multi: true, editable: false,
        widget: { viewer: `${DASH}HyperlinkViewer`, score: 5, viewer_alternatives: [`${DASH}LabelViewer`, `${DASH}ValueTableViewer`] }, values: [{ term: iri("https://example.org/orders/42") }] }),
    ] },
  ],
};
