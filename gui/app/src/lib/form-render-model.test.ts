// [GPT-5.6] sq-lsp7k.1.2 — contract/dispatch tests against F1 golden JSON.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { parseFormDescription, termKey, type FormDescription, type FormField } from "./form-description.js";
import { dashName, editorKind, emptyTermFor, viewerKind, widgetOptions } from "./form-render-model.js";
async function golden(name: string): Promise<FormDescription> {
  const url = new URL(`../../../../crates/sparq-forms/tests/fixtures/${name}.golden.json`, import.meta.url);
  return parseFormDescription(await readFile(url, "utf8"));
}
const fields = (form: FormDescription) => form.groups.flatMap((group) => group.fields);

test("consumes F1 JSON without re-deriving layout", async () => {
  const form = await golden("person_groups");
  assert.equal(form.focus.value, "http://example.org/eve");
  assert.deepEqual(form.groups.map((g) => [g.kind, g.label, g.order]), [["declared", "Identity", 0], ["declared", "Employment", 1], ["other", "Other properties", undefined]]);
  assert.deepEqual(fields(form).slice(0, 5).map((f) => f.label), ["Full name", "Biography", "Motto", "Active", "Hire date"]);
});
test("core editors dispatch only from selected DASH IRIs", async () => {
  const person = fields(await golden("person_groups")); const order = fields(await golden("enum_nested"));
  const kind = (list: FormField[], label: string) => editorKind(list.find((f) => f.label === label)?.widget.editor);
  assert.deepEqual([kind(person, "Full name"), kind(person, "Biography"), kind(person, "Motto"), kind(person, "Active"), kind(person, "Hire date")], ["text", "textarea", "lang-text", "boolean", "date"]);
  assert.deepEqual([kind(order, "Status"), kind(order, "Customer"), kind(order, "Shipping address")], ["enum", "iri-ref", "nested"]);
  const misleading: FormField = { path: "<urn:b>", label: "No rescore", multi: false, editable: true, widget: { editor: "http://datashapes.org/dash#TextFieldEditor" }, values: [], constraints: { datatype: ["http://www.w3.org/2001/XMLSchema#boolean"] } };
  assert.equal(editorKind(misleading.widget.editor), "text");
});
test("retains nested descriptions and widget alternatives", async () => {
  const order = await golden("enum_nested"); const shipping = fields(order).find((f) => f.label === "Shipping address"); assert.ok(shipping);
  assert.deepEqual(shipping.values[0]?.nested?.groups[0]?.fields.map((f) => f.label), ["Street", "City"]);
  const name = fields(await golden("person_groups")).find((f) => f.label === "Full name"); assert.ok(name);
  assert.deepEqual(widgetOptions(name, "edit").map(dashName), ["TextFieldEditor", "TextAreaEditor"]);
  assert.deepEqual(widgetOptions(name, "view").map(dashName), ["LiteralViewer", "HyperlinkViewer", "ValueTableViewer"]);
});
test("viewer and empty-term fallbacks cover the F2 core", () => {
  assert.deepEqual([viewerKind("http://datashapes.org/dash#LabelViewer"), viewerKind("http://datashapes.org/dash#HyperlinkViewer"), viewerKind("http://datashapes.org/dash#ValueTableViewer"), viewerKind("urn:custom")], ["label", "hyperlink", "value-table", "term"]);
  const base: FormField = { path: "<urn:v>", label: "Value", multi: true, editable: true, widget: {}, values: [], constraints: {} };
  assert.equal(emptyTermFor(base, "http://datashapes.org/dash#URIEditor").kind, "iri");
  assert.equal(emptyTermFor(base, "http://datashapes.org/dash#DateTimePickerEditor").datatype, "http://www.w3.org/2001/XMLSchema#dateTime");
  assert.equal(emptyTermFor(base, "http://datashapes.org/dash#BlankNodeEditor").kind, "bnode");
  assert.notEqual(termKey({ kind: "literal", value: "x", language: "en" }), termKey({ kind: "literal", value: "x", language: "no" }));
});
test("rejects malformed roots early", () => {
  assert.throws(() => parseFormDescription('{"mode":"edit","groups":[],"shapes":[]}'), /focus/);
  assert.throws(() => parseFormDescription('{"focus":{"kind":"iri","value":"urn:x"},"mode":"preview","groups":[],"shapes":[]}'), /mode/);
});
