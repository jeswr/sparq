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
    pub fn gen_witness(&self, id: &CircuitId, prover_toml: &str) -> Result<PathBuf, DriverError> {
        let pkg = id.package();
        let toml_path = self.package_dir(id).join("Prover.toml");
        std::fs::write(&toml_path, prover_toml)?;
        let witness_name = format!("{pkg}_w");
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
    pub fn prove(
        &self,
        id: &CircuitId,
        prover_toml: &str,
        out_dir: &Path,
    ) -> Result<ProofArtifacts, DriverError> {
        let acir = self.compile(id)?;
        let witness = self.gen_witness(id, prover_toml)?;
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

    /// Verify artifacts via `bb verify`. Returns Ok(true) on a valid proof,
    /// Ok(false) if bb rejects the proof, Err on a spawn/io failure.
    pub fn verify(&self, art: &ProofArtifacts, work_dir: &Path) -> Result<bool, DriverError> {
        std::fs::create_dir_all(work_dir)?;
        let proof_p = work_dir.join("proof");
        let pi_p = work_dir.join("public_inputs");
        let vk_p = work_dir.join("vk");
        std::fs::write(&proof_p, &art.proof)?;
        std::fs::write(&pi_p, &art.public_inputs)?;
        std::fs::write(&vk_p, &art.vk)?;

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
