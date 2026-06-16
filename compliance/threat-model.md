<!-- [OPUS-4.8] sq-toze — SHARED compliance threat-model pointer (owned by the Privacy
     framework engineer). References research/threat-model.md (the STRIDE source of truth);
     does NOT fork it. Adds the privacy/data-protection lens. Re-review when Fable returns. -->

# Threat model (compliance lens)

This is a **pointer + privacy lens**, **not** a second threat model. The authoritative STRIDE
threat model for the sparq STABLE core is **[`../research/threat-model.md`](../research/threat-model.md)**
(STRIDE per trust boundary B1–B5, with cited mitigations, tests, and beads). The compliance
frameworks **reference** it; this file maps its boundaries/threats onto the
**privacy/data-protection** concerns the certification work cares about, and points each framework
at the relevant section. **Do not duplicate the STRIDE content here** — read the source.

## The five trust boundaries (from `research/threat-model.md`)

| Boundary | What it is | Privacy/data-protection relevance | Compliance cross-ref |
|---|---|---|---|
| **B1** | untrusted RDF bytes → parser | The personal data *enters* here; parser robustness is an availability + integrity concern, not a disclosure one | `data-flow.md` §1 (ingress); memsafety (B1 fuzz) |
| **B2** | untrusted SPARQL string → planner → executor | Query text can embed personal data (a `FILTER` literal); availability via `QueryBudget` | `privacy/controls.md` P-14; `data-flow.md` §1 |
| **B3** | untrusted HTTP client → server (**no per-user auth by design**) | **The key access-control boundary** — `T-HTTP-EoP` (open read+write) and `T-HTTP-INFO` (error disclosure) are the privacy-load-bearing threats | `privacy/controls.md` P-10/P-12; `dpia.md` B2/B3 |
| **B4** | engine → remote SPARQL endpoint (`SERVICE`) | **Exfiltration / SSRF** — `T-SERVICE-SSRF`; deny-all default is the privacy control | `privacy/controls.md` P-13; `dpia.md` B5 |
| **B5** | hostile on-disk index → unsafe mmap loader | Memory-safety of the persisted (possibly personal) dataset on reload; highest-severity *security* asset | `compliance/memsafety/` (owns B5) |

## Privacy-relevant threats (map to the STRIDE source)

| STRIDE threat (in `research/threat-model.md`) | Privacy framing | Engine mitigation status (per the source + `privacy/evidence.md`) |
|---|---|---|
| **T-HTTP-EoP** (open read+write, no per-user auth) | Unauthorised access to / modification of personal data | **Mitigated to operator-config:** optional constant-time Bearer auth (sq-zcby, done) + bind-posture refusal (sq-o4qf, done) + WS gate (sq-cxk5); fine-grained authz via `sparq-solid` (fail-closed). The bare server's no-auth is the documented **B3 decision** ("front with a gateway / `sparq-solid`"), not a silent gap. |
| **T-HTTP-INFO** (error-message disclosure) | Personal data / internals leaked in error responses | **Largely mitigated:** generic structured error bodies (`privacy/controls.md` P-12). Residual: `{:?}` algebra redaction (threat-model item 6, sq-ebii) + UPDATE parse-error fragment (`privacy` PR-G1, sq-toze.32). |
| **T-SERVICE-SSRF** (SSRF / exfiltration via `SERVICE`) | Personal data exfiltrated to an attacker endpoint; internal pivot | **Mitigated:** `SERVICE` **deny-all by default** + explicit allowlist (`privacy/controls.md` P-13). |
| **T-HTTP-DoS** (request flood / pathological query) | Availability denial of access to personal data | **Mitigated to operator-config:** `QueryBudget`, body-size + concurrency caps (`privacy/controls.md` P-14). |
| **T-MMAP-*** (B5 mmap UB/DoS) | Memory-safety of the persisted personal dataset on reload | **Owned by `compliance/memsafety/`** — register + Miri + oracle + fuzz over B5. Not re-assessed here. |

## Privacy-specific additions (not in the security threat model)

The security threat model is about confidentiality/integrity/availability of the *system*. The
**privacy** lens adds two data-lifecycle threats the security model does not foreground — both
tracked in `privacy/gap-register.md`:

- **TP-1 — Incomplete erasure** (a `DELETE`d triple survives in the `--persist` WAL until
  rotation). *Not a security breach* — a data-lifecycle/storage-limitation gap. → PR-G3 (sq-toze.33).
- **TP-2 — Incidental over-collection in logs** (query text captured when `--verbose` is on). →
  PR-G4 (sq-toze.34). Default-off keeps the residual low.

## ZK/MPC

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
The ZK estate has its **own** adversarial threat model and audit: the v1 verifier was originally found
**unsound** (`research/zk-soundness-audit.md`); `sq-1s2` landed the binding layer and the **internal,
single-model** re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found all findings closed
("sound as landed for the assumed threat model"), but it is **NOT externally audited** (`sq-qhy4`, P0,
pending) and carries **no production guarantee**; `sparq-mpc` is a deferred, semi-honest-only
scaffold. Neither is claimed as a privacy or security control anywhere in the compliance set — see
`privacy/controls.md` §carve-out and bead **sq-toze.35**.

---

**Source of truth:** [`../research/threat-model.md`](../research/threat-model.md). If a threat,
boundary, or mitigation changes, update **that** file; this pointer only re-frames it for the
privacy frameworks.
