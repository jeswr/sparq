// [SONNET-4.6] Pin the shared numeric-literal classification used by both result renderers.
import assert from "node:assert/strict";
import test from "node:test";

import { isNumericLiteral } from "../src/lib/numeric-literal.ts";

const XSD = "http://www.w3.org/2001/XMLSchema#";

test("isNumericLiteral accepts finite literals with supported XSD numeric datatypes", () => {
  for (const [datatype, value] of [
    ["integer", "-3"],
    ["decimal", "3.5"],
    ["double", "1.2e3"],
    ["byte", "127"],
    ["unsignedLong", "42"],
  ]) {
    assert.equal(
      isNumericLiteral({ type: "literal", datatype: XSD + datatype, value }),
      true,
      datatype,
    );
  }
});

test("isNumericLiteral rejects non-numeric terms, datatypes, and values", () => {
  assert.equal(isNumericLiteral(undefined), false);
  assert.equal(isNumericLiteral({ type: "uri", value: "https://example.test/3" }), false);
  assert.equal(isNumericLiteral({ type: "literal", value: "3" }), false);
  assert.equal(
    isNumericLiteral({ type: "literal", datatype: XSD + "string", value: "3" }),
    false,
  );
  assert.equal(
    isNumericLiteral({ type: "literal", datatype: XSD + "double", value: "NaN" }),
    false,
  );
});
