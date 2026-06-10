# EYE differential test cases (N3 parity gate)

Each `<case>.n3` is an N3 document; `<case>-answer.n3` is the expected entailed triples. The
`eye_cases.rs` harness runs sparq-reason's forward-chaining closure on the input and asserts
it contains every answer triple (EYE answers are often query-projected; our closure is a
superset). Cases EYE runs with `--query` concatenate the query file onto the input — the
query's `{goal} => {goal}` rule is an ordinary forward rule that also drives backward (`<=`)
proofs.

- `socrates.*` — vendored verbatim from [eyereasoner/eye](https://github.com/eyereasoner/eye)
  `reasoning/socrates` (a forward `subClassOf` rule).
- `math-sum.*` — EYE `math:sum` builtin semantics over a forward rule.
- `backward.n3` + `backward-query.n3` + `backward-answer.n3` — vendored verbatim from
  `reasoning/backward`: a `<=` rule with a pure-builtin premise (`?X math:greaterThan ?Y`),
  provable only goal-directed. The canonical witness that `<=` is NOT a reversed forward
  rule (a reversal derives nothing here — the builtin needs the goal's bindings).
- `witch.n3` + `witch-goal.n3` — vendored verbatim from `reasoning/witch` (chained forward
  rules + query). `witch-answer.n3` is written here: upstream ships a proof file, whose
  top-level lemma gives `:GIRL a :WITCH.`
- `bi-subset.n3` — EYE's own builtin unit-test suite (`reasoning/bi/biP.n3`) restricted to
  the builtins we implement; every test line is verbatim, the header documents each
  exclusion. `bi-subset-answer.n3` lists the `:<test> :result true` conclusion of each
  passing test (upstream `biA.n3` wraps the same results in `{ … } a :PASS` formulae).

As the reasoner gains features, copy the matching EYE `reasoning/<case>` input+answer here
and add a `check!` in `eye_cases.rs`. Each passing case is a parity checkpoint.
