//! U3 — Financial services generator (bead `sq-i6du2.4`).
//!
//! Generates a dataset modelling a financial institution: clients, accounts,
//! transactions, advisory documents, auditors, and regulators with strict
//! compartmentalization, high policy fan-in per resource, ODRL duties/constraints
//! (retention windows, purpose-of-use, count-limited access), and audit-trail reads.
//!
//! # AC shape stressed
//! - Strict compartmentalization: low public-audience mix (private + shared dominate).
//! - High policy fan-in: `policies_per_resource` drives ODRL policy accrual.
//! - ODRL duties/constraints (levels 1–3 per [`crate::ConstraintComplexity`]):
//!   constraint-bearing intents are ODRL-only in the expressibility matrix — WAC and
//!   ACP emit nothing for them (return [`crate::Expressibility::Unsupported`]).
//! - Audit-trail: read-only access for auditor/regulator agents with purpose constraints.
//!
//! # Expressibility matrix entries (documented asymmetries per design record §2.2)
//! - Any intent with `condition ≠ None` → WAC: Unsupported, ACP: Unsupported, ODRL: Native.
//! - `Audience::Group` without expansion → ACP: Expansion(0), WAC: Native (via agentGroup).
//!
//! # File ownership
//! **Only bead `sq-i6du2.4` edits this file.**

use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::{
    compile_acp, compile_odrl, compile_wac, oracle_acp, oracle_odrl, oracle_wac, AcModel,
    AccessMode, Audience, CompiledPolicy, Condition, ConstraintComplexity, Decision, Effect,
    ExpectedDecision, GenParams, IntentRow, QueryClass, QueryFixture, Request, Scope,
};

/// Output of the U3 financial-services generator.
///
/// All fields are deterministic for a given [`GenParams`] (same seed, same output).
pub struct FinancialDataset {
    /// N-Quads lines forming the data graph (accounts, transactions, advisory docs).
    pub data_nquads: Vec<String>,
    /// Compiled WAC policy graph.
    pub wac_policy: Vec<CompiledPolicy>,
    /// Compiled ACP policy graph.
    pub acp_policy: Vec<CompiledPolicy>,
    /// Compiled ODRL policy graph.
    pub odrl_policy: Vec<CompiledPolicy>,
    /// Model-agnostic intent table (one row per access-control decision point).
    pub intents: Vec<IntentRow>,
    /// Request tuples with expected decisions for W1 / W3 oracle checking.
    pub expected_decisions: Vec<ExpectedDecision>,
    /// W2 SPARQL queries with expected result sets (closed-form).
    pub queries: Vec<QueryFixture>,
}

// ── Namespace constants ──────────────────────────────────────────────────────────────

const BASE: &str = "https://bench.sparq.dev/financial/";
const PURPOSE_AUDIT: &str = "https://bench.sparq.dev/vocab/purpose/audit";
const PURPOSE_RISK: &str = "https://bench.sparq.dev/vocab/purpose/risk-assessment";
const PURPOSE_REPORTING: &str = "https://bench.sparq.dev/vocab/purpose/regulatory-reporting";

// ── Internal helpers ─────────────────────────────────────────────────────────────────

/// Client account container URI (a client's root in the institution).
fn client_uri(client_id: u32) -> String {
    format!("{BASE}client/{client_id}/")
}

/// Account resource URI nested under a client.
fn account_uri(client_id: u32, account_id: u32) -> String {
    format!("{BASE}client/{client_id}/account/{account_id}")
}

/// Transaction resource URI nested under an account.
fn txn_uri(client_id: u32, account_id: u32, txn_id: u32) -> String {
    format!("{BASE}client/{client_id}/account/{account_id}/txn/{txn_id}")
}

/// Advisory document URI (institution-level, not per-client).
fn advisory_uri(doc_id: u32) -> String {
    format!("{BASE}advisory/{doc_id}")
}

/// Agent URI for a client's primary contact agent.
fn client_agent_uri(client_id: u32) -> String {
    format!("{BASE}agent/client/{client_id}")
}

/// Auditor agent URI.
fn auditor_agent_uri(auditor_id: u32) -> String {
    format!("{BASE}agent/auditor/{auditor_id}")
}

/// Regulator agent URI.
fn regulator_agent_uri(reg_id: u32) -> String {
    format!("{BASE}agent/regulator/{reg_id}")
}

/// Advisor agent URI (relationship manager for multiple clients).
fn advisor_agent_uri(advisor_id: u32) -> String {
    format!("{BASE}agent/advisor/{advisor_id}")
}

/// Group URI for a client's advisory group.
fn advisory_group_uri(advisor_id: u32) -> String {
    format!("{BASE}group/advisory/{advisor_id}")
}

// ── Data-graph generation ────────────────────────────────────────────────────────────

/// Emit N-Quads for one client: container + accounts + transactions.
///
/// Uses a fixed `schema:` vocabulary so the output is self-contained N-Quads
/// without an external prefix resolution step.
fn emit_client_nquads(
    nquads: &mut Vec<String>,
    client_id: u32,
    n_accounts: u32,
    n_txns_per_account: u32,
) {
    let schema = "https://schema.org/";
    let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    let client_iri = client_uri(client_id);

    // Client container triple.
    nquads.push(format!(
        "<{client_iri}> <{rdf_type}> <{schema}Organization> ."
    ));
    nquads.push(format!(
        "<{client_iri}> <{schema}identifier> \"{client_id}\"^^<http://www.w3.org/2001/XMLSchema#integer> ."
    ));

    for account_id in 0..n_accounts {
        let acct_iri = account_uri(client_id, account_id);
        nquads.push(format!("<{acct_iri}> <{rdf_type}> <{schema}BankAccount> ."));
        nquads.push(format!(
            "<{acct_iri}> <{schema}accountHolderName> <{client_iri}> ."
        ));
        nquads.push(format!("<{client_iri}> <{schema}owns> <{acct_iri}> ."));

        for txn_id in 0..n_txns_per_account {
            let txn_iri = txn_uri(client_id, account_id, txn_id);
            nquads.push(format!(
                "<{txn_iri}> <{rdf_type}> <{schema}MoneyTransfer> ."
            ));
            nquads.push(format!("<{txn_iri}> <{schema}accountId> <{acct_iri}> ."));
            nquads.push(format!(
                "<{txn_iri}> <{schema}amount> \"{txn_id}00\"^^<http://www.w3.org/2001/XMLSchema#decimal> ."
            ));
        }
    }
}

/// Emit N-Quads for advisory documents (institution-level).
fn emit_advisory_nquads(nquads: &mut Vec<String>, doc_id: u32) {
    let schema = "https://schema.org/";
    let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    let doc_iri = advisory_uri(doc_id);
    nquads.push(format!("<{doc_iri}> <{rdf_type}> <{schema}Report> ."));
    nquads.push(format!(
        "<{doc_iri}> <{schema}identifier> \"{doc_id}\"^^<http://www.w3.org/2001/XMLSchema#integer> ."
    ));
}

// ── Intent-table generation ──────────────────────────────────────────────────────────

/// Build the temporal window strings for a given index (deterministic, no wall-clock).
fn temporal_window(idx: u32) -> (String, String) {
    let year_start = 2020 + (idx / 4) % 10;
    let year_end = year_start + 2;
    (
        format!("{year_start}-01-01T00:00:00Z"),
        format!("{year_end}-01-01T00:00:00Z"),
    )
}

/// Choose a purpose IRI in a round-robin deterministic manner.
fn purpose_for(idx: u32) -> &'static str {
    match idx % 3 {
        0 => PURPOSE_AUDIT,
        1 => PURPOSE_RISK,
        _ => PURPOSE_REPORTING,
    }
}

/// Generate intents for one client's account container, accounts, and transactions.
///
/// Financial compartmentalization rules (by construction; no public access):
/// 1. The client's own agent has full read/write/control access to their container subtree
///    (WAC/ACP/ODRL native, `Condition::None`).
/// 2. The assigned advisor group has read-only subtree access (WAC/ACP/ODRL, no condition).
/// 3. Auditors have read-only access to each account, with a usage condition determined by
///    `constraint_complexity` (ODRL-only for levels ≥ Temporal; WAC/ACP: Unsupported).
/// 4. Regulators have read-only access to transactions with a retention window (ODRL-only
///    for levels ≥ Temporal; WAC/ACP: Unsupported).
/// 5. Extra policy fan-in intents per account exercise `policies_per_resource` (ODRL-only
///    for constrained levels, plain agent allows otherwise).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn intents_for_client(
    intents: &mut Vec<IntentRow>,
    client_id: u32,
    advisor_id: u32,
    auditor_id: u32,
    regulator_id: u32,
    n_accounts: u32,
    n_txns_per_account: u32,
    constraint_complexity: &ConstraintComplexity,
    policy_fan_in: u8,
    rng: &mut SmallRng,
) {
    let client_agent = client_agent_uri(client_id);
    let advisor_group = advisory_group_uri(advisor_id);
    let auditor = auditor_agent_uri(auditor_id);
    let regulator = regulator_agent_uri(regulator_id);
    let container = client_uri(client_id);

    // 1. Client owns their container: full subtree access, no condition.
    intents.push(IntentRow {
        audience: Audience::Agent(client_agent.clone()),
        scope: Scope::Subtree,
        mode: AccessMode::full(),
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: container.clone(),
    });

    // 2. Advisor group: read-only subtree access, no condition (WAC/ACP/ODRL native).
    intents.push(IntentRow {
        audience: Audience::Group(advisor_group.clone()),
        scope: Scope::Subtree,
        mode: AccessMode::read_only(),
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: container.clone(),
    });

    // 3. Auditor: read-only access to each account, with constraint depending on level.
    for account_id in 0..n_accounts {
        let acct_uri = account_uri(client_id, account_id);
        let auditor_condition = match constraint_complexity {
            ConstraintComplexity::None => Condition::None,
            ConstraintComplexity::Temporal => {
                let (start, end) = temporal_window(account_id);
                Condition::Temporal { start, end }
            }
            ConstraintComplexity::PurposeOrCount => {
                Condition::Purpose(purpose_for(account_id).to_owned())
            }
            ConstraintComplexity::Compound => {
                let (start, end) = temporal_window(account_id);
                Condition::And(
                    Box::new(Condition::Temporal { start, end }),
                    Box::new(Condition::Purpose(purpose_for(account_id).to_owned())),
                )
            }
        };

        intents.push(IntentRow {
            audience: Audience::Agent(auditor.clone()),
            scope: Scope::Resource,
            mode: AccessMode::read_only(),
            condition: auditor_condition,
            effect: Effect::Allow,
            resource_uri: acct_uri.clone(),
        });

        // 4. Regulator: read-only access to transactions with retention window constraint.
        for txn_id in 0..n_txns_per_account {
            let txn_resource = txn_uri(client_id, account_id, txn_id);
            let reg_condition = match constraint_complexity {
                ConstraintComplexity::None => Condition::None,
                ConstraintComplexity::Temporal => {
                    // Retention window: transactions accessible for a fixed deterministic window.
                    let year_start = 2019 + (txn_id % 5);
                    let year_end = year_start + 7;
                    Condition::Temporal {
                        start: format!("{year_start}-01-01T00:00:00Z"),
                        end: format!("{year_end}-01-01T00:00:00Z"),
                    }
                }
                ConstraintComplexity::PurposeOrCount => {
                    // Count-limited access (audit reads allowed up to a fixed count).
                    Condition::Count(5 + (txn_id % 10))
                }
                ConstraintComplexity::Compound => {
                    let year_start = 2019 + (txn_id % 5);
                    let year_end = year_start + 7;
                    Condition::And(
                        Box::new(Condition::Temporal {
                            start: format!("{year_start}-01-01T00:00:00Z"),
                            end: format!("{year_end}-01-01T00:00:00Z"),
                        }),
                        Box::new(Condition::Purpose(PURPOSE_REPORTING.to_owned())),
                    )
                }
            };

            intents.push(IntentRow {
                audience: Audience::Agent(regulator.clone()),
                scope: Scope::Resource,
                mode: AccessMode::read_only(),
                condition: reg_condition,
                effect: Effect::Allow,
                resource_uri: txn_resource,
            });
        }

        // 5. Extra policy fan-in: additional purpose-constrained intents per account.
        //    These are ODRL-only for any constraint level ≥ Temporal (WAC/ACP: Unsupported).
        for extra in 1..policy_fan_in {
            let extra_condition = match constraint_complexity {
                ConstraintComplexity::None => Condition::None,
                _ => Condition::Purpose(purpose_for(u32::from(extra)).to_owned()),
            };
            let extra_advisor = advisor_agent_uri(advisor_id + u32::from(extra));
            intents.push(IntentRow {
                audience: Audience::Agent(extra_advisor),
                scope: Scope::Resource,
                mode: AccessMode::read_only(),
                condition: extra_condition,
                effect: Effect::Allow,
                resource_uri: acct_uri.clone(),
            });
            // Advance the PRNG so different policy_fan_in values produce divergent streams.
            let _: u32 = rng.gen();
        }
    }

    // Advance RNG to isolate client streams deterministically.
    let _: u32 = rng.gen();
}

/// Generate intents for institution-level advisory documents.
///
/// Advisory docs are accessible only to assigned advisors, with usage constraints
/// determined by `constraint_complexity` (ODRL-only for levels ≥ Temporal).
fn intents_for_advisory(
    intents: &mut Vec<IntentRow>,
    doc_id: u32,
    advisor_id: u32,
    constraint_complexity: &ConstraintComplexity,
) {
    let advisor = advisor_agent_uri(advisor_id);
    let doc = advisory_uri(doc_id);

    let read_condition = match constraint_complexity {
        ConstraintComplexity::None => Condition::None,
        ConstraintComplexity::Temporal => {
            let (start, end) = temporal_window(doc_id);
            Condition::Temporal { start, end }
        }
        ConstraintComplexity::PurposeOrCount => Condition::Purpose(purpose_for(doc_id).to_owned()),
        ConstraintComplexity::Compound => {
            let (start, end) = temporal_window(doc_id);
            Condition::And(
                Box::new(Condition::Temporal { start, end }),
                Box::new(Condition::Purpose(purpose_for(doc_id).to_owned())),
            )
        }
    };

    // Read with potential constraint.
    intents.push(IntentRow {
        audience: Audience::Agent(advisor.clone()),
        scope: Scope::Resource,
        mode: AccessMode::read_only(),
        condition: read_condition,
        effect: Effect::Allow,
        resource_uri: doc.clone(),
    });

    // Write access always unconstrained (advisors own their documents).
    intents.push(IntentRow {
        audience: Audience::Agent(advisor),
        scope: Scope::Resource,
        mode: AccessMode {
            read: false,
            write: true,
            control: false,
        },
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: doc,
    });
}

// ── Request + oracle ground-truth generation ─────────────────────────────────────────

/// Append the three per-model expected decisions for one read request.
///
/// Every row of the ground-truth table is built the same way — one read `Request` for
/// `agent` against `resource`, evaluated by each of the three oracles in turn — so the
/// construction lives here once rather than once per request shape.
fn push_read_decisions(
    decisions: &mut Vec<ExpectedDecision>,
    agent: &str,
    resource: &str,
    intents: &[IntentRow],
) {
    for model in [AcModel::Wac, AcModel::Acp, AcModel::Odrl] {
        let req = Request {
            agent: agent.to_owned(),
            client: None,
            resource: resource.to_owned(),
            mode: AccessMode::read_only(),
        };
        let decision = match model {
            AcModel::Wac => oracle_wac(&req, intents),
            AcModel::Acp => oracle_acp(&req, intents),
            AcModel::Odrl => oracle_odrl(&req, intents),
        };
        decisions.push(ExpectedDecision {
            request: req,
            model,
            decision,
        });
    }
}

/// Build the expected-decisions table from the intent table for all three models.
///
/// Requests are constructed deterministically from the corpus structure:
/// - Per client: client's own agent reading their container (expect Allow — subtree intent).
/// - Per client: auditor reading first account (Allow/Deny depends on `constraint_complexity`
///   and oracle condition evaluation).
/// - Per client (if transactions exist): regulator reading first transaction.
/// - Cross-client: a client's agent reading another client's container (expect Deny).
/// - Unknown agent: completely unknown agent (expect Deny for all models — fail-closed).
fn build_expected_decisions(
    intents: &[IntentRow],
    client_ids: &[u32],
    auditor_id: u32,
    regulator_id: u32,
) -> Vec<ExpectedDecision> {
    let mut decisions = Vec::new();

    for &client_id in client_ids {
        let agent = client_agent_uri(client_id);
        let container = client_uri(client_id);

        // Client reads their own container root (subtree intent → Allow for all models).
        push_read_decisions(&mut decisions, &agent, &container, intents);

        // Auditor reads the first account of each client.
        let acct0 = account_uri(client_id, 0);
        let auditor_agent = auditor_agent_uri(auditor_id);
        push_read_decisions(&mut decisions, &auditor_agent, &acct0, intents);

        // Regulator reads the first transaction of the first account.
        let txn0 = txn_uri(client_id, 0, 0);
        let regulator_agent = regulator_agent_uri(regulator_id);
        push_read_decisions(&mut decisions, &regulator_agent, &txn0, intents);

        // Cross-client: client_id tries to read client_id+1's container → Deny.
        let next_id = client_id + 1;
        if client_ids.contains(&next_id) {
            let cross_resource = client_uri(next_id);
            push_read_decisions(&mut decisions, &agent, &cross_resource, intents);
        }
    }

    // Unknown agent requests a resource → Deny for all models (fail-closed proof point).
    let unknown_agent = format!("{BASE}agent/unknown/9999");
    let first_resource = account_uri(client_ids[0], 0);
    push_read_decisions(&mut decisions, &unknown_agent, &first_resource, intents);

    decisions
}

// ── W2 query fixtures ────────────────────────────────────────────────────────────────

/// Build W2 SPARQL query fixtures for the financial corpus (four query classes).
///
/// - Q-point: explicit GRAPH lookup for a known account.
/// - Q-scan: list all accounts under a client container.
/// - Q-join: cross-client join (advisor sees two clients' accounts, if ≥2 clients).
/// - Q-agg: COUNT of transactions in a client's first account.
///
/// Expected result rows are closed-form (derived from the data shape, not from
/// running any sparq evaluator).
#[allow(clippy::too_many_lines)]
fn build_queries(
    client_ids: &[u32],
    intents: &[IntentRow],
    n_accounts: u32,
    n_txns_per_account: u32,
) -> Vec<QueryFixture> {
    let mut queries = Vec::new();
    let schema = "https://schema.org/";

    if client_ids.is_empty() {
        return queries;
    }

    let c0 = client_ids[0];
    let client_agent = client_agent_uri(c0);

    // Q-point: client reads their first account's type and accountHolderName.
    let acct0 = account_uri(c0, 0);
    {
        let sparql = format!("SELECT ?p ?o WHERE {{ GRAPH <{acct0}> {{ <{acct0}> ?p ?o }} }}");
        let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let client_cont = client_uri(c0);
        let expected_rows = vec![
            format!("<{rdf_type}> <{schema}BankAccount>"),
            format!("<{schema}accountHolderName> <{client_cont}>"),
        ];
        queries.push(QueryFixture {
            class: QueryClass::Point,
            sparql,
            expected_rows,
            agent: client_agent.clone(),
            model: AcModel::Odrl,
        });
    }

    // Q-scan: list all accounts for client c0.
    {
        let container = client_uri(c0);
        let sparql =
            format!("SELECT ?acct WHERE {{ ?acct <{schema}accountHolderName> <{container}> }}");
        let expected_rows: Vec<String> = (0..n_accounts)
            .map(|a| format!("?acct=<{}>", account_uri(c0, a)))
            .collect();
        queries.push(QueryFixture {
            class: QueryClass::Scan,
            sparql,
            expected_rows,
            agent: client_agent.clone(),
            model: AcModel::Odrl,
        });
    }

    // Q-join: advisor sees accounts of two clients (if ≥ 2 clients exist).
    if client_ids.len() >= 2 {
        let c1 = client_ids[1];
        let advisor = advisor_agent_uri(0);
        let sparql = format!(
            "SELECT ?acct WHERE {{ \
             {{ ?acct <{schema}accountHolderName> <{cid0}> }} \
             UNION \
             {{ ?acct <{schema}accountHolderName> <{cid1}> }} \
             }}",
            cid0 = client_uri(c0),
            cid1 = client_uri(c1),
        );
        // Expected rows: accounts for each client where the advisor's read request is allowed.
        // (Group semantics: the oracle uses prefix matching as a placeholder; see oracle.rs.)
        let req_c0 = Request {
            agent: advisor.clone(),
            client: None,
            resource: account_uri(c0, 0),
            mode: AccessMode::read_only(),
        };
        let req_c1 = Request {
            agent: advisor.clone(),
            client: None,
            resource: account_uri(c1, 0),
            mode: AccessMode::read_only(),
        };
        let dec_c0 = oracle_odrl(&req_c0, intents);
        let dec_c1 = oracle_odrl(&req_c1, intents);
        let mut expected_rows = Vec::new();
        if dec_c0 == Decision::Allow {
            for a in 0..n_accounts {
                expected_rows.push(format!("?acct=<{}>", account_uri(c0, a)));
            }
        }
        if dec_c1 == Decision::Allow {
            for a in 0..n_accounts {
                expected_rows.push(format!("?acct=<{}>", account_uri(c1, a)));
            }
        }
        queries.push(QueryFixture {
            class: QueryClass::Join,
            sparql,
            expected_rows,
            agent: advisor,
            model: AcModel::Odrl,
        });
    }

    // Q-agg: COUNT of transactions in client's first account.
    {
        let acct0_iri = account_uri(c0, 0);
        let sparql = format!(
            "SELECT (COUNT(?txn) AS ?count) WHERE {{ \
             ?txn <{schema}accountId> <{acct0_iri}> }}"
        );
        // Client owns all transactions in their account (full subtree access).
        let expected_rows = vec![format!("?count={n_txns_per_account}")];
        queries.push(QueryFixture {
            class: QueryClass::Aggregate,
            sparql,
            expected_rows,
            agent: client_agent,
            model: AcModel::Odrl,
        });
    }

    queries
}

// ── Main generator ───────────────────────────────────────────────────────────────────

/// Generate a U3 financial-services dataset.
///
/// # Invariants
/// - **Determinism**: same `params` → same `FinancialDataset` every call.
///   A [`SmallRng`] seeded from `params.seed` drives all randomised choices;
///   no wall-clock, OS entropy, or thread-local state is used.
/// - **Fail-closed oracle**: every `ExpectedDecision` with no matching allow rule
///   has `decision = Deny`. Verified by `tests/financial.rs::generate_u3_oracle_fail_closed`.
/// - **Independent oracle**: expected decisions are computed from the intent table
///   without calling any sparq evaluator (see [`crate::oracle`]).
/// - **Constraint intents are ODRL-only**: any [`IntentRow`] with `condition ≠ None`
///   produces [`crate::Expressibility::Unsupported`] from [`compile_wac`] and
///   [`compile_acp`]. Verified by `tests/financial.rs::generate_u3_constraint_intents_odrl_only`.
///
/// # Dataset shape (linear in `sf`)
/// - `sf` clients, each with 2 accounts and 3 transactions.
/// - `sf/2 + 1` institution-level advisory documents.
/// - 1 auditor, 1 regulator, `sf/2 + 1` advisors.
///
/// # Panics
/// Panics if `params.validate()` fails.
#[must_use]
pub fn generate(params: &GenParams) -> FinancialDataset {
    params
        .validate()
        .expect("GenParams must be valid (U3 financial generator)");

    let mut rng = SmallRng::seed_from_u64(params.seed);

    // Derived counts (all linear in sf).
    let n_clients = params.sf.max(1);
    let n_accounts_per_client: u32 = 2;
    let n_txns_per_account: u32 = 3;
    let n_advisors = (params.sf / 2 + 1).max(1);
    let n_advisory_docs = (params.sf / 2 + 1).max(1);
    let auditor_id: u32 = 0;
    let regulator_id: u32 = 0;

    // ── Data graph ───────────────────────────────────────────────────────────────────
    let mut data_nquads: Vec<String> = Vec::new();

    let client_ids: Vec<u32> = (0..n_clients).collect();

    for &client_id in &client_ids {
        emit_client_nquads(
            &mut data_nquads,
            client_id,
            n_accounts_per_client,
            n_txns_per_account,
        );
    }
    for doc_id in 0..n_advisory_docs {
        emit_advisory_nquads(&mut data_nquads, doc_id);
    }

    // ── Intent table ─────────────────────────────────────────────────────────────────
    let mut intents: Vec<IntentRow> = Vec::new();

    for &client_id in &client_ids {
        let advisor_id = client_id % n_advisors;
        intents_for_client(
            &mut intents,
            client_id,
            advisor_id,
            auditor_id,
            regulator_id,
            n_accounts_per_client,
            n_txns_per_account,
            &params.constraint_complexity,
            params.policies_per_resource,
            &mut rng,
        );
    }
    for doc_id in 0..n_advisory_docs {
        let advisor_id = doc_id % n_advisors;
        intents_for_advisory(
            &mut intents,
            doc_id,
            advisor_id,
            &params.constraint_complexity,
        );
    }

    // ── Per-model policy compilation ─────────────────────────────────────────────────
    let wac_policy: Vec<CompiledPolicy> = intents.iter().map(compile_wac).collect();
    let acp_policy: Vec<CompiledPolicy> = intents.iter().map(|r| compile_acp(r, &[])).collect();
    let odrl_policy: Vec<CompiledPolicy> = intents.iter().map(compile_odrl).collect();

    // ── Expected decisions ───────────────────────────────────────────────────────────
    let expected_decisions =
        build_expected_decisions(&intents, &client_ids, auditor_id, regulator_id);

    // ── W2 queries ───────────────────────────────────────────────────────────────────
    let queries = build_queries(
        &client_ids,
        &intents,
        n_accounts_per_client,
        n_txns_per_account,
    );

    FinancialDataset {
        data_nquads,
        wac_policy,
        acp_policy,
        odrl_policy,
        intents,
        expected_decisions,
        queries,
    }
}
