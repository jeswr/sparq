// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! nargo + bb subprocess driver (v1: subprocess proving is acceptable per the
//! plan). One [`CircuitProver`] points at the `zk/compose/` workspace and a
//! scratch dir; it compiles a member, generates a witness from a Prover.toml,
//! produces a bb proof + vk, and verifies.
//!
//! File conventions (nargo 1.0.0-beta.21, bb 5.0.0-nightly.20260324):
//! - `nargo execute <name> --package P` writes `target/<name>.gz` (witness)
//!   and requires `target/P.json` (ACIR) from `nargo compile`.
//! - `bb prove -b acir.json -w witness.gz -o OUTDIR` writes `OUTDIR/proof`
//!   and `OUTDIR/public_inputs`.
//! - `bb write_vk -b acir.json -o OUTDIR` writes `OUTDIR/vk`.
//! - `bb verify -p proof -i public_inputs -k vk` exits 0 on success.

use crate::manifest::CircuitId;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Driver / proving error.
#[derive(Debug)]
pub enum DriverError {
    Spawn { tool: String, source: std::io::Error },
    Tool { tool: String, stderr: String },
    Io(std::io::Error),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::Spawn { tool, source } => {
                write!(f, "failed to spawn `{tool}`: {source}")
            }
            DriverError::Tool { tool, stderr } => {
                write!(f, "`{tool}` failed:\n{stderr}")
            }
            DriverError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<std::io::Error> for DriverError {
    fn from(e: std::io::Error) -> Self {
        DriverError::Io(e)
    }
}

/// A produced proof: bb proof bytes, its public-inputs bytes, and the vk.
#[derive(Debug, Clone)]
pub struct ProofArtifacts {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<u8>,
    pub vk: Vec<u8>,
}

/// Drives nargo/bb against the `zk/compose/` Noir workspace.
pub struct CircuitProver {
    /// Path to `zk/compose/` (the Nargo workspace root).
    compose_dir: PathBuf,
    /// bb verifier target (Noir circuits use `noir-recursive`).
    target: String,
}

impl CircuitProver {
    /// `compose_dir` is the `zk/compose/` workspace root.
    pub fn new(compose_dir: impl Into<PathBuf>) -> Self {
        CircuitProver {
            compose_dir: compose_dir.into(),
            target: "noir-recursive".to_string(),
        }
    }

    /// Locate `zk/compose/` relative to this crate (workspace layout).
    pub fn from_crate_root() -> Self {
        // crates/sparq-zk-compose -> ../../zk/compose
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let compose = here
            .parent()
            .and_then(Path::parent)
            .map(|root| root.join("zk").join("compose"))
            .expect("workspace layout");
        CircuitProver::new(compose)
    }

    fn package_dir(&self, id: &CircuitId) -> PathBuf {
        self.compose_dir.join(id.package())
    }

    fn target_dir(&self) -> PathBuf {
        self.compose_dir.join("target")
    }

    /// `nargo compile --package P`. Idempotent; produces `target/P.json`.
    pub fn compile(&self, id: &CircuitId) -> Result<PathBuf, DriverError> {
        let pkg = id.package();
        run(
            "nargo",
            Command::new("nargo")
                .arg("compile")
                .arg("--package")
                .arg(&pkg)
                .current_dir(&self.compose_dir),
        )?;
        let acir = self.target_dir().join(format!("{pkg}.json"));
        Ok(acir)
    }

    /// Write `Prover.toml` and run `nargo execute`, returning the witness
    /// path. This is the FAST path (no proving) the ignored tests use to
    /// exercise the relation cheaply.
    ///
    /// Concurrency: the per-package `Prover.toml` and the workspace
    /// `target/<pkg>_w.gz` witness are SHARED state. Two prove/witness calls
    /// against the SAME member (e.g. two `filter_int_d1` tests) running
    /// concurrently would overwrite each other's `Prover.toml` between write
    /// and `nargo execute`, and clobber each other's witness — proving (or
    /// failing) the WRONG statement. Callers that may run concurrently against
    /// the same member MUST pass a unique `tag` via [`Self::gen_witness_tagged`]
    /// (and [`Self::prove_in`]) so the prover-input toml and witness get unique,
    /// non-colliding names. This default wrapper uses the empty tag (the shared
    /// `Prover.toml` / `<pkg>_w.gz`) and is only safe single-threaded.
    pub fn gen_witness(&self, id: &CircuitId, prover_toml: &str) -> Result<PathBuf, DriverError> {
        self.gen_witness_tagged(id, prover_toml, "")
    }

    /// As [`Self::gen_witness`], but isolates the prover-input toml and the
    /// emitted witness under a unique `tag` so concurrent calls against the same
    /// member don't race on shared file paths. With a non-empty `tag` the input
    /// is written to `<pkg>/Prover_<tag>.toml` (selected via `nargo
    /// execute --prover-name`) and the witness to `target/<pkg>_w_<tag>.gz`.
    // [OPUS-4.8] tag-isolated witness path so the toolchain tests are safe under
    // default (parallel) `cargo test` — no shared Prover.toml/witness race
    // (roborev codex job 2180).
    pub fn gen_witness_tagged(
        &self,
        id: &CircuitId,
        prover_toml: &str,
        tag: &str,
    ) -> Result<PathBuf, DriverError> {
        let pkg = id.package();
        // Empty tag => legacy shared names; non-empty => per-call-unique names.
        let (prover_name, witness_name) = if tag.is_empty() {
            ("Prover".to_string(), format!("{pkg}_w"))
        } else {
            (format!("Prover_{tag}"), format!("{pkg}_w_{tag}"))
        };
        let toml_path = self.package_dir(id).join(format!("{prover_name}.toml"));
        std::fs::write(&toml_path, prover_toml)?;
        let witness_path = self.target_dir().join(format!("{witness_name}.gz"));
        // nargo execute exits 0 even on a failed assertion / bad input — it
        // signals failure by NOT writing the witness file (and printing to
        // stderr). So we delete any stale witness first and treat its absence
        // afterwards as the unsatisfiability signal.
        let _ = std::fs::remove_file(&witness_path);
        let out = Command::new("nargo")
            .arg("execute")
            .arg(&witness_name)
            .arg("--package")
            .arg(&pkg)
            .arg("--prover-name")
            .arg(&prover_name)
            .current_dir(&self.compose_dir)
            .output()
            .map_err(|source| DriverError::Spawn { tool: "nargo".into(), source })?;
        if !witness_path.exists() {
            return Err(DriverError::Tool {
                tool: "nargo execute".into(),
                stderr: format!(
                    "no witness produced (relation unsatisfiable or bad input):\n{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
            });
        }
        Ok(witness_path)
    }

    /// Full prove: compile -> witness -> bb prove + write_vk. `out_dir` is a
    /// scratch directory the artifacts are written into.
    ///
    /// NOTE: this uses the SHARED (untagged) witness path and so is only safe
    /// single-threaded against a given member; concurrent callers must use
    /// [`Self::prove_in`] with a unique tag.
    pub fn prove(
        &self,
        id: &CircuitId,
        prover_toml: &str,
        out_dir: &Path,
    ) -> Result<ProofArtifacts, DriverError> {
        self.prove_in(id, prover_toml, out_dir, "")
    }

    /// As [`Self::prove`], but threads a unique `tag` through the witness step
    /// so concurrent proves against the same member don't race on the shared
    /// `Prover.toml` / `target/<pkg>_w.gz` (the bb artifacts already land in the
    /// caller's isolated `out_dir`). Use a per-test-unique `tag`.
    // [OPUS-4.8] tag-isolated prove path (roborev codex job 2180).
    pub fn prove_in(
        &self,
        id: &CircuitId,
        prover_toml: &str,
        out_dir: &Path,
        tag: &str,
    ) -> Result<ProofArtifacts, DriverError> {
        let acir = self.compile(id)?;
        let witness = self.gen_witness_tagged(id, prover_toml, tag)?;
        std::fs::create_dir_all(out_dir)?;

        // `--write_vk` emits proof, public_inputs, AND vk in one pass (bb
        // prove otherwise requires a pre-existing vk).
        run(
            "bb",
            Command::new("bb")
                .arg("prove")
                .arg("-b")
                .arg(&acir)
                .arg("-w")
                .arg(&witness)
                .arg("-o")
                .arg(out_dir)
                .arg("--write_vk")
                .arg("-t")
                .arg(&self.target),
        )?;

        let proof = std::fs::read(out_dir.join("proof"))?;
        let public_inputs = std::fs::read(out_dir.join("public_inputs"))?;
        let vk = std::fs::read(out_dir.join("vk"))?;
        Ok(ProofArtifacts { proof, public_inputs, vk })
    }

    /// Recompute the CANONICAL verification key for a circuit-family member,
    /// verifier-side, from the compiled member named by `id` — never trusting a
    /// prover-supplied vk (audit #2). Compiles the member (idempotent) then runs
    /// `bb write_vk`. Measured deterministic and fast (~40-60ms once the ACIR is
    /// cached, ~350ms cold); a freshly-recompiled ACIR yields a byte-identical
    /// vk to `bb prove --write_vk`, so this is the authentic member vk.
    // [OPUS-4.8] new verifier-side canonical-vk path (audit #2).
    pub fn canonical_vk(&self, id: &CircuitId, work_dir: &Path) -> Result<Vec<u8>, DriverError> {
        let acir = self.compile(id)?;
        std::fs::create_dir_all(work_dir)?;
        run(
            "bb",
            Command::new("bb")
                .arg("write_vk")
                .arg("-b")
                .arg(&acir)
                .arg("-o")
                .arg(work_dir)
                .arg("-t")
                .arg(&self.target),
        )?;
        Ok(std::fs::read(work_dir.join("vk"))?)
    }

    /// Verify a proof against an EXPLICIT verification key and public-input
    /// bytes — the verifier supplies the canonical member vk (from
    /// [`Self::canonical_vk`]) and the reconstructed-and-byte-checked public
    /// inputs, NEVER the prover's `art.vk` / `art.public_inputs`. Returns
    /// Ok(true) on a valid proof, Ok(false) if bb rejects, Err on spawn/io.
    // [OPUS-4.8] verify against caller-pinned vk + public inputs (audit #1/#2).
    pub fn verify_with(
        &self,
        proof: &[u8],
        public_inputs: &[u8],
        vk: &[u8],
        work_dir: &Path,
    ) -> Result<bool, DriverError> {
        std::fs::create_dir_all(work_dir)?;
        let proof_p = work_dir.join("proof");
        let pi_p = work_dir.join("public_inputs");
        let vk_p = work_dir.join("vk");
        std::fs::write(&proof_p, proof)?;
        std::fs::write(&pi_p, public_inputs)?;
        std::fs::write(&vk_p, vk)?;

        let out = Command::new("bb")
            .arg("verify")
            .arg("-p")
            .arg(&proof_p)
            .arg("-i")
            .arg(&pi_p)
            .arg("-k")
            .arg(&vk_p)
            .arg("-t")
            .arg(&self.target)
            .output()
            .map_err(|source| DriverError::Spawn { tool: "bb".into(), source })?;
        Ok(out.status.success())
    }

    /// Verify artifacts via `bb verify` using the bundled vk + public inputs.
    ///
    /// NOTE: this trusts `art.vk` and `art.public_inputs` and so is NOT a sound
    /// third-party gate on its own — `verify_manifest` does NOT call this; it
    /// recomputes the canonical vk and reconstructs the public inputs (see
    /// [`Self::canonical_vk`] / [`Self::verify_with`] and the verifier). Kept
    /// for the prover-side round-trip / bb-tamper tests only.
    pub fn verify(&self, art: &ProofArtifacts, work_dir: &Path) -> Result<bool, DriverError> {
        self.verify_with(&art.proof, &art.public_inputs, &art.vk, work_dir)
    }
}

fn run(tool: &str, cmd: &mut Command) -> Result<(), DriverError> {
    let out = cmd
        .output()
        .map_err(|source| DriverError::Spawn { tool: tool.into(), source })?;
    if !out.status.success() {
        return Err(DriverError::Tool {
            tool: tool.into(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(())
}
