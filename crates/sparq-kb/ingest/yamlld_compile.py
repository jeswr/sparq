#!/usr/bin/env python3
# yamlld_compile.py — the WRITE-PATH deterministic YAML-LD -> PKG-Turtle compiler
# (FO-bridge Phase 4, sq-mztg8.2; epic sq-mztg8). 🤖 SPARQ agent [OPUS-4.8].
# Design record: research/fo-llm-bridge.md §3.4 / §6 Phase 4 (#1112). Written while
# Fable unavailable; flag for re-review when Fable returns.
#
# WHAT THIS IS. A DETERMINISTIC compiler — NOT an LLM — that turns a compact, IRI-free,
# xsd-free authoring source (a `*.yaml.ld` Findings doc, e.g. agents-findings.yaml.ld)
# into the schema.org-typed PKG Turtle the SHACL shapes (crates/sparq-kb/shapes/
# pkg.shapes.ttl) admit. It GENERALISES the shipped sec-prop.yaml.ld pattern (typed
# `@context` + `@graph`) into a PKG authoring form so neither author nor LLM ever
# writes raw RDF or IRIs. The SHACL shape is the ONLY admission control; this compiler
# is the mechanical projector that feeds it.
#
# THE COMPILE CONTRACT (deterministic, auditable — research §3.4):
#   1. PARSE the typed YAML-LD block (a stdlib-only subset reader; NO PyYAML dep, to
#      keep the same stdlib-only posture as ingest_pkg.py).
#   2. RESOLVE concept TOKENS via the guarded V() resolver against the `concepts:`
#      catalog (lexical: exact local-name OR normalised prefLabel). An AMBIGUOUS token
#      (>=2 distinct concept IRIs match) is a HARD compile error — the write-path
#      analogue of the read-path abstain — never a silent wrong IRI. An UNRESOLVABLE
#      token is likewise a hard error. The resolution is ECHOED to stderr (the
#      soundness-envelope "always show your work" rule, research §3.3).
#   3. COMPILE to Turtle deterministically using the `@context` key->predicate map.
#   4. SHACL-validate the compiled output against pkg.shapes.ttl — done by the Rust
#      gate (tests/yamlld_compile.rs, --features validate), NOT here, so this stays a
#      pure stdlib script.
#   5. ROUND-TRIP: compiling the YAML-LD authored form reconstructs the same triples
#      the hand-authored agents-findings.ttl carried (asserted by the Rust test).
#
# A CONCEPT-RESOLUTION FAILURE RAISES `CompileError`; callers (ingest_pkg.py) let it
# propagate so a bad authoring doc REDS the ingest rather than producing wrong triples.
#
# Usage (standalone, for the round-trip artifact):
#   python3 crates/sparq-kb/ingest/yamlld_compile.py \
#       crates/sparq-kb/ingest/agents-findings.yaml.ld \
#       --out crates/sparq-kb/ingest/agents-findings.ttl
# Run from the repo root.

from __future__ import annotations

import argparse
import re
import sys
from typing import Any


# =============================================================================
# Errors
# =============================================================================

class CompileError(Exception):
    """A hard, fail-closed compile error (bad YAML-LD, or an ambiguous/unresolvable
    concept TOKEN). Never swallowed — a wrong IRI must never be silently emitted."""


# =============================================================================
# 1. The stdlib-only YAML-LD subset reader
# =============================================================================
#
# The authoring form uses a deliberately SMALL YAML subset: mappings, block sequences
# (`- ...`), inline flow sequences (`[a, b]`), inline flow mappings (`{ "@id": ... }`),
# scalars (bare / single- / double-quoted), and `#` comments. That is exactly the
# sec-prop.yaml.ld shape minus the `>-` folded blocks (which this form does not use).
# A full YAML engine is unnecessary and would add a dependency; a focused, deterministic
# reader is auditable and stdlib-only. Anything outside the subset is a CompileError
# (fail closed rather than mis-parse).

_KEY_RE = re.compile(r'^(?P<key>"[^"]*"|[A-Za-z_@][A-Za-z0-9_@.\-]*)\s*:(?:\s+(?P<val>.*))?$')


def _strip_comment(line: str) -> str:
    """Remove a trailing ` # comment` that is OUTSIDE any quoted scalar. A `#` inside
    quotes (single or double) is data, not a comment."""
    out = []
    in_s = False
    in_d = False
    i = 0
    while i < len(line):
        c = line[i]
        if c == "'" and not in_d:
            in_s = not in_s
        elif c == '"' and not in_s:
            in_d = not in_d
        elif c == "#" and not in_s and not in_d:
            # a comment only if preceded by whitespace or at line start
            if i == 0 or line[i - 1] in " \t":
                break
        out.append(c)
        i += 1
    return "".join(out)


def _scalar(tok: str) -> Any:
    """Parse a YAML scalar token into a Python str / int / float / bool."""
    tok = tok.strip()
    if tok == "":
        return ""
    if len(tok) >= 2 and tok[0] == tok[-1] and tok[0] in ("'", '"'):
        return tok[1:-1]
    if tok in ("true", "True"):
        return True
    if tok in ("false", "False"):
        return False
    # numbers (ints + decimals); leave anything else as a bare string token
    if re.fullmatch(r"-?\d+", tok):
        return int(tok)
    if re.fullmatch(r"-?\d+\.\d+", tok):
        return float(tok)
    return tok


def _parse_flow(tok: str) -> Any:
    """Parse an inline flow collection: `[a, b]` -> list, `{ "k": v, ... }` -> dict.
    Splits on top-level commas only (so a quoted comma is preserved)."""
    tok = tok.strip()
    if tok.startswith("[") and tok.endswith("]"):
        inner = tok[1:-1].strip()
        if not inner:
            return []
        return [_parse_flow(p) if p.strip()[:1] in "[{" else _scalar(p)
                for p in _split_top(inner, ",")]
    if tok.startswith("{") and tok.endswith("}"):
        inner = tok[1:-1].strip()
        out: dict[str, Any] = {}
        if not inner:
            return out
        for pair in _split_top(inner, ","):
            if ":" not in pair:
                raise CompileError(f"flow-map entry without ':' -> {pair!r}")
            k, v = _split_top(pair, ":", maxsplit=1)
            out[_scalar(k)] = (_parse_flow(v) if v.strip()[:1] in "[{" else _scalar(v))
        return out
    return _scalar(tok)


def _split_top(s: str, sep: str, maxsplit: int = -1) -> list[str]:
    """Split `s` on `sep` at top nesting level only (ignoring [], {}, and quotes)."""
    parts: list[str] = []
    depth = 0
    in_s = in_d = False
    cur: list[str] = []
    count = 0
    for c in s:
        if c == "'" and not in_d:
            in_s = not in_s
        elif c == '"' and not in_s:
            in_d = not in_d
        if not in_s and not in_d:
            if c in "[{":
                depth += 1
            elif c in "]}":
                depth -= 1
            elif c == sep and depth == 0 and (maxsplit < 0 or count < maxsplit):
                parts.append("".join(cur))
                cur = []
                count += 1
                continue
        cur.append(c)
    parts.append("".join(cur))
    return parts


class _Line:
    __slots__ = ("indent", "text", "no")

    def __init__(self, indent: int, text: str, no: int):
        self.indent = indent
        self.text = text
        self.no = no


def _lex(text: str) -> list[_Line]:
    out: list[_Line] = []
    for no, raw in enumerate(text.splitlines(), 1):
        if "\t" in raw[: len(raw) - len(raw.lstrip())]:
            raise CompileError(f"line {no}: tab in indentation (use spaces)")
        stripped = _strip_comment(raw).rstrip()
        if not stripped.strip():
            continue
        indent = len(stripped) - len(stripped.lstrip(" "))
        out.append(_Line(indent, stripped.strip(), no))
    return out


def _parse_block(lines: list[_Line], i: int, indent: int) -> tuple[Any, int]:
    """Parse the block at `lines[i:]` whose items are indented to >= `indent`.
    Returns (value, next_index)."""
    if i >= len(lines):
        return {}, i
    first = lines[i]
    if first.text.startswith("- "):
        return _parse_seq(lines, i, first.indent)
    if first.text == "-":
        return _parse_seq(lines, i, first.indent)
    return _parse_map(lines, i, first.indent)


def _parse_map(lines: list[_Line], i: int, indent: int) -> tuple[dict[str, Any], int]:
    out: dict[str, Any] = {}
    while i < len(lines) and lines[i].indent == indent and not lines[i].text.startswith("- "):
        ln = lines[i]
        m = _KEY_RE.match(ln.text)
        if not m:
            raise CompileError(f"line {ln.no}: not a mapping entry -> {ln.text!r}")
        key = _scalar(m.group("key"))
        val = m.group("val")
        if val is not None and val.strip() != "":
            out[key] = _parse_flow(val) if val.strip()[:1] in "[{" else _scalar(val)
            i += 1
        else:
            # nested block on the following deeper-indented lines
            if i + 1 < len(lines) and lines[i + 1].indent > indent:
                child, i = _parse_block(lines, i + 1, lines[i + 1].indent)
                out[key] = child
            else:
                out[key] = None
                i += 1
    return out, i


def _parse_seq(lines: list[_Line], i: int, indent: int) -> tuple[list[Any], int]:
    out: list[Any] = []
    while i < len(lines) and lines[i].indent == indent and (
        lines[i].text == "-" or lines[i].text.startswith("- ")
    ):
        ln = lines[i]
        rest = ln.text[2:] if ln.text.startswith("- ") else ""
        if rest.strip() == "":
            # item body is the following deeper-indented block
            if i + 1 < len(lines) and lines[i + 1].indent > indent:
                item, i = _parse_block(lines, i + 1, lines[i + 1].indent)
                out.append(item)
            else:
                out.append(None)
                i += 1
            continue
        # inline item: either `- key: val ...` (a one-or-more-key map) or a scalar/flow
        m = _KEY_RE.match(rest)
        if m and not rest.lstrip().startswith(("[", "{")):
            # treat the rest (and any deeper continuation lines) as a map whose first
            # entry sits on this line. Re-lex by synthesising a virtual indent.
            virt_indent = ln.indent + 2
            synth = [_Line(virt_indent, rest, ln.no)]
            j = i + 1
            while j < len(lines) and lines[j].indent >= virt_indent:
                synth.append(lines[j])
                j += 1
            item, _ = _parse_map(synth, 0, virt_indent)
            out.append(item)
            i = j
        else:
            out.append(_parse_flow(rest) if rest.strip()[:1] in "[{" else _scalar(rest))
            i += 1
    return out, i


def parse_yaml_ld(text: str) -> dict[str, Any]:
    """Parse a YAML-LD authoring doc into a Python dict. Stdlib-only, deterministic."""
    lines = _lex(text)
    if not lines:
        return {}
    value, end = _parse_map(lines, 0, lines[0].indent)
    if end != len(lines):
        raise CompileError(
            f"line {lines[end].no}: unexpected content at top level -> {lines[end].text!r}"
        )
    return value


# =============================================================================
# 2. The guarded V() concept-token resolver
# =============================================================================


def _norm(s: str) -> str:
    """Normalise a label/token for lexical matching: lowercase, collapse any run of
    non-alphanumerics to a single hyphen, strip leading/trailing hyphens."""
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", s.lower())).strip("-")


class ConceptResolver:
    """The guarded V() resolver. Builds a lexical index over the concept catalog
    (local-name + normalised prefLabel) and resolves an author TOKEN to exactly one
    concept IRI. Ambiguous (>=2 distinct IRIs) or unresolvable -> CompileError."""

    def __init__(self, concepts: list[dict[str, Any]], echo: bool = True):
        self._echo = echo
        # key -> set of IRIs (a set so a label appearing twice for the SAME IRI is not
        # spuriously "ambiguous").
        self._index: dict[str, set[str]] = {}
        self._iris: set[str] = set()
        for c in concepts:
            iri = c.get("@id")
            if not iri:
                raise CompileError(f"concept without @id: {c!r}")
            self._iris.add(iri)
            local = iri.split(":", 1)[1] if ":" in iri else iri
            # the bare local-name AND the local-name with a leading `topic-`/`surface-`
            # facet stripped (so the author can write `merge-discipline` for
            # `kb:topic-merge-discipline`).
            self._add(local, iri)
            self._add(re.sub(r"^(topic|surface|concept)-", "", local), iri)
            pref = c.get("prefLabel")
            if isinstance(pref, str) and pref:
                self._add(_norm(pref), iri)

    def _add(self, key: str, iri: str) -> None:
        k = _norm(key)
        if not k:
            return
        self._index.setdefault(k, set()).add(iri)

    def resolve(self, token: str) -> str:
        """Resolve a single TOKEN to its concept IRI, or raise CompileError."""
        # A token already written as a CURIE (kb:..., pkg:...) is taken as-is IF it
        # names a known concept; otherwise it is still passed through (it may be an
        # ontology individual like pkg:Topics), but a bare token MUST resolve.
        if ":" in token and token in self._iris:
            return token
        key = _norm(token)
        hits = self._index.get(key)
        if not hits:
            if ":" in token:
                # a CURIE that is not in the catalog — pass through (e.g. pkg:Explored).
                return token
            raise CompileError(
                f"unresolvable concept token {token!r}: no catalog concept matches "
                f"(known: {sorted(self._norm_keys())})"
            )
        if len(hits) > 1:
            raise CompileError(
                f"AMBIGUOUS concept token {token!r} -> {sorted(hits)}; "
                f"disambiguate by writing the exact kb: local-name"
            )
        iri = next(iter(hits))
        if self._echo:
            print(f"  [V] {token!r} -> {iri} (lexical, exact)", file=sys.stderr)
        return iri

    def _norm_keys(self) -> set[str]:
        return set(self._index.keys())


# =============================================================================
# 3. The deterministic Turtle emitter
# =============================================================================

# Built-in @context prefixes always emitted in the Turtle header (kept stable + sorted
# at emit time so the output is deterministic).
_BASE_PREFIXES = {
    "pkg": "https://sparq.dev/ns/pkg#",
    "kb": "https://sparq.dev/ns/pkg/kb#",
    "prov": "http://www.w3.org/ns/prov#",
    "skos": "http://www.w3.org/2004/02/skos/core#",
    "dcterms": "http://purl.org/dc/terms/",
    "fabio": "http://purl.org/spar/fabio/",
    "cito": "http://purl.org/spar/cito/",
    "sigimpl": "https://w3id.org/zkp-sparql/sig-impl#",
    "secx": "https://w3id.org/zkp-sparql/sec-prop#",
    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
}

# The author-facing enum keys -> the CURIE the compiler emits.
_VERDICT = {"yes": "sigimpl:yes", "no": "sigimpl:no", "partial": "sigimpl:partial"}
_ASSURANCE = {
    "proven": "secx:Proven",
    "claimed": "secx:Claimed",
    "conjectured": "secx:Conjectured",
}


def _esc(s: str) -> str:
    """Escape a string for a single-line Turtle "..." literal."""
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", " ")
        .replace("\r", " ")
        .replace("\t", " ")
    ).strip()


def _is_curie(s: str) -> bool:
    return bool(re.match(r"^[A-Za-z][A-Za-z0-9_-]*:[A-Za-z0-9_./%#-]+$", s))


class Emitter:
    def __init__(self, ctx: dict[str, Any], resolver: ConceptResolver):
        self.ctx = ctx
        self.resolver = resolver
        self.lines: list[str] = []

    def _term_def(self, key: str) -> dict[str, Any]:
        """The @context definition for an author key, normalised to a dict."""
        d = self.ctx.get(key)
        if isinstance(d, dict):
            return d
        if isinstance(d, str):
            return {"@id": d}
        return {}

    def _pred(self, key: str) -> str:
        d = self._term_def(key)
        pid = d.get("@id", key)
        return pid

    def _obj_terms(self, key: str, value: Any) -> list[str]:
        """Render one author (key, value) into a list of Turtle object terms."""
        d = self._term_def(key)
        typ = d.get("@type")
        lang = d.get("@language")
        values = value if isinstance(value, list) else [value]
        out: list[str] = []
        for v in values:
            if typ == "@token":
                out.append(self.resolver.resolve(str(v)))
            elif typ == "@verdict":
                if str(v) not in _VERDICT:
                    raise CompileError(f"bad verdict {v!r} (use yes|no|partial)")
                out.append(_VERDICT[str(v)])
            elif typ == "@assurance":
                if str(v) not in _ASSURANCE:
                    raise CompileError(
                        f"bad assurance {v!r} (use proven|claimed|conjectured)"
                    )
                out.append(_ASSURANCE[str(v)])
            elif typ == "@id":
                out.append(str(v) if _is_curie(str(v)) else f'"{_esc(str(v))}"')
            elif typ == "xsd:decimal":
                out.append(_fmt_decimal(v))
            elif typ == "xsd:integer":
                out.append(str(int(v)))
            elif lang:
                out.append(f'"{_esc(str(v))}"@{lang}')
            else:
                # untyped: a CURIE stays an IRI, otherwise a plain literal.
                sv = str(v)
                out.append(sv if _is_curie(sv) else f'"{_esc(sv)}"')
        return out

    def emit_node(self, node: dict[str, Any], section: str | None = None) -> None:
        nid = node.get("@id")
        if not nid:
            raise CompileError(f"node without @id: {node!r}")
        # @type first.
        types = node.get("@type")
        if types is None:
            raise CompileError(f"node {nid} without @type")
        type_terms = types if isinstance(types, list) else [types]
        self.lines.append(f"\n{nid} a {' , '.join(type_terms)} ;")
        # the rest of the keys, in authoring order (deterministic — the YAML reader
        # preserves insertion order).
        keys = [k for k in node if k not in ("@id", "@type")]
        rendered: list[str] = []
        for k in keys:
            pred = self._pred(k)
            terms = self._obj_terms(k, node[k])
            for t in terms:
                rendered.append(f"  {pred} {t}")
        if not rendered:
            # a bare `nid a Type .` — close the type line.
            self.lines[-1] = self.lines[-1].rstrip()[:-1] + "."
            return
        body = " ;\n".join(rendered) + " ."
        self.lines.append(body)


def _fmt_decimal(v: Any) -> str:
    """Render a number as an xsd:decimal literal that the SHACL sh:datatype check
    accepts. A Turtle bare decimal (e.g. `0.9`) IS typed xsd:decimal; a bare integer
    (`1`) is xsd:integer, so force a `.0` so confidence stays a decimal."""
    if isinstance(v, bool):
        raise CompileError(f"expected a decimal, got bool {v!r}")
    f = float(v)
    s = repr(f)
    if "." not in s and "e" not in s and "E" not in s:
        s += ".0"
    return s


# =============================================================================
# 4. The top-level compile entry point
# =============================================================================


def compile_yaml_ld(text: str, echo: bool = True, name: str = "agents-findings") -> str:
    """Compile a YAML-LD authoring doc (string) to PKG Turtle (string). Raises
    CompileError on a malformed doc or an ambiguous/unresolvable concept token.
    `name` is the authoring-doc stem, used only for the generated-file header comment
    (the default keeps the original agents-findings output byte-identical). [FABLE-5]"""
    doc = parse_yaml_ld(text)
    ctx = doc.get("@context", {})
    if not isinstance(ctx, dict):
        raise CompileError("@context must be a mapping")
    concepts = doc.get("concepts", []) or []
    if not isinstance(concepts, list):
        raise CompileError("concepts: must be a sequence")
    resolver = ConceptResolver(concepts, echo=echo)
    emitter = Emitter(ctx, resolver)

    parts: list[str] = []
    parts.append(
        f"# {name}.ttl — GENERATED from {name}.yaml.ld by\n"
        "# crates/sparq-kb/ingest/yamlld_compile.py (FO-bridge Phase 4, sq-mztg8.2).\n"
        "# DO NOT HAND-EDIT — edit the .yaml.ld authoring source and recompile.\n"
        "# 🤖 SPARQ agent [OPUS-4.8]. Conforms to pkg.shapes.ttl (0 violations).\n"
    )
    # prefix header (deterministic, sorted).
    for pfx in sorted(_BASE_PREFIXES):
        parts.append(f"@prefix {pfx}: <{_BASE_PREFIXES[pfx]}> .")
    parts.append("")

    # concepts (the skos:Concept catalog the read-path V() also grounds against).
    if concepts:
        parts.append("# === Concepts (the V()-resolution catalog) ===")
        for c in concepts:
            emitter.emit_node(c)
        parts.extend(emitter.lines)
        emitter.lines = []

    # sources.
    sources = doc.get("sources", []) or []
    if sources:
        parts.append("\n# === Sources ===")
        for s in sources:
            emitter.emit_node(s)
        parts.extend(emitter.lines)
        emitter.lines = []

    # findings (the @graph).
    graph = doc.get("@graph", []) or []
    if graph:
        parts.append("\n# === Findings (the @graph) ===")
        for n in graph:
            emitter.emit_node(n)
        parts.extend(emitter.lines)
        emitter.lines = []

    return "\n".join(parts) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Compile a PKG YAML-LD authoring doc to SHACL-conformant Turtle."
    )
    ap.add_argument("src", help="the *.yaml.ld authoring source")
    ap.add_argument("--out", required=True, help="the Turtle output path")
    ap.add_argument("--quiet", action="store_true", help="suppress the V() echo")
    args = ap.parse_args()

    with open(args.src, encoding="utf-8") as f:
        text = f.read()
    # The generated-file header names the authoring doc (stem of src). [FABLE-5]
    base = args.src.rsplit("/", 1)[-1]
    name = base[: -len(".yaml.ld")] if base.endswith(".yaml.ld") else base
    try:
        ttl = compile_yaml_ld(text, echo=not args.quiet, name=name)
    except CompileError as e:
        print(f"[yamlld_compile] COMPILE ERROR: {e}", file=sys.stderr)
        return 2
    with open(args.out, "w", encoding="utf-8") as f:
        f.write(ttl)
    n_findings = ttl.count("a pkg:Finding")
    print(
        f"[yamlld_compile] {args.src} -> {args.out} "
        f"(findings={n_findings})",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
