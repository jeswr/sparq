fn main() {
    // On macOS an extension module must be linked with `-undefined dynamic_lookup`
    // (the interpreter provides the CPython symbols at import time). maturin sets
    // this up itself, but adding it here keeps a plain `cargo build -p sparq-py`
    // (e.g. the workspace CI build) working too. No-op on other platforms.
    pyo3_build_config::add_extension_module_link_args();
}
