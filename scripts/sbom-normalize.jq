# [OPUS-4.8] sq-toze.30 (GS-6 / F-6): deterministic, idempotent normalization of a
# cargo-cyclonedx CycloneDX SBOM so the PUBLISHED document carries NO host-revealing
# absolute build path.
#
# cargo-cyclonedx 0.5.9 emits, for every workspace / path dependency, refs + purls that
# embed the CI runner's absolute build directory (and the local layout):
#
#   bom-ref (component / dependency edge):
#       path+file:///abs/build/dir/crates/<name>#<version>
#       path+file:///abs/build/dir/crates/<name>#<version> bin-target-N   (root build targets)
#   purl:
#       pkg:cargo/<name>@<version>?download_url=file://.                   (root component)
#       pkg:cargo/<name>@<version>?download_url=file://../<rel>            (workspace members)
#       pkg:cargo/<name>@<version>?download_url=file://.#src/lib.rs        (root build targets)
#
# Registry deps are already host-independent
#   (registry+https://github.com/rust-lang/crates.io-index#<name>@<version>) and untouched.
#
# This filter rewrites every leaking ref to the canonical, host-independent CycloneDX form
#   pkg:cargo/<name>@<version>[<suffix>]
# preserving the real component identity (name@version) plus any trailing target suffix, and
# rewriting EVERY place a ref appears — metadata.component.bom-ref, that component's nested
# build-target components, each top-level component's bom-ref + purl, and every
# dependencies[].ref / dependsOn[] edge — so the internal reference graph stays consistent.
#
# Properties:
#   * Deterministic: a pure function of the input (no time / host / RNG); same in -> same out.
#   * Idempotent: the canonical form contains no "path+file" and no "download_url=file://",
#     so the guard conditions are false on a second pass -> byte-identical output.
#   * Validity-preserving: only ref/purl strings change; the dependency graph is rewritten in
#     lock-step (component bom-ref, every ref and every dependsOn edge), so all internal
#     references still resolve.

# Canonicalise a single bom-ref / dependsOn string.
#   path+file:///abs/.../<name>#<version><suffix>  ->  pkg:cargo/<name>@<version><suffix>
# where <suffix> is whatever trails the version (e.g. " bin-target-0"); usually empty.
def canon_ref:
  if type == "string" and startswith("path+file://") and test("#") then
    (sub("#.*$"; "") | sub("^.*/"; "")) as $name        # crate name = basename before '#'
    | (sub("^[^#]*#"; "")) as $rest                      # everything after the first '#'
    | ($rest | sub("^(?<v>[^ ]*)"; "")) as $suffix       # trailing suffix after the version token
    | ($rest | sub("^(?<v>[^ ]*).*$"; "\(.v)")) as $version
    | "pkg:cargo/\($name)@\($version)\($suffix)"
  else
    .
  end;

# Canonicalise a purl: strip the workspace-local ?download_url=file://... query (the only
# file:// a purl carries), preserving any trailing #fragment. Registry purls have no such
# query and are returned unchanged.
def canon_purl:
  if type == "string" and test("[?&]download_url=file://") then
    sub("[?&]download_url=file://[^#]*"; "")
  else
    .
  end;

def fix_component:
  (if has("bom-ref") then ."bom-ref" |= canon_ref else . end)
  | (if has("purl") then .purl |= canon_purl else . end)
  | (if has("components") then .components |= map(fix_component) else . end);

# root metadata component (+ its nested build-target components, via fix_component recursion)
( if (.metadata? and .metadata.component?) then
    .metadata.component |= fix_component
  else . end )
# every top-level component (recursively)
| ( if has("components") then .components |= map(fix_component) else . end )
# dependency graph: ref + every dependsOn edge
| ( if has("dependencies") then
      .dependencies |= map(
        (if has("ref") then .ref |= canon_ref else . end)
        | (if has("dependsOn") then .dependsOn |= map(canon_ref) else . end)
      )
    else . end )
