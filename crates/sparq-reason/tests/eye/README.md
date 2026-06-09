# EYE differential test cases (N3 parity gate)

Each `<case>.n3` is an N3 document; `<case>-answer.n3` is the expected entailed triples. The
`eye_cases.rs` harness runs sparq-reason's forward-chaining closure on the input and asserts
it contains every answer triple (EYE answers are often query-projected; our closure is a
superset).

- `socrates.*` — vendored verbatim from [eyereasoner/eye](https://github.com/eyereasoner/eye)
  `reasoning/socrates` (a forward `subClassOf` rule).
- `math-sum.*` — EYE `math:sum` builtin semantics over a forward rule.

As the reasoner gains features (backward chaining `<=`, path syntax `!`/`^`, more `math:`/
`string:`/`list:`/`log:` builtins), copy the matching EYE `reasoning/<case>` input+answer
here and add a `check!` in `eye_cases.rs`. Each passing case is a parity checkpoint.
