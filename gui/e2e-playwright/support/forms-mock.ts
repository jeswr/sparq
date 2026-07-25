// [GPT-5.6] sq-3eukz — one deterministic FormDescription factory shared by the mocked Tauri
// command and the structurally mocked sparq-wasm method. Both hosts therefore render equivalent
// content while still echoing focus/mode/shape so re-derivation assertions are non-vacuous.

export const formsMockPrelude = `
  function formsMockDescription(focus, mode, selectedShape) {
    var EX = "http://example.org/";
    var DASH = "http://datashapes.org/dash#";
    var shape = selectedShape || EX + "PersonShape";
    var local = String(focus || "").split(/[\\/#]/).filter(Boolean).pop() || "resource";
    var label = local.charAt(0).toUpperCase() + local.slice(1);
    return JSON.stringify({
      focus: { kind: "iri", value: focus },
      mode: mode,
      shapes: [
        { shape: { kind: "iri", value: EX + "PersonShape" }, label: "Person", via: "target-class" },
        { shape: { kind: "iri", value: EX + "AuditableShape" }, label: "Auditable", via: "applicable-to-class" }
      ],
      shape: { kind: "iri", value: shape },
      groups: [{
        kind: "declared",
        label: "Workspace profile",
        fields: [{
          path: "<" + EX + "name>",
          label: "Workspace name",
          multi: false,
          editable: mode === "edit",
          widget: {
            editor: mode === "edit" ? DASH + "TextFieldEditor" : undefined,
            viewer: DASH + "LiteralViewer"
          },
          values: [{ term: { kind: "literal", value: label } }],
          constraints: {}
        }]
      }]
    });
  }
`;

/** Install a non-enumerable mock method where every wasm-bindgen Store can structurally see it. */
export const wasmFormsMockScript = `
(function () {
  "use strict";
  ${formsMockPrelude}
  var LOG = [];
  Object.defineProperty(Object.prototype, "deriveForm", {
    configurable: true,
    enumerable: false,
    value: function (data, shapes, focus, format, optionsJson) {
      var options = JSON.parse(optionsJson || "{}");
      LOG.push({ data: data, shapes: shapes, focus: focus, format: format, optionsJson: optionsJson });
      return formsMockDescription(focus, options.mode || "edit", options.shape);
    }
  });
  window.__SPARQ_FORMS_WASM_LOG__ = LOG;
})();
`;
