// [GPT-5.6] sq-lsp7k.1.2 — renderer dispatch over F1's chosen DASH widget IRIs.
import type { FormField, FormMode, TermRef } from "./form-description.js";

export type EditorKind = "text" | "lang-text" | "textarea" | "lang-textarea" | "boolean" |
  "date" | "datetime" | "enum" | "iri-ref" | "nested" | "blank-node";
export type ViewerKind = "label" | "literal" | "hyperlink" | "value-table" | "nested" |
  "image" | "html" | "term";

export function dashName(iri: string | undefined): string {
  if (!iri) return "";
  const boundary = Math.max(iri.lastIndexOf("#"), iri.lastIndexOf("/"));
  return boundary >= 0 ? iri.slice(boundary + 1) : iri;
}
export function editorKind(widgetIri: string | undefined): EditorKind {
  switch (dashName(widgetIri)) {
    case "TextAreaEditor": case "RichTextEditor": return "textarea";
    case "TextFieldWithLangEditor": return "lang-text";
    case "TextAreaWithLangEditor": return "lang-textarea";
    case "BooleanSelectEditor": return "boolean";
    case "DatePickerEditor": return "date";
    case "DateTimePickerEditor": return "datetime";
    case "EnumSelectEditor": return "enum";
    case "AutoCompleteEditor": case "InstancesSelectEditor": case "URIEditor":
    case "SubClassEditor": return "iri-ref";
    case "DetailsEditor": return "nested";
    case "BlankNodeEditor": return "blank-node";
    default: return "text";
  }
}
export function viewerKind(widgetIri: string | undefined): ViewerKind {
  switch (dashName(widgetIri)) {
    case "LabelViewer": case "URIViewer": case "LangStringViewer": return "label";
    case "LiteralViewer": return "literal";
    case "HyperlinkViewer": return "hyperlink";
    case "ValueTableViewer": return "value-table";
    case "DetailsViewer": case "BlankNodeViewer": return "nested";
    case "ImageViewer": return "image";
    case "HTMLViewer": return "html";
    default: return "term";
  }
}
export function widgetOptions(field: FormField, mode: FormMode): string[] {
  const selected = mode === "edit" ? field.widget.editor : field.widget.viewer;
  const alternatives = mode === "edit" ? field.widget.editor_alternatives ?? [] : field.widget.viewer_alternatives ?? [];
  return [...new Set([selected, ...alternatives].filter((value): value is string => !!value))];
}
const XSD = "http://www.w3.org/2001/XMLSchema#";
const RDF_LANG = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
export function emptyTermFor(field: FormField, widgetIri = field.widget.editor): TermRef {
  switch (editorKind(widgetIri)) {
    case "iri-ref": return { kind: "iri", value: "" };
    case "blank-node": case "nested": return { kind: "bnode", value: "" };
    case "boolean": return { kind: "literal", value: "false", datatype: `${XSD}boolean` };
    case "date": return { kind: "literal", value: "", datatype: `${XSD}date` };
    case "datetime": return { kind: "literal", value: "", datatype: `${XSD}dateTime` };
    case "lang-text": case "lang-textarea": return { kind: "literal", value: "", datatype: RDF_LANG, language: field.constraints.language_in?.[0] ?? "" };
    case "enum": return field.constraints.in_values?.[0] ?? { kind: "literal", value: "" };
    default: return { kind: "literal", value: "", datatype: field.constraints.datatype?.[0] };
  }
}
