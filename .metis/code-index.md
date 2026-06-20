# Code Index

> Generated: 2026-06-20T02:29:45Z | 143 files | Go, JavaScript, Python, Rust

## Project Structure

```
├── crates/
│   ├── fidius/
│   │   └── src/
│   │       └── lib.rs
│   ├── fidius-build/
│   │   └── src/
│   │       └── lib.rs
│   ├── fidius-cli/
│   │   ├── src/
│   │   │   ├── commands.rs
│   │   │   ├── main.rs
│   │   │   └── python_stub.rs
│   │   └── tests/
│   │       ├── cli.rs
│   │       ├── full_pipeline.rs
│   │       └── wasm_pack.rs
│   ├── fidius-core/
│   │   ├── src/
│   │   │   ├── async_runtime.rs
│   │   │   ├── lib.rs
│   │   │   ├── package.rs
│   │   │   └── registry.rs
│   │   └── tests/
│   │       └── layout_and_roundtrip.rs
│   ├── fidius-guest/
│   │   ├── src/
│   │   │   ├── descriptor.rs
│   │   │   ├── error.rs
│   │   │   ├── frame.rs
│   │   │   ├── hash.rs
│   │   │   ├── http.rs
│   │   │   ├── lib.rs
│   │   │   ├── python_descriptor.rs
│   │   │   ├── status.rs
│   │   │   ├── stream_ffi.rs
│   │   │   ├── stream_marker.rs
│   │   │   ├── value.rs
│   │   │   ├── wasm_descriptor.rs
│   │   │   └── wire.rs
│   │   └── tests/
│   │       └── wasi_http_pin.rs
│   ├── fidius-host/
│   │   ├── benches/
│   │   │   └── backends.rs
│   │   ├── build.rs
│   │   ├── src/
│   │   │   ├── arch.rs
│   │   │   ├── arena.rs
│   │   │   ├── error.rs
│   │   │   ├── executor/
│   │   │   │   ├── cdylib.rs
│   │   │   │   ├── python.rs
│   │   │   │   └── wasm.rs
│   │   │   ├── executor.rs
│   │   │   ├── handle.rs
│   │   │   ├── host.rs
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs
│   │   │   ├── package.rs
│   │   │   ├── signing.rs
│   │   │   ├── stream.rs
│   │   │   └── types.rs
│   │   └── tests/
│   │       ├── cdylib_streaming_e2e.rs
│   │       ├── e2e.rs
│   │       ├── integration.rs
│   │       ├── macro_egress_e2e.rs
│   │       ├── macro_wasm.rs
│   │       ├── macro_wasm_streaming.rs
│   │       ├── package_e2e.rs
│   │       ├── plugin_dep_graph.rs
│   │       ├── python_plugin_e2e.rs
│   │       ├── python_routing.rs
│   │       ├── python_streaming_e2e.rs
│   │       ├── records_wasm.rs
│   │       ├── wasm_egress_e2e.rs
│   │       ├── wasm_executor.rs
│   │       └── wasm_streaming_e2e.rs
│   ├── fidius-macro/
│   │   ├── src/
│   │   │   ├── impl_macro.rs
│   │   │   ├── interface.rs
│   │   │   ├── ir.rs
│   │   │   ├── lib.rs
│   │   │   └── wit.rs
│   │   └── tests/
│   │       ├── arena_basic.rs
│   │       ├── async_plugin.rs
│   │       ├── compile_fail/
│   │       │   ├── caller_allocated_removed.rs
│   │       │   ├── duplicate_method_meta_key.rs
│   │       │   ├── missing_version.rs
│   │       │   ├── mut_self.rs
│   │       │   ├── reserved_fidius_namespace.rs
│   │       │   └── stream_in_arg_position.rs
│   │       ├── crate_path.rs
│   │       ├── impl_basic.rs
│   │       ├── interface_basic.rs
│   │       ├── metadata.rs
│   │       ├── multi_arg.rs
│   │       ├── multi_plugin.rs
│   │       ├── raw_wire.rs
│   │       ├── smoke_cdylib.rs
│   │       └── trybuild.rs
│   ├── fidius-python/
│   │   ├── build.rs
│   │   ├── src/
│   │   │   ├── error.rs
│   │   │   ├── handle.rs
│   │   │   ├── interpreter.rs
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs
│   │   │   ├── stream.rs
│   │   │   └── value_bridge.rs
│   │   └── tests/
│   │       ├── loader_e2e.rs
│   │       └── smoke.rs
│   ├── fidius-test/
│   │   ├── src/
│   │   │   ├── dylib.rs
│   │   │   ├── lib.rs
│   │   │   ├── signing.rs
│   │   │   └── stream.rs
│   │   └── tests/
│   │       └── smoke.rs
│   └── fidius-wit/
│       └── src/
│           ├── generate.rs
│           └── lib.rs
├── pluggable-poc/
│   ├── crates/
│   │   ├── emit-console/
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── ingest-csv/
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── pipeline-host/
│   │   │   └── src/
│   │   │       ├── arrow_bridge.rs
│   │   │       ├── config.rs
│   │   │       ├── main.rs
│   │   │       └── orchestrator.rs
│   │   ├── pipeline-types/
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── plugin-runtime/
│   │   │   └── src/
│   │   │       ├── ffi_plugin.rs
│   │   │       ├── lib.rs
│   │   │       ├── native.rs
│   │   │       ├── pyo3_process.rs
│   │   │       ├── pyo3_thread.rs
│   │   │       └── pyo3_zerocopy.rs
│   │   ├── transform-double/
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── transform-normalize/
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   └── transform-onnx/
│   │       └── src/
│   │           └── lib.rs
│   ├── data/
│   │   └── generate_data.py
│   ├── models/
│   │   └── train_model.py
│   └── plugins/
│       ├── ffi/
│       │   └── transform-double-ffi/
│       │       └── src/
│       │           └── lib.rs
│       ├── harness.py
│       └── transform_column_doubler.py
├── python/
│   ├── fidius/
│   │   ├── __init__.py
│   │   ├── _errors.py
│   │   └── _registry.py
│   └── tests/
│       └── test_sdk.py
├── tests/
│   ├── test-plugin-py-greeter/
│   │   └── byte_pipe.py
│   ├── test-plugin-py-ticker/
│   │   └── ticker.py
│   ├── test-plugin-smoke/
│   │   └── src/
│   │       └── lib.rs
│   └── wasm-fixtures/
│       ├── fetcher/
│       │   └── src/
│       │       └── lib.rs
│       ├── greeter/
│       │   └── src/
│       │       └── lib.rs
│       ├── greeter-go/
│       │   └── main.go
│       ├── greeter-js/
│       │   └── greeter.js
│       ├── greeter-py/
│       │   └── app.py
│       ├── macro-fetcher/
│       │   └── src/
│       │       └── lib.rs
│       ├── macro-greeter/
│       │   └── src/
│       │       └── lib.rs
│       ├── macro-ticker/
│       │   └── src/
│       │       └── lib.rs
│       ├── records-greeter/
│       │   ├── build.rs
│       │   └── src/
│       │       ├── geom.rs
│       │       └── lib.rs
│       ├── ticker/
│       │   └── src/
│       │       └── lib.rs
│       ├── ticker-js/
│       │   └── ticker.js
│       └── ticker-py/
│           └── app.py
└── wasm-spike/
    ├── guest/
    │   └── src/
    │       └── lib.rs
    ├── host/
    │   └── src/
    │       └── main.rs
    └── twogen/
        └── src/
            └── lib.rs
```

## Modules

### crates/fidius-build/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-build/src/lib.rs

- pub `emit_wit` function L41-48 — `()` — Regenerate `wit/` and the conversions from `src/lib.rs`.
- pub `run` function L52-76 — `(manifest_dir: &Path, out_dir: &Path) -> Result<(), String>` — Core of [`emit_wit`], parameterized on the crate dir + output dir so it is
-  `tests` module L79-165 — `-` — trait and the `#[derive(WitType)]` types to live in `src/lib.rs`.
-  `writes_wit_and_conversions_for_a_user_typed_interface` function L83-108 — `()` — trait and the `#[derive(WitType)]` types to live in `src/lib.rs`.
-  `follows_external_modules` function L111-142 — `()` — trait and the `#[derive(WitType)]` types to live in `src/lib.rs`.
-  `primitives_only_writes_empty_conversions` function L145-164 — `()` — trait and the `#[derive(WitType)]` types to live in `src/lib.rs`.

### crates/fidius-cli/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-cli/src/commands.rs

- pub `init_interface` function L78-138 — `( name: &str, trait_name: &str, path: Option<&Path>, version: Option<&str>, exte...`
- pub `init_plugin` function L142-280 — `( name: &str, interface: &str, trait_name: &str, path: Option<&Path>, version: O...`
- pub `init_host` function L284-392 — `( name: &str, interface: &str, trait_name: &str, path: Option<&Path>, version: O...`
- pub `keygen` function L396-417 — `(out: &str) -> Result`
- pub `sign` function L421-441 — `(key_path: &Path, dylib_path: &Path) -> Result`
- pub `verify` function L445-477 — `(key_path: &Path, dylib_path: &Path) -> Result`
- pub `inspect` function L481-527 — `(dylib_path: &Path) -> Result`
- pub `test` function L531-612 — `(dir: &Path, release: bool) -> Result`
- pub `package_validate` function L616-631 — `(dir: &Path) -> Result`
- pub `package_build` function L635-666 — `(dir: &Path, release: bool) -> Result`
- pub `package_inspect` function L670-707 — `(dir: &Path) -> Result`
- pub `package_sign` function L711-732 — `(key_path: &Path, dir: &Path) -> Result`
- pub `package_verify` function L736-764 — `(key_path: &Path, dir: &Path) -> Result`
- pub `package_pack` function L768-807 — `(dir: &Path, output: Option<&Path>, precompile: bool) -> Result`
- pub `wit` function L882-897 — `(dir: Option<&Path>) -> Result` — Generate `<dir>/wit/<interface>.wit` from `<dir>/src/lib.rs` (the
- pub `package_unpack` function L901-906 — `(archive: &Path, dest: Option<&Path>) -> Result`
- pub `python_stub` function L910-912 — `(interface_src: &Path, out: &Path, trait_name: Option<&str>) -> Result`
-  `Result` type L19 — `= std::result::Result<T, Box<dyn std::error::Error>>`
-  `resolve_dep` function L30-56 — `(value: &str, version_override: Option<&str>) -> String` — Resolve a dependency string to a Cargo.toml dependency value.
-  `check_crates_io` function L59-74 — `(name: &str) -> Option<String>` — Check crates.io for a crate and return its latest version, if found.
-  `prepare_wasm_pack` function L812-836 — `(dir: &Path, component: &str, precompile: bool) -> Result` — Validate (and optionally precompile) a wasm component at pack time.
-  `prepare_wasm_pack` function L839-848 — `(_dir: &Path, component: &str, precompile: bool) -> Result`
-  `record_precompiled` function L853-874 — `(dir: &Path, cwasm_name: &str) -> Result` — Record `precompiled = "<name>"` under the `[wasm]` table in package.toml,

#### crates/fidius-cli/src/main.rs

-  `commands` module L20 — `-`
-  `python_stub` module L21 — `-`
-  `Cli` struct L25-28 — `{ command: Commands }`
-  `Commands` enum L31-145 — `InitInterface | InitPlugin | InitHost | Keygen | Sign | Verify | Inspect | Test ...`
-  `PackageCommands` enum L148-203 — `Validate | Build | Inspect | Sign | Verify | Pack | Unpack`
-  `main` function L205-280 — `()`

#### crates/fidius-cli/src/python_stub.rs

- pub `generate_stub` function L54-72 — `(interface_src: &Path, requested_trait: Option<&str>) -> Result<String>` — Generate the contents of a Python stub file for the named trait found in
- pub `write_stub` function L75-89 — `(interface_src: &Path, out_path: &Path, requested_trait: Option<&str>) -> Result` — Write the stub for the named trait to `out_path`.
-  `Result` type L29 — `= std::result::Result<T, Box<dyn std::error::Error>>` — agree byte-for-byte.
-  `MethodSpec` struct L32-49 — `{ name: String, arg_types: Vec<String>, arg_names_with_py_types: Vec<(String, St...` — One method extracted from a trait, ready for stub emission.
-  `has_plugin_interface_attr` function L91-99 — `(item: &ItemTrait) -> bool` — agree byte-for-byte.
-  `pick_trait` function L101-135 — `( traits: &'a [&'a ItemTrait], requested: Option<&str>, src: &Path, ) -> Result<...` — agree byte-for-byte.
-  `extract_methods` function L137-146 — `(item: &ItemTrait) -> Result<Vec<MethodSpec>>` — agree byte-for-byte.
-  `method_to_spec` function L148-208 — `(method: &TraitItemFn) -> Result<MethodSpec>` — agree byte-for-byte.
-  `is_wire_raw_attr` function L210-222 — `(attr: &syn::Attribute) -> bool` — agree byte-for-byte.
-  `token_string` function L224-226 — `(t: &T) -> String` — agree byte-for-byte.
-  `extract_doc_line` function L228-242 — `(attr: &syn::Attribute) -> Option<String>` — agree byte-for-byte.
-  `rust_type_to_python` function L246-313 — `(ty: &Type) -> String` — Map a Rust type to its Python type-hint counterpart.
-  `is_u8` function L315-320 — `(ty: &Type) -> bool` — agree byte-for-byte.
-  `render_python_stub` function L322-393 — `(trait_name: &str, methods: &[MethodSpec]) -> String` — agree byte-for-byte.
-  `tests` module L396-537 — `-` — agree byte-for-byte.
-  `parse_methods` function L399-411 — `(src: &str) -> (String, Vec<MethodSpec>)` — agree byte-for-byte.
-  `primitive_type_mapping` function L414-429 — `()` — agree byte-for-byte.
-  `vec_u8_maps_to_bytes_even_without_wire_raw` function L432-442 — `()` — agree byte-for-byte.
-  `wire_raw_signatures_are_bytes` function L445-459 — `()` — agree byte-for-byte.
-  `unknown_types_get_todo_marker` function L462-472 — `()` — agree byte-for-byte.
-  `rendered_stub_hash_matches_macro` function L475-507 — `()` — agree byte-for-byte.
-  `picks_named_trait_when_multiple_present` function L510-536 — `()` — agree byte-for-byte.

### crates/fidius-cli/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-cli/tests/cli.rs

-  `fidius_cmd` function L23-25 — `() -> Command` — CLI integration tests using assert_cmd.
-  `plugin_source_dir` function L27-29 — `() -> PathBuf` — CLI integration tests using assert_cmd.
-  `plugin_dir` function L31-39 — `() -> &'static Path` — CLI integration tests using assert_cmd.
-  `DIR` variable L32 — `: std::sync::OnceLock<PathBuf>` — CLI integration tests using assert_cmd.
-  `smoke_dylib_name` function L41-49 — `() -> &'static str` — CLI integration tests using assert_cmd.
-  `help_works` function L52-63 — `()` — CLI integration tests using assert_cmd.
-  `init_interface_creates_files` function L66-95 — `()` — CLI integration tests using assert_cmd.
-  `init_interface_errors_if_exists` function L98-127 — `()` — CLI integration tests using assert_cmd.
-  `init_plugin_creates_files` function L130-162 — `()` — CLI integration tests using assert_cmd.
-  `keygen_sign_verify_roundtrip` function L165-199 — `()` — CLI integration tests using assert_cmd.
-  `inspect_shows_plugin_info` function L202-212 — `()` — CLI integration tests using assert_cmd.

#### crates/fidius-cli/tests/full_pipeline.rs

-  `fides_cmd` function L23-25 — `() -> Command` — Everything is generated from scratch by the CLI.
-  `workspace_fidius_path` function L28-30 — `() -> PathBuf` — Path to the workspace root's `fidius` facade crate (for local dep resolution).
-  `full_pipeline_scaffold_package_build_sign_load_call` function L33-363 — `()` — Everything is generated from scratch by the CLI.

#### crates/fidius-cli/tests/wasm_pack.rs

-  `stage_wasm_pkg` function L28-50 — `(dir: &std::path::Path)` — `fidius-host --features wasm` tests (`wasm_executor.rs`).
-  `pack_wasm_package_archives_with_a_skip_warning` function L53-73 — `()` — `fidius-host --features wasm` tests (`wasm_executor.rs`).
-  `precompile_without_wasm_feature_errors` function L76-87 — `()` — `fidius-host --features wasm` tests (`wasm_executor.rs`).
-  `inspect_renders_wasm_fields` function L90-127 — `()` — `fidius-host --features wasm` tests (`wasm_executor.rs`).
-  `sign_verify_and_tamper_wasm_package` function L130-163 — `()` — `fidius-host --features wasm` tests (`wasm_executor.rs`).

### crates/fidius-core/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-core/src/async_runtime.rs

- pub `FIDIUS_RUNTIME` variable L25-31 — `: std::sync::LazyLock<tokio::runtime::Runtime>` — The shared tokio runtime for this dylib.

#### crates/fidius-core/src/lib.rs

- pub `package` module L16 — `-`
- pub `registry` module L17 — `-`
- pub `async_runtime` module L20 — `-`

#### crates/fidius-core/src/package.rs

- pub `PackageManifest` struct L32-47 — `{ package: PackageHeader, metadata: M, python: Option<PythonPackageMeta>, wasm: ...` — A parsed package manifest, generic over the host-defined metadata schema.
- pub `validate_runtime` function L56-100 — `(&self) -> Result<(), PackageError>` — Cross-section validation: runtime + python section must agree.
- pub `PackageHeader` struct L105-123 — `{ name: String, version: String, interface: String, interface_version: u32, exte...` — Fixed header fields that every package manifest must have.
- pub `extension` function L127-129 — `(&self) -> &str` — Returns the package extension, defaulting to `"fid"`.
- pub `runtime` function L134-144 — `(&self) -> PackageRuntime` — Returns the runtime kind, defaulting to `Rust` when absent.
- pub `runtime_strict` function L147-156 — `(&self) -> Result<PackageRuntime, PackageError>` — Returns the runtime kind, erroring on unknown values.
- pub `PackageRuntime` enum L162-175 — `Rust | Python | Wasm` — Plugin runtime kind.
- pub `as_str` function L179-185 — `(&self) -> &'static str` — Returns the canonical string form used in `package.toml`.
- pub `PythonPackageMeta` struct L197-206 — `{ entry_module: String, requirements: Option<String> }` — Fields under the `[python]` section of `package.toml`.
- pub `WasmPackageMeta` struct L211-225 — `{ component: String, precompiled: Option<String>, capabilities: Vec<String> }` — Fields under the `[wasm]` section of `package.toml`.
- pub `requirements_path` function L229-231 — `(&self) -> &str` — Returns the requirements file path, defaulting to `"requirements.txt"`.
- pub `PackageError` enum L236-300 — `ManifestNotFound | ParseError | Io | BuildFailed | SignatureNotFound | Signature...` — Errors that can occur when loading a package manifest.
- pub `UnpackOptions` struct L309-319 — `{ max_decompressed: u64, max_ratio: u64, max_entries: u32 }` — Options controlling archive extraction safety limits.
- pub `load_manifest` function L349-366 — `(dir: &Path) -> Result<PackageManifest<M>, PackageError>` — Load and parse a `package.toml` manifest from a package directory.
- pub `load_manifest_untyped` function L372-374 — `(dir: &Path) -> Result<PackageManifest<toml::Value>, PackageError>` — Load a manifest validating only the fixed header (accepting any metadata).
- pub `package_digest` function L384-405 — `(dir: &Path) -> Result<[u8; 32], PackageError>` — Compute a deterministic SHA-256 digest over all package source files.
- pub `PackResult` struct L474-479 — `{ path: PathBuf, unsigned: bool }` — Result of packing a package, including any warnings.
- pub `pack_package` function L560-613 — `(dir: &Path, output: Option<&Path>) -> Result<PackResult, PackageError>` — Create a `.fid` archive (tar + bzip2) from a package directory.
- pub `unpack_package` function L632-634 — `(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError>` — Extract a `.fid` archive (tar + bzip2) to a destination directory using
- pub `unpack_package_with_options` function L640-777 — `( archive: &Path, dest: &Path, options: &UnpackOptions, ) -> Result<PathBuf, Pac...` — Extract a `.fid` archive with caller-provided safety limits.
-  `PackageHeader` type L125-157 — `= PackageHeader` — host-defined schema type.
-  `PackageRuntime` type L177-186 — `= PackageRuntime` — host-defined schema type.
-  `PackageRuntime` type L188-192 — `= PackageRuntime` — host-defined schema type.
-  `fmt` function L189-191 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — host-defined schema type.
-  `PythonPackageMeta` type L227-232 — `= PythonPackageMeta` — host-defined schema type.
-  `UnpackOptions` type L321-329 — `impl Default for UnpackOptions` — host-defined schema type.
-  `default` function L322-328 — `() -> Self` — host-defined schema type.
-  `collect_files` function L408-439 — `(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), PackageError>` — Recursively collect file paths relative to `root`, skipping excluded dirs/files.
-  `collect_archive_files` function L442-470 — `( root: &Path, dir: &Path, out: &mut Vec<String>, ) -> Result<(), PackageError>` — Recursively collect file paths for archiving (includes `.sig` files).
-  `vendor_python_deps` function L490-545 — `(dir: &Path, py: &PythonPackageMeta) -> Result<(), PackageError>` — Vendor Python dependencies into `<dir>/vendor/` by invoking
-  `tests` module L780-1696 — `-` — host-defined schema type.
-  `write_manifest` function L784-786 — `(dir: &Path, content: &str)` — host-defined schema type.
-  `TestMeta` struct L789-793 — `{ category: String, tags: Vec<String> }` — host-defined schema type.
-  `valid_manifest_parses` function L796-820 — `()` — host-defined schema type.
-  `missing_required_metadata_field_fails` function L823-847 — `()` — host-defined schema type.
-  `missing_manifest_returns_not_found` function L850-854 — `()` — host-defined schema type.
-  `extra_metadata_fields_ignored` function L857-878 — `()` — host-defined schema type.
-  `untyped_manifest_accepts_any_metadata` function L881-902 — `()` — host-defined schema type.
-  `digest_is_deterministic` function L905-913 — `()` — host-defined schema type.
-  `digest_changes_on_file_modification` function L916-927 — `()` — host-defined schema type.
-  `digest_excludes_target_and_sig` function L930-944 — `()` — host-defined schema type.
-  `make_package` function L946-962 — `(dir: &Path)` — host-defined schema type.
-  `pack_unpack_round_trip` function L965-986 — `()` — host-defined schema type.
-  `pack_includes_sig_file` function L989-1003 — `()` — host-defined schema type.
-  `pack_excludes_target_and_git` function L1006-1022 — `()` — host-defined schema type.
-  `unpack_invalid_archive_no_manifest` function L1025-1055 — `()` — host-defined schema type.
-  `pack_default_output_name` function L1058-1068 — `()` — host-defined schema type.
-  `pack_custom_extension` function L1071-1101 — `()` — host-defined schema type.
-  `extension_defaults_to_fid` function L1104-1120 — `()` — host-defined schema type.
-  `rust_runtime_default_when_absent` function L1125-1143 — `()` — host-defined schema type.
-  `python_runtime_with_python_section_parses` function L1146-1171 — `()` — host-defined schema type.
-  `python_runtime_requirements_default` function L1174-1198 — `()` — host-defined schema type.
-  `python_runtime_without_python_section_rejected` function L1201-1227 — `()` — host-defined schema type.
-  `python_section_without_python_runtime_rejected` function L1230-1250 — `()` — host-defined schema type.
-  `unknown_runtime_rejected` function L1253-1276 — `()` — host-defined schema type.
-  `package_runtime_display_and_str` function L1279-1283 — `()` — host-defined schema type.
-  `build_archive` function L1293-1302 — `(path: &Path, build: F)` — Build a bz2-compressed tar archive from a builder callback.
-  `write_name` function L1307-1315 — `(header: &mut Header, path: &str)` — Write a raw entry name directly into a GNU tar header, bypassing
-  `write_linkname` function L1317-1325 — `(header: &mut Header, link: &str)` — host-defined schema type.
-  `append_regular` function L1330-1338 — `(tar: &mut tar::Builder<BzEncoder<std::fs::File>>, path: &str, data: &[u8])` — Append a regular file entry with explicit path and content bytes.
-  `append_link` function L1341-1355 — `( tar: &mut tar::Builder<BzEncoder<std::fs::File>>, path: &str, link_target: &st...` — Append a link entry with a chosen EntryType (symlink/hardlink).
-  `unpack_rejects_parent_dir_component` function L1358-1373 — `()` — host-defined schema type.
-  `unpack_rejects_absolute_path` function L1376-1389 — `()` — host-defined schema type.
-  `unpack_rejects_symlink` function L1392-1405 — `()` — host-defined schema type.
-  `unpack_rejects_hardlink` function L1408-1421 — `()` — host-defined schema type.
-  `unpack_symlink_then_file_rejected_at_first_entry` function L1424-1445 — `()` — host-defined schema type.
-  `unpack_rejects_declared_size_bomb` function L1448-1477 — `()` — host-defined schema type.
-  `unpack_rejects_ratio_bomb` function L1480-1507 — `()` — host-defined schema type.
-  `unpack_rejects_too_many_entries` function L1510-1529 — `()` — host-defined schema type.
-  `unpack_staging_cleans_up_on_rejection` function L1532-1553 — `()` — host-defined schema type.
-  `unpack_with_options_accepts_large_archive` function L1556-1574 — `()` — host-defined schema type.
-  `make_python_package` function L1579-1613 — `(dir: &Path, with_requirements: Option<&str>)` — Build a minimal Python package directory (manifest + entry .py).
-  `pack_python_with_prevendored_directory_skips_pip` function L1616-1642 — `()` — host-defined schema type.
-  `pack_python_with_no_requirements_or_vendor_warns_but_succeeds` function L1645-1658 — `()` — host-defined schema type.
-  `pack_python_with_unresolvable_requirement_surfaces_pip_error` function L1661-1695 — `()` — host-defined schema type.

#### crates/fidius-core/src/registry.rs

- pub `DescriptorEntry` struct L24-26 — `{ descriptor: &'static PluginDescriptor }` — A submitted descriptor pointer.
- pub `get_registry` function L55-58 — `() -> &'static PluginRegistry` — Get or build the plugin registry.
-  `build_registry` function L34-49 — `() -> PluginRegistry` — Build the plugin registry from all submitted descriptors.
-  `REGISTRY` variable L56 — `: std::sync::OnceLock<PluginRegistry>` — `fidius_get_registry` export function that the host calls via `dlsym`.
-  `fidius_plugin_registry` macro L69-76 — `-` — Emit the `fidius_get_registry` export function.

### crates/fidius-core/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-core/tests/layout_and_roundtrip.rs

-  `registry_size_and_align` function L32-36 — `()` — and interface hash determinism.
-  `registry_field_offsets` function L39-44 — `()` — and interface hash determinism.
-  `descriptor_size_and_align` function L49-58 — `()` — and interface hash determinism.
-  `descriptor_field_offsets` function L61-83 — `()` — and interface hash determinism.
-  `buffer_strategy_kind_layout` function L88-93 — `()` — and interface hash determinism.
-  `status_code_values` function L98-104 — `()` — and interface hash determinism.
-  `TestPayload` struct L109-113 — `{ name: String, value: i64, tags: Vec<String> }` — and interface hash determinism.
-  `wire_roundtrip` function L116-126 — `()` — and interface hash determinism.
-  `wire_is_bincode_always` function L129-143 — `()` — and interface hash determinism.
-  `plugin_error_roundtrip_without_details` function L148-155 — `()` — and interface hash determinism.
-  `plugin_error_roundtrip_with_details` function L158-165 — `()` — and interface hash determinism.
-  `plugin_error_display` function L168-171 — `()` — and interface hash determinism.
-  `hash_known_vectors` function L176-204 — `()` — and interface hash determinism.
-  `hash_const_fnv1a` function L207-212 — `()` — and interface hash determinism.
-  `HASH` variable L209 — `: u64` — and interface hash determinism.
-  `magic_bytes_value` function L217-220 — `()` — and interface hash determinism.
-  `version_constants` function L223-230 — `()` — and interface hash determinism.

### crates/fidius-guest/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-guest/src/descriptor.rs

- pub `FIDIUS_MAGIC` variable L24 — `: [u8; 8]` — Magic bytes identifying a Fidius plugin registry.
- pub `REGISTRY_VERSION` variable L27 — `: u32` — Current version of the `PluginRegistry` struct layout.
- pub `ABI_VERSION` variable L50-54 — `: u32` — Current version of the `PluginDescriptor` struct layout.
- pub `BufferStrategyKind` enum L65-84 — `PluginAllocated | Arena` — Buffer management strategy for an interface.
- pub `MetaKv` struct L94-99 — `{ key: *const c_char, value: *const c_char }` — Static key/value pair for method-level or trait-level metadata.
- pub `MethodMetaEntry` struct L112-118 — `{ kvs: *const MetaKv, kv_count: u32 }` — Per-method metadata entry.
- pub `PluginRegistry` struct L145-154 — `{ magic: [u8; 8], registry_version: u32, plugin_count: u32, descriptors: *const ...` — Top-level registry exported by every Fidius plugin dylib.
- pub `PluginDescriptor` struct L177-236 — `{ descriptor_size: u32, abi_version: u32, interface_name: *const c_char, interfa...` — Metadata descriptor for a single plugin within a dylib.
- pub `DescriptorPtr` struct L250 — `-` — A `Sync` wrapper for a raw pointer to a `PluginDescriptor`.
- pub `interface_name_str` function L263-266 — `(&self) -> &str` — Read the `interface_name` field as a Rust `&str`.
- pub `plugin_name_str` function L274-277 — `(&self) -> &str` — Read the `plugin_name` field as a Rust `&str`.
- pub `buffer_strategy_kind` function L283-289 — `(&self) -> Result<BufferStrategyKind, u8>` — Returns the `buffer_strategy` field as a `BufferStrategyKind`.
- pub `has_capability` function L294-299 — `(&self, bit: u32) -> bool` — Check if the given optional method capability bit is set.
-  `parse_u32_const` function L34-43 — `(s: &str) -> u32` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `CRATE_MAJOR` variable L45 — `: u32` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `CRATE_MINOR` variable L46 — `: u32` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `MetaKv` type L102 — `impl Send for MetaKv` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `MetaKv` type L103 — `impl Sync for MetaKv` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `MethodMetaEntry` type L121 — `impl Send for MethodMetaEntry` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `MethodMetaEntry` type L122 — `impl Sync for MethodMetaEntry` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `BufferStrategyKind` type L124-131 — `= BufferStrategyKind` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `fmt` function L125-130 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `PluginRegistry` type L159 — `impl Send for PluginRegistry` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `PluginRegistry` type L160 — `impl Sync for PluginRegistry` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `PluginDescriptor` type L241 — `impl Send for PluginDescriptor` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `PluginDescriptor` type L242 — `impl Sync for PluginDescriptor` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `DescriptorPtr` type L253 — `impl Send for DescriptorPtr` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `DescriptorPtr` type L254 — `impl Sync for DescriptorPtr` — All types use `#[repr(C)]` layout and are read directly from dylib memory.
-  `PluginDescriptor` type L256-300 — `= PluginDescriptor` — All types use `#[repr(C)]` layout and are read directly from dylib memory.

#### crates/fidius-guest/src/error.rs

- pub `PluginError` struct L30-37 — `{ code: String, message: String, details: Option<String> }` — Error returned by plugin method implementations to signal business logic failures.
- pub `new` function L41-47 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — Create a new `PluginError` without details.
- pub `with_details` function L52-62 — `( code: impl Into<String>, message: impl Into<String>, details: serde_json::Valu...` — Create a new `PluginError` with structured details.
- pub `details_value` function L67-71 — `(&self) -> Option<serde_json::Value>` — Parse the `details` field back into a `serde_json::Value`.
-  `PluginError` type L39-72 — `= PluginError` — Error types for the Fidius plugin framework.
-  `PluginError` type L74-78 — `= PluginError` — Error types for the Fidius plugin framework.
-  `fmt` function L75-77 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` — Error types for the Fidius plugin framework.
-  `PluginError` type L80 — `= PluginError` — Error types for the Fidius plugin framework.

#### crates/fidius-guest/src/frame.rs

- pub `FRAME_ITEM` variable L45 — `: u8` — Frame tag: one streamed item.
- pub `FRAME_END` variable L47 — `: u8` — Frame tag: clean end of stream.
- pub `FRAME_ERROR` variable L49 — `: u8` — Frame tag: producer error.
- pub `FRAME_HEADER_LEN` variable L52 — `: usize` — Fixed size of a frame header: one tag byte plus a `u32` length.
- pub `Frame` enum L60-67 — `Item | End | Error` — One frame crossing the streaming boundary.
- pub `FrameError` enum L71-88 — `Truncated | UnknownTag | Payload | Malformed` — Errors decoding a [`Frame`] from bytes.
- pub `encode` function L92-103 — `(&self) -> Result<Vec<u8>, WireError>` — Encode this frame as `[tag][len][payload]`.
- pub `decode` function L108-117 — `(bytes: &[u8]) -> Result<Frame, FrameError>` — Decode exactly one frame from `bytes`, which must contain a single frame
- pub `read` function L122-153 — `(bytes: &[u8]) -> Result<(Frame, usize), FrameError>` — Read one frame from the front of `bytes`, returning the frame and the
-  `Frame` type L90-154 — `= Frame` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `tests` module L157-265 — `-` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `item` function L160-162 — `(payload: &[u8]) -> Frame` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `item_round_trip` function L165-170 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `end_round_trip` function L173-178 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `error_round_trip` function L181-187 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `empty_item_is_valid` function L190-194 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `read_walks_concatenated_frames` function L197-210 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `truncated_header_is_rejected` function L213-216 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `truncated_payload_is_rejected` function L219-226 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `unknown_tag_is_rejected` function L229-236 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `end_with_payload_is_rejected` function L239-247 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `trailing_bytes_after_single_decode_rejected` function L250-257 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.
-  `garbage_is_rejected_not_panicking` function L260-264 — `()` — D5) is simply *n* `ITEM` frames concatenated, needing no wire change.

#### crates/fidius-guest/src/hash.rs

- pub `fnv1a` function L28-37 — `(bytes: &[u8]) -> u64` — Compute the FNV-1a 64-bit hash of a byte slice.
- pub `interface_hash` function L47-52 — `(signatures: &[&str]) -> u64` — Compute the interface hash from a set of method signatures.
- pub `signature_string` function L80-97 — `( name: &str, arg_types: &[String], ret: &str, wire_raw: bool, streaming: bool, ...` — Build the canonical signature string for one method.
-  `FNV_OFFSET_BASIS` variable L22 — `: u64` — FNV-1a 64-bit offset basis.
-  `FNV_PRIME` variable L25 — `: u64` — FNV-1a 64-bit prime.
-  `tests` module L100-148 — `-` — plugins compiled against a different interface.
-  `empty_input` function L104-108 — `()` — plugins compiled against a different interface.
-  `known_vector` function L111-117 — `()` — plugins compiled against a different interface.
-  `order_independence` function L120-130 — `()` — plugins compiled against a different interface.
-  `sensitivity` function L133-137 — `()` — plugins compiled against a different interface.
-  `different_signatures_differ` function L140-147 — `()` — plugins compiled against a different interface.

#### crates/fidius-guest/src/http.rs

- pub `Request` struct L57-66 — `{ method: String, url: String, headers: Vec<(String, String)>, body: Vec<u8> }` — An outbound request.
- pub `get` function L70-77 — `(url: impl Into<String>) -> Self` — A GET request for `url`.
- pub `post` function L80-87 — `(url: impl Into<String>, body: impl Into<Vec<u8>>) -> Self` — A POST request for `url` with `body`.
- pub `header` function L90-93 — `(mut self, name: impl Into<String>, value: impl Into<String>) -> Self` — Add a header (builder style).
- pub `Response` struct L98-105 — `{ status: u16, headers: Vec<(String, String)>, body: Vec<u8> }` — A response.
- pub `is_success` function L109-111 — `(&self) -> bool` — `true` for a 2xx status.
- pub `text` function L114-116 — `(&self) -> String` — The body as UTF-8 (lossy).
- pub `HttpError` struct L123-126 — `{ message: String }` — A failed request.
- pub `get` function L145-147 — `(url: &str) -> Result<Response, HttpError>` — GET `url`.
- pub `post` function L150-152 — `(url: &str, body: &[u8]) -> Result<Response, HttpError>` — POST `body` to `url`.
- pub `send` function L156-252 — `(req: Request) -> Result<Response, HttpError>` — Send an arbitrary [`Request`], blocking until the response is read.
-  `bindings` module L40-46 — `-` — ```
-  `Request` type L68-94 — `= Request` — ```
-  `Response` type L107-117 — `= Response` — ```
-  `HttpError` type L128-134 — `= HttpError` — ```
-  `new` function L129-133 — `(msg: impl Into<String>) -> Self` — ```
-  `HttpError` type L136-140 — `= HttpError` — ```
-  `fmt` function L137-139 — `(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result` — ```
-  `HttpError` type L142 — `= HttpError` — ```

#### crates/fidius-guest/src/lib.rs

- pub `descriptor` module L32 — `-` — `fidius-guest` — the wasm-buildable subset of the Fidius shared types.
- pub `error` module L33 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `frame` module L34 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `hash` module L35 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `http` module L39 — `-` — Brokered outbound HTTP for sandboxed WASM connectors (FIDIUS-I-0028).
- pub `python_descriptor` module L40 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `status` module L41 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `stream_ffi` module L42 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `stream_marker` module L43 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `value` module L44 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `wasm_descriptor` module L45 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.
- pub `wire` module L46 — `-` — (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.

#### crates/fidius-guest/src/python_descriptor.rs

- pub `PythonInterfaceDescriptor` struct L31-42 — `{ interface_name: &'static str, interface_hash: u64, methods: &'static [PythonMe...` — Static descriptor for one fidius interface, consumed by the Python
- pub `PythonMethodDesc` struct L46-54 — `{ name: &'static str, wire_raw: bool }` — One method on the interface.

#### crates/fidius-guest/src/status.rs

- pub `STATUS_OK` variable L22 — `: i32` — Method executed successfully.
- pub `STATUS_BUFFER_TOO_SMALL` variable L26 — `: i32` — Output buffer was too small (CallerAllocated/Arena strategies only).
- pub `STATUS_SERIALIZATION_ERROR` variable L30 — `: i32` — Serialization or deserialization failed at the FFI boundary.
- pub `STATUS_PLUGIN_ERROR` variable L34 — `: i32` — The plugin method returned an error.
- pub `STATUS_PANIC` variable L38 — `: i32` — A panic was caught at the `extern "C"` boundary via `catch_unwind`.
- pub `STATUS_STREAM_END` variable L43 — `: i32` — Clean end of a server-stream: the streaming `next()` shim has no more items

#### crates/fidius-guest/src/stream_ffi.rs

- pub `FidiusStreamHandle` struct L50-59 — `{ next: unsafe extern "C" fn(*mut FidiusStreamHandle, *mut u8, u32, *mut u32) ->...` — Per-stream handle returned by a cdylib streaming method's init shim.
- pub `NextStatus` enum L63-73 — `Item | End | TooSmall | SerErr` — Outcome of [`StreamState::next_into`] — mapped to FFI status codes by the
- pub `StreamState` struct L81-85 — `{ stream: crate::stream_marker::Stream<T>, pending: Option<T> }` — Guest-side driver for an arena-style cdylib stream (FIDIUS-T-0138).
- pub `new` function L89-94 — `(stream: crate::stream_marker::Stream<T>) -> Self` — Wrap a producer stream.
- pub `next_into` function L99-120 — `(&mut self, buf: &mut [u8]) -> NextStatus` — Pull the next item (if needed) and serialize it **directly into `buf`** —

#### crates/fidius-guest/src/stream_marker.rs

- pub `Stream` struct L61-65 — `{ iter: Option<Box<dyn Iterator<Item = T> + Send>>, _marker: PhantomData<fn() ->...` — Marker type a plugin interface uses to declare a **server-streaming** method:
- pub `new` function L70-75 — `() -> Self` — The marker form — declares a streaming method without producing items.
- pub `from_iter` function L81-90 — `(items: I) -> Self` — Build a stream from any iterator — how a Rust WASM guest produces its
- pub `next_item` function L95-97 — `(&mut self) -> Option<T>` — Advance the underlying iterator by one item.
-  `default` function L101-103 — `() -> Self` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
-  `tests` module L107-142 — `-` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
-  `from_iter_yields_then_none` function L111-118 — `()` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
-  `from_iter_accepts_a_range` function L121-125 — `()` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
-  `marker_form_is_empty` function L128-133 — `()` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).
-  `collect` function L135-141 — `(mut s: Stream<T>) -> Vec<T>` — The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).

#### crates/fidius-guest/src/value.rs

- pub `Value` enum L48-97 — `Bool | S8 | S16 | S32 | S64 | U8 | U16 | U32 | U64 | F32 | F64 | Char | String |...` — A self-describing value crossing the plugin-call boundary.
- pub `ValueError` struct L102 — `-` — Error produced while converting to or from [`Value`].
- pub `to_value` function L117-119 — `(value: &T) -> Result<Value, ValueError>` — Convert any [`Serialize`] type into a [`Value`].
- pub `from_value` function L122-127 — `(value: Value) -> Result<T, ValueError>` — Convert a [`Value`] into any [`Deserialize`] type.
-  `ValueError` type L104-108 — `= ValueError` — records, options, and variants.
-  `custom` function L105-107 — `(msg: T) -> Self` — records, options, and variants.
-  `ValueError` type L110-114 — `= ValueError` — records, options, and variants.
-  `custom` function L111-113 — `(msg: T) -> Self` — records, options, and variants.
-  `ValueSerializer` struct L133 — `-` — records, options, and variants.
-  `ValueSerializer` type L135-296 — `= ValueSerializer` — records, options, and variants.
-  `Ok` type L136 — `= Value` — records, options, and variants.
-  `Error` type L137 — `= ValueError` — records, options, and variants.
-  `SerializeSeq` type L139 — `= SeqSerializer` — records, options, and variants.
-  `SerializeTuple` type L140 — `= SeqSerializer` — records, options, and variants.
-  `SerializeTupleStruct` type L141 — `= SeqSerializer` — records, options, and variants.
-  `SerializeTupleVariant` type L142 — `= TupleVariantSerializer` — records, options, and variants.
-  `SerializeMap` type L143 — `= MapSerializer` — records, options, and variants.
-  `SerializeStruct` type L144 — `= StructSerializer` — records, options, and variants.
-  `SerializeStructVariant` type L145 — `= StructVariantSerializer` — records, options, and variants.
-  `serialize_bool` function L147-149 — `(self, v: bool) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_i8` function L150-152 — `(self, v: i8) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_i16` function L153-155 — `(self, v: i16) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_i32` function L156-158 — `(self, v: i32) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_i64` function L159-161 — `(self, v: i64) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_u8` function L162-164 — `(self, v: u8) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_u16` function L165-167 — `(self, v: u16) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_u32` function L168-170 — `(self, v: u32) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_u64` function L171-173 — `(self, v: u64) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_f32` function L174-176 — `(self, v: f32) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_f64` function L177-179 — `(self, v: f64) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_char` function L180-182 — `(self, v: char) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_str` function L183-185 — `(self, v: &str) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_bytes` function L186-188 — `(self, v: &[u8]) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_none` function L189-191 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_some` function L192-199 — `(self, value: &T) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_unit` function L200-202 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_unit_struct` function L203-205 — `(self, _name: &'static str) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_unit_variant` function L206-216 — `( self, _name: &'static str, _variant_index: u32, variant: &'static str, ) -> Re...` — records, options, and variants.
-  `serialize_newtype_struct` function L217-226 — `( self, _name: &'static str, value: &T, ) -> Result<Value, ValueError>` — records, options, and variants.
-  `serialize_newtype_variant` function L227-241 — `( self, _name: &'static str, _variant_index: u32, variant: &'static str, value: ...` — records, options, and variants.
-  `serialize_seq` function L242-246 — `(self, len: Option<usize>) -> Result<SeqSerializer, ValueError>` — records, options, and variants.
-  `serialize_tuple` function L247-249 — `(self, len: usize) -> Result<SeqSerializer, ValueError>` — records, options, and variants.
-  `serialize_tuple_struct` function L250-256 — `( self, _name: &'static str, len: usize, ) -> Result<SeqSerializer, ValueError>` — records, options, and variants.
-  `serialize_tuple_variant` function L257-268 — `( self, _name: &'static str, _variant_index: u32, variant: &'static str, len: us...` — records, options, and variants.
-  `serialize_map` function L269-274 — `(self, _len: Option<usize>) -> Result<MapSerializer, ValueError>` — records, options, and variants.
-  `serialize_struct` function L275-283 — `( self, _name: &'static str, len: usize, ) -> Result<StructSerializer, ValueErro...` — records, options, and variants.
-  `serialize_struct_variant` function L284-295 — `( self, _name: &'static str, _variant_index: u32, variant: &'static str, len: us...` — records, options, and variants.
-  `SeqSerializer` struct L298-300 — `{ items: Vec<Value> }` — records, options, and variants.
-  `SeqSerializer` type L301-314 — `= SeqSerializer` — records, options, and variants.
-  `Ok` type L302 — `= Value` — records, options, and variants.
-  `Error` type L303 — `= ValueError` — records, options, and variants.
-  `serialize_element` function L304-310 — `(&mut self, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L311-313 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `SeqSerializer` type L315-327 — `= SeqSerializer` — records, options, and variants.
-  `Ok` type L316 — `= Value` — records, options, and variants.
-  `Error` type L317 — `= ValueError` — records, options, and variants.
-  `serialize_element` function L318-323 — `(&mut self, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L324-326 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `SeqSerializer` type L328-340 — `= SeqSerializer` — records, options, and variants.
-  `Ok` type L329 — `= Value` — records, options, and variants.
-  `Error` type L330 — `= ValueError` — records, options, and variants.
-  `serialize_field` function L331-336 — `(&mut self, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L337-339 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `TupleVariantSerializer` struct L342-345 — `{ name: String, items: Vec<Value> }` — records, options, and variants.
-  `TupleVariantSerializer` type L346-362 — `= TupleVariantSerializer` — records, options, and variants.
-  `Ok` type L347 — `= Value` — records, options, and variants.
-  `Error` type L348 — `= ValueError` — records, options, and variants.
-  `serialize_field` function L349-355 — `(&mut self, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L356-361 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `MapSerializer` struct L364-367 — `{ entries: Vec<(Value, Value)>, next_key: Option<Value> }` — records, options, and variants.
-  `MapSerializer` type L368-410 — `= MapSerializer` — records, options, and variants.
-  `Ok` type L369 — `= Value` — records, options, and variants.
-  `Error` type L370 — `= ValueError` — records, options, and variants.
-  `serialize_key` function L371-377 — `(&mut self, key: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `serialize_value` function L378-388 — `(&mut self, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L389-409 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `StructSerializer` struct L412-414 — `{ fields: Vec<(String, Value)> }` — records, options, and variants.
-  `StructSerializer` type L415-429 — `= StructSerializer` — records, options, and variants.
-  `Ok` type L416 — `= Value` — records, options, and variants.
-  `Error` type L417 — `= ValueError` — records, options, and variants.
-  `serialize_field` function L418-425 — `(&mut self, key: &'static str, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L426-428 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `StructVariantSerializer` struct L431-434 — `{ name: String, fields: Vec<(String, Value)> }` — records, options, and variants.
-  `StructVariantSerializer` type L435-452 — `= StructVariantSerializer` — records, options, and variants.
-  `Ok` type L436 — `= Value` — records, options, and variants.
-  `Error` type L437 — `= ValueError` — records, options, and variants.
-  `serialize_field` function L438-445 — `(&mut self, key: &'static str, value: &T) -> Result<(), ValueError>` — records, options, and variants.
-  `end` function L446-451 — `(self) -> Result<Value, ValueError>` — records, options, and variants.
-  `Value` type L458-578 — `= Value` — records, options, and variants.
-  `Error` type L459 — `= ValueError` — records, options, and variants.
-  `deserialize_any` function L461-499 — `(self, visitor: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `deserialize_option` function L501-510 — `(self, visitor: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `deserialize_enum` function L512-537 — `( self, _name: &'static str, _variants: &'static [&'static str], visitor: V, ) -...` — records, options, and variants.
-  `deserialize_newtype_struct` function L539-548 — `( self, _name: &'static str, visitor: V, ) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `deserialize_unit_struct` function L550-559 — `( self, _name: &'static str, visitor: V, ) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `deserialize_unit` function L561-571 — `(self, visitor: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `Value` type L580-598 — `= Value` — records, options, and variants.
-  `kind` function L581-597 — `(&self) -> &'static str` — records, options, and variants.
-  `SeqAccess` struct L600-602 — `{ iter: std::vec::IntoIter<Value> }` — records, options, and variants.
-  `SeqAccess` type L603-617 — `= SeqAccess` — records, options, and variants.
-  `Error` type L604 — `= ValueError` — records, options, and variants.
-  `next_element_seed` function L605-613 — `(&mut self, seed: T) -> Result<Option<T::Value>, ValueError>` — records, options, and variants.
-  `size_hint` function L614-616 — `(&self) -> Option<usize>` — records, options, and variants.
-  `RecordAccess` struct L619-622 — `{ iter: std::vec::IntoIter<(String, Value)>, value: Option<Value> }` — records, options, and variants.
-  `RecordAccess` type L623-650 — `= RecordAccess` — records, options, and variants.
-  `Error` type L624 — `= ValueError` — records, options, and variants.
-  `next_key_seed` function L625-636 — `(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>` — records, options, and variants.
-  `next_value_seed` function L637-646 — `(&mut self, seed: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `size_hint` function L647-649 — `(&self) -> Option<usize>` — records, options, and variants.
-  `MapAccess` struct L652-655 — `{ iter: std::vec::IntoIter<(Value, Value)>, value: Option<Value> }` — records, options, and variants.
-  `MapAccess` type L656-683 — `= MapAccess` — records, options, and variants.
-  `Error` type L657 — `= ValueError` — records, options, and variants.
-  `next_key_seed` function L658-669 — `(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>` — records, options, and variants.
-  `next_value_seed` function L670-679 — `(&mut self, seed: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `size_hint` function L680-682 — `(&self) -> Option<usize>` — records, options, and variants.
-  `SingletonMapAccess` struct L687-690 — `{ key: Option<String>, value: Option<Value> }` — Presents a `Value::Variant` as a single-entry map for `deserialize_any`
-  `SingletonMapAccess` type L691-712 — `= SingletonMapAccess` — records, options, and variants.
-  `Error` type L692 — `= ValueError` — records, options, and variants.
-  `next_key_seed` function L693-701 — `(&mut self, seed: K) -> Result<Option<K::Value>, ValueError>` — records, options, and variants.
-  `next_value_seed` function L702-711 — `(&mut self, seed: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `EnumAccess` struct L714-717 — `{ name: String, value: Value }` — records, options, and variants.
-  `EnumAccess` type L718-728 — `= EnumAccess` — records, options, and variants.
-  `Error` type L719 — `= ValueError` — records, options, and variants.
-  `Variant` type L720 — `= VariantAccess` — records, options, and variants.
-  `variant_seed` function L721-727 — `(self, seed: V) -> Result<(V::Value, VariantAccess), ValueError>` — records, options, and variants.
-  `VariantAccess` struct L730-732 — `{ value: Value }` — records, options, and variants.
-  `VariantAccess` type L733-783 — `= VariantAccess` — records, options, and variants.
-  `Error` type L734 — `= ValueError` — records, options, and variants.
-  `unit_variant` function L735-743 — `(self) -> Result<(), ValueError>` — records, options, and variants.
-  `newtype_variant_seed` function L744-749 — `(self, seed: T) -> Result<T::Value, ValueError>` — records, options, and variants.
-  `tuple_variant` function L750-763 — `(self, _len: usize, visitor: V) -> Result<V::Value, ValueError>` — records, options, and variants.
-  `struct_variant` function L764-782 — `( self, _fields: &'static [&'static str], visitor: V, ) -> Result<V::Value, Valu...` — records, options, and variants.
-  `tests` module L786-898 — `-` — records, options, and variants.
-  `round_trip` function L790-797 — `(value: T)` — records, options, and variants.
-  `Greeting` struct L800-804 — `{ name: String, times: u32, loud: bool }` — records, options, and variants.
-  `Wrapper` struct L807 — `-` — records, options, and variants.
-  `Shape` enum L810-815 — `Unit | Newtype | Tuple | Struct` — records, options, and variants.
-  `primitives` function L818-828 — `()` — records, options, and variants.
-  `collections` function L831-837 — `()` — records, options, and variants.
-  `structs_and_maps` function L840-858 — `()` — records, options, and variants.
-  `enums` function L861-866 — `()` — records, options, and variants.
-  `nested` function L869-879 — `()` — records, options, and variants.
-  `Outer` struct L871-874 — `{ shapes: Vec<Shape>, tag: Option<String> }` — records, options, and variants.
-  `struct_shape_is_record` function L882-897 — `()` — records, options, and variants.
-  `Value` type L902-957 — `impl Serialize for Value` — records, options, and variants.
-  `serialize` function L903-956 — `(&self, serializer: S) -> Result<S::Ok, S::Error>` — records, options, and variants.
-  `Value` type L959-1050 — `= Value` — records, options, and variants.
-  `deserialize` function L960-1049 — `(deserializer: D) -> Result<Value, D::Error>` — records, options, and variants.
-  `ValueVisitor` struct L964 — `-` — records, options, and variants.
-  `ValueVisitor` type L965-1047 — `= ValueVisitor` — records, options, and variants.
-  `Value` type L966 — `= Value` — records, options, and variants.
-  `expecting` function L967-969 — `(&self, f: &mut fmt::Formatter) -> fmt::Result` — records, options, and variants.
-  `visit_bool` function L970-972 — `(self, v: bool) -> Result<Value, E>` — records, options, and variants.
-  `visit_i64` function L973-975 — `(self, v: i64) -> Result<Value, E>` — records, options, and variants.
-  `visit_i128` function L976-983 — `(self, v: i128) -> Result<Value, E>` — records, options, and variants.
-  `visit_u64` function L984-986 — `(self, v: u64) -> Result<Value, E>` — records, options, and variants.
-  `visit_u128` function L987-994 — `(self, v: u128) -> Result<Value, E>` — records, options, and variants.
-  `visit_f64` function L995-997 — `(self, v: f64) -> Result<Value, E>` — records, options, and variants.
-  `visit_char` function L998-1000 — `(self, v: char) -> Result<Value, E>` — records, options, and variants.
-  `visit_str` function L1001-1003 — `(self, v: &str) -> Result<Value, E>` — records, options, and variants.
-  `visit_string` function L1004-1006 — `(self, v: String) -> Result<Value, E>` — records, options, and variants.
-  `visit_bytes` function L1007-1009 — `(self, v: &[u8]) -> Result<Value, E>` — records, options, and variants.
-  `visit_byte_buf` function L1010-1012 — `(self, v: Vec<u8>) -> Result<Value, E>` — records, options, and variants.
-  `visit_unit` function L1013-1015 — `(self) -> Result<Value, E>` — records, options, and variants.
-  `visit_none` function L1016-1018 — `(self) -> Result<Value, E>` — records, options, and variants.
-  `visit_some` function L1019-1026 — `(self, deserializer: D) -> Result<Value, D::Error>` — records, options, and variants.
-  `visit_seq` function L1027-1036 — `(self, mut seq: A) -> Result<Value, A::Error>` — records, options, and variants.
-  `visit_map` function L1037-1046 — `(self, mut map: A) -> Result<Value, A::Error>` — records, options, and variants.

#### crates/fidius-guest/src/wasm_descriptor.rs

- pub `WasmInterfaceDescriptor` struct L27-40 — `{ interface_name: &'static str, interface_export: &'static str, interface_hash: ...` — Static descriptor for one fidius interface, consumed by the WASM loader to
- pub `WasmMethodDesc` struct L44-53 — `{ name: &'static str, wire_raw: bool, streaming: bool }` — One method on the interface.

#### crates/fidius-guest/src/wire.rs

- pub `WireError` enum L28-32 — `Bincode` — Errors that can occur during wire serialization or deserialization.
- pub `serialize` function L35-37 — `(val: &T) -> Result<Vec<u8>, WireError>` — Serialize a value as bincode for transport across the FFI boundary.
- pub `deserialize` function L40-42 — `(bytes: &[u8]) -> Result<T, WireError>` — Deserialize a value from bincode bytes received across the FFI boundary.
- pub `serialized_size` function L47-49 — `(val: &T) -> Result<u64, WireError>` — The exact serialized size of `val` in bytes, without allocating.
- pub `serialize_into` function L54-56 — `(buf: &mut [u8], val: &T) -> Result<(), WireError>` — Serialize `val` directly into a caller-provided buffer — no intermediate

### crates/fidius-guest/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-guest/tests/wasi_http_pin.rs

-  `PINNED` variable L24 — `: &str` — Drift tripwire (FIDIUS-A-0005).
-  `vendored_wasi_http_version_is_pinned` function L27-35 — `()` — `crates/fidius-guest/wit/` and update `PINNED` here in the same change.

### crates/fidius-host/benches

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-host/benches/backends.rs

-  `IFACE` variable L45 — `: &str` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `HASH` variable L46 — `: u64` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `METHODS` variable L47-68 — `: [WasmMethodDesc; 4]` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `GREETER` variable L69-74 — `: WasmInterfaceDescriptor` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `W_ADD` variable L76 — `: usize` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `W_ECHO` variable L77 — `: usize` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `T_IFACE` variable L85 — `: &str` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `T_HASH` variable L87 — `: u64` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `T_METHODS` variable L89-93 — `: [WasmMethodDesc; 1]` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `TICKER` variable L95-100 — `: WasmInterfaceDescriptor` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `C_ADD` variable L102 — `: usize` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `C_ECHO` variable L103 — `: usize` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `SIZES` variable L105 — `: &[usize]` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `greeter_component` function L107-117 — `() -> Vec<u8>` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `load_wasm` function L120-144 — `(host: &PluginHost, root: &std::path::Path, bytes: &[u8], aot: bool) -> PluginHa...` — Stage a wasm package dir (optionally with a precompiled `.cwasm`) and load it.
-  `compute` function L148-156 — `(op_is_add: bool, body: &[u8]) -> Vec<u8>` — The op a request asks the server to do.
-  `serve_lenprefix` function L161-179 — `(mut s: S)` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `lenprefix_call` function L181-191 — `(s: &mut S, op: u8, payload: &[u8]) -> Vec<u8>` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `spawn_tcp` function L193-203 — `() -> u16` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `spawn_uds` function L205-212 — `(path: PathBuf)` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `spawn_http` function L217-261 — `() -> u16` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `http_call` function L263-287 — `(s: &mut TcpStream, path: &str, payload: &[u8]) -> Vec<u8>` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `find_subslice` function L289-291 — `(hay: &[u8], needle: &[u8]) -> Option<usize>` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `content_length` function L293-301 — `(head: &str) -> usize` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `cdylib_handle` function L303-305 — `(host: &PluginHost, name: &str) -> PluginHandle` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `ticker_component` function L309-319 — `() -> Vec<u8>` — Build the (hand-authored) ticker streaming component for the per-item bench.
-  `stage_load_wasm_ticker` function L326-345 — `( host: &PluginHost, root: &std::path::Path, pkg: &str, bytes: &[u8], ) -> Plugi...` — Stage + load a ticker streaming **wasm** component (Rust or JS guest) as a
-  `ticker_component_file` function L350-352 — `(rel: &str) -> Option<Vec<u8>>` — A committed polyglot ticker component (JS/Python/C), if built.
-  `stage_load_python_ticker` function L358-381 — `(host: &PluginHost, root: &std::path::Path) -> PluginHandle` — Stage the py-ticker package (copy fixture + vendor the SDK + inject the macro
-  `copy_dir` function L384-396 — `(src: &std::path::Path, dst: &std::path::Path)` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.
-  `benches` function L398-580 — `(c: &mut Criterion)` — - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.

### crates/fidius-host

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-host/build.rs

-  `main` function L26-45 — `()` — Build script: when the `python` feature is enabled, embed a runtime

### crates/fidius-host/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-host/src/arch.rs

- pub `BinaryInfo` struct L26-29 — `{ format: BinaryFormat, arch: Arch }` — Detected binary format and architecture.
- pub `BinaryFormat` enum L32-37 — `Elf | MachO | Pe | Unknown` — architecture before attempting to dlopen.
- pub `Arch` enum L40-44 — `X86_64 | Aarch64 | Unknown` — architecture before attempting to dlopen.
- pub `detect_architecture` function L68-147 — `(path: &Path) -> Result<BinaryInfo, LoadError>` — Detect the binary format and architecture of a file.
- pub `check_architecture` function L150-185 — `(path: &Path) -> Result<(), LoadError>` — Check that a dylib matches the current platform's expected format.
-  `BinaryFormat` type L46-55 — `= BinaryFormat` — architecture before attempting to dlopen.
-  `fmt` function L47-54 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — architecture before attempting to dlopen.
-  `Arch` type L57-65 — `= Arch` — architecture before attempting to dlopen.
-  `fmt` function L58-64 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — architecture before attempting to dlopen.
-  `tests` module L188-243 — `-` — architecture before attempting to dlopen.
-  `detects_elf` function L192-204 — `()` — architecture before attempting to dlopen.
-  `detects_macho_le` function L207-219 — `()` — architecture before attempting to dlopen.
-  `detects_pe` function L222-231 — `()` — architecture before attempting to dlopen.
-  `unknown_format` function L234-242 — `()` — architecture before attempting to dlopen.

#### crates/fidius-host/src/arena.rs

- pub `DEFAULT_ARENA_CAPACITY` variable L27 — `: usize` — Default initial arena capacity (4 KB) when the pool is empty and a
- pub `acquire_arena` function L40-59 — `(min_capacity: usize) -> Vec<u8>` — Acquire an arena buffer with at least `min_capacity` bytes.
- pub `release_arena` function L64-66 — `(buf: Vec<u8>)` — Return an arena buffer to the pool for future reuse.
- pub `grow_arena` function L70-78 — `(buf: &mut Vec<u8>, needed_capacity: usize)` — Grow an in-flight arena buffer to hold at least `needed_capacity` bytes.

#### crates/fidius-host/src/error.rs

- pub `LoadError` enum L21-78 — `LibraryNotFound | SymbolNotFound | InvalidMagic | IncompatibleRegistryVersion | ...` — Errors that can occur when loading a plugin.
- pub `CallError` enum L82-143 — `Serialization | Deserialization | Plugin | Panic | BufferTooSmall | NotImplement...` — Errors that can occur when calling a plugin method.
-  `CallError` type L153-175 — `= CallError` — Fold the Python backend's call error into the unified [`CallError`].
-  `from` function L154-174 — `(e: fidius_python::PythonCallError) -> Self` — Error types for fidius-host plugin loading and calling.

#### crates/fidius-host/src/executor.rs

- pub `cdylib` module L42 — `-` — `PluginExecutor` — the dispatch seam across execution backends.
- pub `python` module L44 — `-` — bincode `call_method`, keeping the bytes byte-identical to pre-refactor.
- pub `wasm` module L46 — `-` — bincode `call_method`, keeping the bytes byte-identical to pre-refactor.
- pub `PluginExecutor` interface L66-77 — `{ fn info(), fn method_count(), fn call_raw() }` — The surface every execution backend shares.
- pub `ValueExecutor` interface L85-90 — `{ fn call() }` — Backends whose typed boundary is the self-describing [`Value`] model —

#### crates/fidius-host/src/handle.rs

- pub `PluginHandle` struct L68-70 — `{ backend: Backend }` — A handle to a loaded plugin, ready for calling methods.
- pub `from_loaded` function L74-78 — `(plugin: crate::loader::LoadedPlugin) -> Self` — Create a `PluginHandle` from a freshly loaded cdylib plugin.
- pub `from_descriptor` function L83-87 — `(desc: &'static PluginDescriptor) -> Result<Self, LoadError>` — Create a `PluginHandle` from a descriptor already registered in the
- pub `find_in_process_descriptor` function L91-95 — `( plugin_name: &str, ) -> Result<&'static PluginDescriptor, LoadError>` — Look up a descriptor in the current process's inventory registry by
- pub `from_python` function L101-105 — `(py: fidius_python::PythonPluginHandle, info: PluginInfo) -> Self` — Create a `PluginHandle` backed by a loaded Python plugin.
- pub `from_wasm` function L110-114 — `(executor: WasmComponentExecutor) -> Self` — Create a `PluginHandle` backed by a loaded WASM component.
- pub `call_method` function L121-149 — `( &self, index: usize, input: &I, ) -> Result<O, CallError>` — Call a plugin method by vtable index.
- pub `call_streaming` function L164-191 — `( &self, index: usize, input: &I, ) -> Result<crate::stream::ChunkStream, CallEr...` — Start a server-streaming method call by vtable index (FIDIUS-I-0026).
- pub `call_method_raw` function L194-202 — `(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError>` — Call a `#[wire(raw)]` method: raw bytes in, raw bytes out, no bincode.
- pub `has_capability` function L206-211 — `(&self, bit: u32) -> bool` — Check if an optional method is supported (capability bit set).
- pub `info` function L214-222 — `(&self) -> &PluginInfo` — Access the plugin's owned metadata.
- pub `method_metadata` function L227-236 — `(&self, method_id: u32) -> Vec<(&str, &str)>` — Static `#[method_meta(...)]` key/value metadata for the given method,
- pub `trait_metadata` function L240-248 — `(&self) -> Vec<(&str, &str)>` — Static `#[trait_meta(...)]` key/value metadata declared on the trait.
-  `Backend` enum L50-60 — `Cdylib | Python | Wasm` — The execution backend behind a [`PluginHandle`].
-  `PluginHandle` type L72-249 — `= PluginHandle` — refactor (`bincode(input)` straight to the FFI; `Value` is never involved).
-  `cdylib_stream_decode` function L257-263 — `( bytes: &[u8], ) -> Result<fidius_core::Value, CallError>` — Per-item decoder for the cdylib streaming fast path (FIDIUS-T-0137): each item

#### crates/fidius-host/src/host.rs

- pub `PluginHost` struct L31-43 — `{ search_paths: Vec<PathBuf>, load_policy: LoadPolicy, require_signature: bool, ...` — Host for loading and managing plugins.
- pub `PluginHostBuilder` struct L46-55 — `{ search_paths: Vec<PathBuf>, load_policy: LoadPolicy, require_signature: bool, ...` — Builder for configuring a PluginHost.
- pub `egress` function L77-80 — `(mut self, policy: impl crate::executor::wasm::EgressPolicy) -> Self` — Set a host-wide default `wasi:http` egress policy (FIDIUS-I-0027).
- pub `egress_policy` function L88-91 — `(mut self, policy: Arc<dyn crate::executor::wasm::EgressPolicy>) -> Self` — Like [`Self::egress`] but accepts an already-erased
- pub `search_path` function L94-97 — `(mut self, path: impl Into<PathBuf>) -> Self` — Add a directory to search for plugin dylibs.
- pub `load_policy` function L100-103 — `(mut self, policy: LoadPolicy) -> Self` — Set the load policy (Strict or Lenient).
- pub `require_signature` function L106-109 — `(mut self, require: bool) -> Self` — Require plugins to have valid signatures.
- pub `trusted_keys` function L112-115 — `(mut self, keys: &[VerifyingKey]) -> Self` — Set trusted Ed25519 public keys for signature verification.
- pub `interface_hash` function L118-121 — `(mut self, hash: u64) -> Self` — Set the expected interface hash for validation.
- pub `buffer_strategy` function L124-127 — `(mut self, strategy: BufferStrategyKind) -> Self` — Set the expected buffer strategy for validation.
- pub `build` function L130-141 — `(self) -> Result<PluginHost, LoadError>` — Build the PluginHost.
- pub `builder` function L146-148 — `() -> PluginHostBuilder` — Create a new builder.
- pub `discover` function L159-184 — `(&self) -> Result<Vec<PluginInfo>, LoadError>` — Discover all valid plugins in the configured search paths.
- pub `load` function L241-285 — `(&self, name: &str) -> Result<LoadedPlugin, LoadError>` — Load a specific plugin by name.
- pub `find_python_package` function L290-320 — `(&self, name: &str) -> Result<PathBuf, LoadError>` — Find a python plugin package directory by name across the configured
- pub `load_python` function L332-359 — `( &self, name: &str, descriptor: &'static fidius_core::python_descriptor::Python...` — Load a Python plugin package by name and validate it against the
- pub `find_wasm_package` function L364-390 — `(&self, name: &str) -> Result<PathBuf, LoadError>` — Find a WASM package directory by name across the search paths (matches
- pub `load_wasm` function L407-413 — `( &self, name: &str, descriptor: &'static fidius_core::wasm_descriptor::WasmInte...` — Load a WASM component plugin package by name and validate it against the
- pub `load_wasm_with_egress` function L421-428 — `( &self, name: &str, descriptor: &'static fidius_core::wasm_descriptor::WasmInte...` — Like [`Self::load_wasm`] but with a **per-plugin** `wasi:http` egress
-  `PluginHostBuilder` type L57-142 — `= PluginHostBuilder` — PluginHost builder and plugin discovery.
-  `new` function L58-69 — `() -> Self` — PluginHost builder and plugin discovery.
-  `PluginHost` type L144-544 — `= PluginHost` — PluginHost builder and plugin discovery.
-  `discover_cdylib` function L186-206 — `(&self, path: &Path, plugins: &mut Vec<PluginInfo>)` — PluginHost builder and plugin discovery.
-  `discover_package` function L211-235 — `(&self, dir: &Path, plugins: &mut Vec<PluginInfo>)` — Discover a directory-based package (`package.toml`) and surface it by
-  `load_wasm_impl` function L431-543 — `( &self, name: &str, descriptor: &'static fidius_core::wasm_descriptor::WasmInte...` — PluginHost builder and plugin discovery.
-  `is_dylib` function L547-556 — `(path: &Path) -> bool` — Check if a path has a platform-appropriate dylib extension.

#### crates/fidius-host/src/lib.rs

- pub `arch` module L15 — `-`
- pub `arena` module L16 — `-`
- pub `error` module L17 — `-`
- pub `executor` module L18 — `-`
- pub `handle` module L19 — `-`
- pub `host` module L20 — `-`
- pub `loader` module L21 — `-`
- pub `package` module L22 — `-`
- pub `signing` module L23 — `-`
- pub `stream` module L25 — `-`
- pub `types` module L26 — `-`

#### crates/fidius-host/src/loader.rs

- pub `LoadedLibrary` struct L28-33 — `{ library: Arc<Library>, plugins: Vec<LoadedPlugin> }` — A loaded plugin library with validated descriptors.
- pub `LoadedPlugin` struct L36-51 — `{ info: PluginInfo, vtable: *const c_void, free_buffer: Option<unsafe extern "C"...` — A single validated plugin from a loaded library.
- pub `load_library` function L71-124 — `(path: &Path) -> Result<LoadedLibrary, LoadError>` — Load a plugin library from a path.
- pub `validate_against_interface` function L166-190 — `( plugin: &LoadedPlugin, expected_hash: Option<u64>, expected_strategy: Option<B...` — Validate a loaded plugin against expected interface parameters.
-  `LoadedPlugin` type L55 — `impl Send for LoadedPlugin` — Core plugin loading and descriptor validation.
-  `LoadedPlugin` type L56 — `impl Sync for LoadedPlugin` — Core plugin loading and descriptor validation.
-  `LoadedPlugin` type L58-65 — `= LoadedPlugin` — Core plugin loading and descriptor validation.
-  `fmt` function L59-64 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core plugin loading and descriptor validation.
-  `validate_descriptor` function L127-163 — `( desc: &PluginDescriptor, library: &Arc<Library>, ) -> Result<LoadedPlugin, Loa...` — Validate a single descriptor and copy to owned types.

#### crates/fidius-host/src/package.rs

- pub `load_package_manifest` function L41-45 — `( dir: &Path, ) -> Result<PackageManifest<M>, PackageError>` — Load and validate a package manifest against a host-defined schema.
- pub `discover_packages` function L51-70 — `(dir: &Path) -> Result<Vec<PathBuf>, PackageError>` — Discover packages in a directory.
- pub `verify_package` function L81-108 — `(dir: &Path, trusted_keys: &[VerifyingKey]) -> Result<(), PackageError>` — Verify a source package's signature against trusted public keys.
- pub `unpack_fid` function L127-139 — `(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError>` — Extract a `.fid` archive and validate its contents.
- pub `build_package` function L144-193 — `(dir: &Path, release: bool) -> Result<PathBuf, PackageError>` — Build a package by running `cargo build` inside the package directory.

#### crates/fidius-host/src/signing.rs

- pub `sig_path_for` function L27-32 — `(path: &Path) -> std::path::PathBuf` — Compute the detached signature file path for a given file.
- pub `verify_signature` function L43-74 — `(dylib_path: &Path, trusted_keys: &[VerifyingKey]) -> Result<(), LoadError>` — Verify a plugin dylib's signature against trusted public keys.
- pub `verify_package_signature` function L84-117 — `( dir: &Path, trusted_keys: &[VerifyingKey], ) -> Result<(), LoadError>` — Verify a **package** signature: `package.sig` in `dir`, an Ed25519 signature
-  `tests` module L120-189 — `-` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `create_test_file` function L126-130 — `(content: &[u8]) -> NamedTempFile` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `sign_file` function L132-140 — `(path: &Path, signing_key: &SigningKey)` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `valid_signature_succeeds` function L143-152 — `()` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `tampered_file_fails` function L155-167 — `()` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `wrong_key_fails` function L170-179 — `()` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).
-  `missing_sig_file_returns_required` function L182-188 — `()` — packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).

#### crates/fidius-host/src/stream.rs

- pub `ChunkStream` struct L57-59 — `{ inner: Pin<Box<dyn Stream<Item = Result<Value, CallError>> + Send>> }` — Host-facing pull handle for a server-streaming plugin call.
- pub `new` function L64-71 — `(stream: S) -> Self` — Wrap any item stream as a [`ChunkStream`].
- pub `from_frame_bytes` function L95-127 — `(frames: S, decode_item: D) -> Self` — Build a [`ChunkStream`] from a stream of raw, length-delimited frame
- pub `from_frames` function L133-142 — `(frames: Vec<Frame>, decode_item: D) -> Self` — Build a [`ChunkStream`] over a fixed, in-memory sequence of [`Frame`]s.
- pub `StreamExecutor` interface L161-166 — `{ fn call_streaming() }` — Backends whose typed boundary can produce a **server-streaming** result.
-  `ChunkStream` type L61-143 — `= ChunkStream` — turns that byte sequence into the item stream every backend bridge feeds.
-  `ChunkStream` type L145-151 — `impl Stream for ChunkStream` — turns that byte sequence into the item stream every backend bridge feeds.
-  `Item` type L146 — `= Result<Value, CallError>` — turns that byte sequence into the item stream every backend bridge feeds.
-  `poll_next` function L148-150 — `(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>` — turns that byte sequence into the item stream every backend bridge feeds.
-  `tests` module L169-267 — `-` — turns that byte sequence into the item stream every backend bridge feeds.
-  `item` function L176-178 — `(v: i64) -> Frame` — An ITEM frame carrying a concrete `i64` (bincode of a concrete type
-  `decode_i64` function L181-185 — `(b: &[u8]) -> Result<Value, CallError>` — The matching item decoder: concrete-bincode `i64` → `Value`.
-  `collect` function L187-193 — `(mut s: ChunkStream) -> Vec<Result<Value, CallError>>` — turns that byte sequence into the item stream every backend bridge feeds.
-  `items_then_clean_end` function L196-204 — `()` — turns that byte sequence into the item stream every backend bridge feeds.
-  `native_value_stream_via_new` function L207-218 — `()` — turns that byte sequence into the item stream every backend bridge feeds.
-  `error_frame_terminates_after_one_err` function L221-234 — `()` — turns that byte sequence into the item stream every backend bridge feeds.
-  `missing_terminal_is_abort` function L237-244 — `()` — turns that byte sequence into the item stream every backend bridge feeds.
-  `malformed_frame_surfaces_then_stops` function L247-260 — `()` — turns that byte sequence into the item stream every backend bridge feeds.
-  `empty_stream_just_ends` function L263-266 — `()` — turns that byte sequence into the item stream every backend bridge feeds.

#### crates/fidius-host/src/types.rs

- pub `PluginRuntimeKind` enum L23-34 — `Cdylib | Python | Wasm` — Plugin runtime kind.
- pub `PluginInfo` struct L43-59 — `{ name: String, interface_name: String, interface_hash: u64, interface_version: ...` — Owned metadata for a discovered or loaded plugin.
- pub `is_cdylib` function L63-65 — `(&self) -> bool` — True if this is a cdylib-backed plugin.
- pub `is_python` function L68-70 — `(&self) -> bool` — True if this is a Python plugin.
- pub `is_wasm` function L73-75 — `(&self) -> bool` — True if this is a WASM component plugin.
- pub `LoadPolicy` enum L80-86 — `Strict | Lenient` — Controls how strictly the host validates plugins.
-  `PluginInfo` type L61-76 — `= PluginInfo` — Owned metadata types for loaded plugins.

### crates/fidius-host/src/executor

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-host/src/executor/cdylib.rs

- pub `CdylibExecutor` struct L71-96 — `{ _library: Option<Arc<Library>>, vtable: *const c_void, descriptor: *const Plug...` — A handle to a loaded plugin, ready for calling methods.
- pub `from_loaded` function L148-162 — `(plugin: crate::loader::LoadedPlugin) -> Self` — Create a CdylibExecutor from a LoadedPlugin.
- pub `from_descriptor` function L171-196 — `(desc: &'static PluginDescriptor) -> Result<Self, LoadError>` — Create a CdylibExecutor from a plugin descriptor already registered in
- pub `find_in_process_descriptor` function L204-218 — `( plugin_name: &str, ) -> Result<&'static PluginDescriptor, LoadError>` — Look up a descriptor in the current process's inventory registry by
- pub `call_method` function L236-256 — `( &self, index: usize, input: &I, ) -> Result<O, CallError>` — Call a plugin method by vtable index.
- pub `call_method_raw` function L267-278 — `(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError>` — Call a plugin method whose argument and successful return value are
- pub `call_streaming_raw` function L637-772 — `( &self, index: usize, input_bytes: &[u8], decode_item: fn(&[u8]) -> Result<fidi...` — Start a server-streaming cdylib call (FIDIUS-I-0026 CS.1).
- pub `has_capability` function L777-782 — `(&self, bit: u32) -> bool` — Check if an optional method is supported (capability bit is set).
- pub `info` function L785-787 — `(&self) -> &PluginInfo` — Access the plugin's owned metadata.
- pub `method_metadata` function L800-832 — `(&self, method_id: u32) -> Vec<(&str, &str)>` — Returns the static key/value metadata declared on the given method via
- pub `trait_metadata` function L838-859 — `(&self) -> Vec<(&str, &str)>` — Returns the static key/value metadata declared on the trait via
-  `FfiFn` type L45 — `= unsafe extern "C" fn(*mut c_void, *const u8, u32, *mut *mut u8, *mut u32) -> i...` — Type alias for the PluginAllocated FFI function pointer signature.
-  `ArenaFn` type L48-49 — `= unsafe extern "C" fn(*mut c_void, *const u8, u32, *mut u8, u32, *mut u32, *mut...` — Type alias for the Arena FFI function pointer signature.
-  `construct_instance` function L56-61 — `(descriptor: *const PluginDescriptor) -> *mut c_void` — Construct the plugin instance via the descriptor's `construct` (FIDIUS-A-0006).
-  `CdylibExecutor` type L106 — `impl Send for CdylibExecutor` — (and future WASM) backends.
-  `CdylibExecutor` type L107 — `impl Sync for CdylibExecutor` — (and future WASM) backends.
-  `CdylibExecutor` type L109-118 — `impl Drop for CdylibExecutor` — (and future WASM) backends.
-  `drop` function L110-117 — `(&mut self)` — (and future WASM) backends.
-  `CdylibExecutor` type L120-860 — `= CdylibExecutor` — (and future WASM) backends.
-  `new` function L123-145 — `( library: Arc<Library>, vtable: *const c_void, descriptor: *const PluginDescrip...` — Create a new CdylibExecutor.
-  `call_plugin_allocated` function L282-361 — `( &self, index: usize, input_bytes: &[u8], ) -> Result<O, CallError>` — PluginAllocated path: plugin allocates an output buffer via
-  `call_arena` function L367-454 — `( &self, index: usize, input_bytes: &[u8], ) -> Result<O, CallError>` — Arena path: host supplies a buffer from the thread-local pool.
-  `call_plugin_allocated_raw` function L459-538 — `( &self, index: usize, input_bytes: &[u8], ) -> Result<Vec<u8>, CallError>` — PluginAllocated raw path — same FFI shape as `call_plugin_allocated`,
-  `call_arena_raw` function L542-620 — `(&self, index: usize, input_bytes: &[u8]) -> Result<Vec<u8>, CallError>` — Arena raw path — same FFI shape as `call_arena`, success bytes
-  `STREAM_CHANNEL_CAP` variable L648 — `: usize` — Bounded backpressure/memory window between the pump thread and the
-  `SendHandle` struct L689 — `-` — (and future WASM) backends.
-  `SendHandle` type L690 — `impl Send for SendHandle` — (and future WASM) backends.
-  `INITIAL_ITEM_CAP` variable L705 — `: usize` — (and future WASM) backends.
-  `CdylibExecutor` type L862-878 — `impl PluginExecutor for CdylibExecutor` — (and future WASM) backends.
-  `info` function L863-865 — `(&self) -> &PluginInfo` — (and future WASM) backends.
-  `method_count` function L867-869 — `(&self) -> u32` — (and future WASM) backends.
-  `call_raw` function L875-877 — `(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>` — Raw byte dispatch.

#### crates/fidius-host/src/executor/python.rs

- pub `Pyo3Executor` struct L39-42 — `{ py: PythonPluginHandle, info: PluginInfo }` — Python-backed executor: an embedded-interpreter plugin handle plus the
- pub `new` function L46-48 — `(py: PythonPluginHandle, info: PluginInfo) -> Self` — Wrap a loaded `PythonPluginHandle` with its owned metadata.
-  `Pyo3Executor` type L44-49 — `= Pyo3Executor` — routed through the neutral `Value` currency.
-  `Pyo3Executor` type L51-64 — `impl PluginExecutor for Pyo3Executor` — routed through the neutral `Value` currency.
-  `info` function L52-54 — `(&self) -> &PluginInfo` — routed through the neutral `Value` currency.
-  `method_count` function L56-58 — `(&self) -> u32` — routed through the neutral `Value` currency.
-  `call_raw` function L60-63 — `(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>` — routed through the neutral `Value` currency.
-  `Pyo3Executor` type L66-79 — `impl ValueExecutor for Pyo3Executor` — routed through the neutral `Value` currency.
-  `call` function L67-78 — `(&self, method: usize, args: Value) -> Result<Value, CallError>` — routed through the neutral `Value` currency.
-  `STREAM_CHANNEL_CAP` variable L86 — `: usize` — Bounded channel depth between the GIL-holding pump thread and the host's
-  `Pyo3Executor` type L90-150 — `= Pyo3Executor` — routed through the neutral `Value` currency.
-  `call_streaming` function L91-149 — `( &self, method: usize, args: Value, ) -> Result<crate::stream::ChunkStream, Cal...` — routed through the neutral `Value` currency.

#### crates/fidius-host/src/executor/wasm.rs

- pub `EgressDenied` struct L54-58 — `{ reason: String }` — Denial returned by an [`EgressPolicy`] to refuse an outbound request.
- pub `new` function L62-66 — `(reason: impl Into<String>) -> Self` — A denial with a reason.
- pub `EgressPolicy` interface L82-85 — `{ fn authorize() }` — Embedder-supplied policy governing a sandboxed WASM guest's **outbound HTTP**
- pub `WasmMethod` struct L332-340 — `{ name: String, wire_raw: bool, streaming: bool }` — A method on the WASM interface, in declaration (vtable) order.
- pub `WasmComponentExecutor` struct L343-362 — `{ engine: Engine, instance_pre: InstancePre<HostState>, interface: String, metho...` — WASM component execution backend.
- pub `from_component_bytes` function L367-375 — `( bytes: &[u8], interface: String, methods: Vec<WasmMethod>, capabilities: Vec<S...` — Build an executor from raw component bytes (a `.wasm` component).
- pub `from_component_bytes_with_egress` function L380-403 — `( bytes: &[u8], interface: String, methods: Vec<WasmMethod>, capabilities: Vec<S...` — Like [`Self::from_component_bytes`] but with an embedder [`EgressPolicy`]
- pub `from_cwasm` function L411-419 — `( cwasm: &[u8], interface: String, methods: Vec<WasmMethod>, capabilities: Vec<S...` — Build from a precompiled `.cwasm` (engine/version-specific).
- pub `from_cwasm_with_egress` function L427-450 — `( cwasm: &[u8], interface: String, methods: Vec<WasmMethod>, capabilities: Vec<S...` — Like [`Self::from_cwasm`] but with an embedder [`EgressPolicy`]
- pub `interface_hash` function L585-601 — `(&self) -> Result<u64, CallError>` — Call the `fidius-interface-hash` export — the integrity check the loader
- pub `validate_component` function L952-960 — `(bytes: &[u8]) -> Result<(), CallError>` — Validate that `bytes` is a well-formed WASM **component** (Component Model),
- pub `precompile_component` function L966-974 — `(bytes: &[u8]) -> Result<Vec<u8>, CallError>` — Ahead-of-time compile a component into engine/version-specific `.cwasm`
-  `EgressDenied` type L60-67 — `= EgressDenied` — from the package manifest's allow-list.
-  `EgressHooks` struct L92-94 — `{ policy: Option<Arc<dyn EgressPolicy>> }` — fidius's [`WasiHttpHooks`] adapter: routes every outbound request through the
-  `EgressHooks` type L96-116 — `impl WasiHttpHooks for EgressHooks` — from the package manifest's allow-list.
-  `send_request` function L97-115 — `( &mut self, request: http::Request<HyperOutgoingBody>, config: OutgoingRequestC...` — from the package manifest's allow-list.
-  `HostState` struct L121-126 — `{ ctx: WasiCtx, table: ResourceTable, http_ctx: WasiHttpCtx, hooks: EgressHooks ...` — Per-store host state.
-  `HostState` type L128-136 — `impl WasiHttpView for HostState` — from the package manifest's allow-list.
-  `http` function L129-135 — `(&mut self) -> WasiHttpCtxView<'_>` — from the package manifest's allow-list.
-  `KNOWN_CAPABILITIES` variable L142-151 — `: &[&str]` — Capabilities the host knows how to grant.
-  `validate_capabilities` function L155-191 — `(caps: &[String]) -> Result<(), CallError>` — Reject unknown capability names early (at load) so a typo fails closed and
-  `build_wasi_ctx` function L196-244 — `(caps: &[String]) -> WasiCtx` — Build a `WasiCtx` from the allow-list.
-  `is_blocked_ip` function L251-270 — `(ip: &IpAddr) -> bool` — Baseline SSRF denylist for the raw-socket grant (FIDIUS-T-0143): an address a
-  `HOST_WASI_HTTP` variable L275 — `: (u32, u32, u32)` — The `wasi:http` version this host provides — matched to `wasmtime-wasi-http`
-  `wasi_http_incompatibility` function L287-317 — `(import_names: impl Iterator<Item = &'a str>) -> Option<String>` — Scan a component's import names for a `wasi:http` version this host can't
-  `HostState` type L321-328 — `impl WasiView for HostState` — from the package manifest's allow-list.
-  `ctx` function L322-327 — `(&mut self) -> WasiCtxView<'_>` — from the package manifest's allow-list.
-  `WasmComponentExecutor` type L364-602 — `= WasmComponentExecutor` — from the package manifest's allow-list.
-  `build` function L454-510 — `( engine: Engine, component: &Component, interface: String, methods: Vec<WasmMet...` — Shared constructor: wire WASI into a `Linker` and pre-instantiate the
-  `instantiate` function L515-533 — `(&self) -> Result<(Store<HostState>, wasmtime::component::Instance), CallError>` — Instantiate a fresh sandboxed `Store` + component instance from the cached
-  `func` function L536-563 — `( &self, store: &mut Store<HostState>, instance: &wasmtime::component::Instance,...` — Resolve an exported function within the plugin's interface by name.
-  `method` function L565-581 — `(&self, index: usize, want_raw: bool) -> Result<&WasmMethod, CallError>` — from the package manifest's allow-list.
-  `WasmComponentExecutor` type L604-641 — `impl PluginExecutor for WasmComponentExecutor` — from the package manifest's allow-list.
-  `info` function L605-607 — `(&self) -> &PluginInfo` — from the package manifest's allow-list.
-  `method_count` function L609-611 — `(&self) -> u32` — from the package manifest's allow-list.
-  `call_raw` function L613-640 — `(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>` — from the package manifest's allow-list.
-  `WasmComponentExecutor` type L643-675 — `impl ValueExecutor for WasmComponentExecutor` — from the package manifest's allow-list.
-  `call` function L644-674 — `(&self, method: usize, args: Value) -> Result<Value, CallError>` — from the package manifest's allow-list.
-  `STREAM_CHANNEL_CAP` variable L681 — `: usize` — Bounded channel depth between the wasmtime pump thread and the async
-  `WasmComponentExecutor` type L685-788 — `= WasmComponentExecutor` — from the package manifest's allow-list.
-  `call_streaming` function L686-787 — `( &self, method: usize, args: Value, ) -> Result<crate::stream::ChunkStream, Cal...` — from the package manifest's allow-list.
-  `plugin_error_from_val` function L792-818 — `(payload: Option<&Val>) -> CallError` — Map a `result::err` payload (expected: a record with `code`/`message`/
-  `to_kebab` function L823-838 — `(s: &str) -> String` — fidius `Value` → wasmtime `Val`.
-  `kebab_to_snake` function L841-843 — `(s: &str) -> String` — kebab-case → snake_case (WIT record field → serde struct field).
-  `kebab_to_pascal` function L846-856 — `(s: &str) -> String` — kebab-case → PascalCase (WIT variant case → serde enum variant).
-  `value_to_val` function L858-901 — `(v: &Value) -> Result<Val, CallError>` — from the package manifest's allow-list.
-  `val_to_value` function L904-942 — `(v: &Val) -> Value` — wasmtime `Val` → fidius `Value` (structural; self-describing).
-  `ssrf_tests` module L977-1015 — `-` — from the package manifest's allow-list.
-  `ip` function L981-983 — `(s: &str) -> IpAddr` — from the package manifest's allow-list.
-  `blocks_internal_and_metadata_targets` function L986-1002 — `()` — from the package manifest's allow-list.
-  `allows_public_targets` function L1005-1014 — `()` — from the package manifest's allow-list.
-  `wasi_http_version_tests` module L1018-1053 — `-` — from the package manifest's allow-list.
-  `host_matched_version_is_compatible` function L1022-1028 — `()` — from the package manifest's allow-list.
-  `newer_minor_or_patch_is_rejected_with_a_clear_message` function L1031-1043 — `()` — from the package manifest's allow-list.
-  `no_wasi_http_import_is_fine` function L1046-1052 — `()` — from the package manifest's allow-list.

### crates/fidius-host/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-host/tests/cdylib_streaming_e2e.rs

-  `ticker_handle` function L31-43 — `() -> PluginHandle` — streaming peer alongside Python and WASM.
-  `cdylib_stream_yields_all_items` function L46-57 — `()` — streaming peer alongside Python and WASM.
-  `cdylib_empty_stream` function L60-67 — `()` — streaming peer alongside Python and WASM.
-  `cdylib_huge_stream_is_bounded_and_cancellable` function L70-84 — `()` — streaming peer alongside Python and WASM.

#### crates/fidius-host/tests/e2e.rs

-  `plugin_source_dir` function L22-24 — `() -> PathBuf` — End-to-end validation tests: signing, negative cases.
-  `plugin_dir` function L27-35 — `() -> &'static Path` — Cached plugin build directory — same fixture shared across all e2e tests.
-  `DIR` variable L28 — `: std::sync::OnceLock<PathBuf>` — End-to-end validation tests: signing, negative cases.
-  `dylib_path` function L37-46 — `() -> PathBuf` — End-to-end validation tests: signing, negative cases.
-  `cleanup_sig` function L48-53 — `()` — End-to-end validation tests: signing, negative cases.
-  `signed_plugin_loads_with_correct_key` function L57-72 — `()` — End-to-end validation tests: signing, negative cases.
-  `signed_plugin_fails_with_wrong_key` function L76-96 — `()` — End-to-end validation tests: signing, negative cases.
-  `unsigned_plugin_fails_when_signature_required` function L100-118 — `()` — End-to-end validation tests: signing, negative cases.
-  `unsigned_plugin_loads_without_signature_requirement` function L122-147 — `()` — End-to-end validation tests: signing, negative cases.
-  `AddInput` struct L134-137 — `{ a: i64, b: i64 }` — End-to-end validation tests: signing, negative cases.
-  `AddOutput` struct L139-141 — `{ result: i64 }` — End-to-end validation tests: signing, negative cases.
-  `lenient_policy_still_enforces_signatures` function L151-172 — `()` — End-to-end validation tests: signing, negative cases.
-  `lenient_policy_still_rejects_wrong_key` function L176-197 — `()` — End-to-end validation tests: signing, negative cases.

#### crates/fidius-host/tests/integration.rs

-  `plugin_source_dir` function L29-31 — `() -> PathBuf` — capability / info assertions where the Client abstracts them away.
-  `plugin_dir` function L34-45 — `() -> &'static Path` — Directory containing the cached-built test plugin cdylib.
-  `DIR` variable L38 — `: std::sync::OnceLock<PathBuf>` — capability / info assertions where the Client abstracts them away.
-  `client` function L48-57 — `() -> CalculatorClient` — Build a client from the built+loaded plugin.
-  `discover_finds_plugin` function L60-73 — `()` — capability / info assertions where the Client abstracts them away.
-  `load_plugin_by_name` function L76-85 — `()` — capability / info assertions where the Client abstracts them away.
-  `call_add_method_via_client` function L88-92 — `()` — capability / info assertions where the Client abstracts them away.
-  `call_multiply_method_via_client` function L95-100 — `()` — capability / info assertions where the Client abstracts them away.
-  `call_multi_arg_add_direct_via_client` function L103-107 — `()` — capability / info assertions where the Client abstracts them away.
-  `call_zero_arg_version_via_client` function L110-114 — `()` — capability / info assertions where the Client abstracts them away.
-  `plugin_info_is_correct` function L117-133 — `()` — capability / info assertions where the Client abstracts them away.
-  `load_nonexistent_plugin_returns_not_found` function L136-144 — `()` — capability / info assertions where the Client abstracts them away.
-  `out_of_bounds_vtable_index_returns_error` function L147-169 — `()` — capability / info assertions where the Client abstracts them away.
-  `Dummy` struct L157 — `-` — capability / info assertions where the Client abstracts them away.
-  `raw_wire_method_round_trips` function L172-192 — `()` — capability / info assertions where the Client abstracts them away.
-  `raw_wire_method_handles_large_payload` function L195-212 — `()` — capability / info assertions where the Client abstracts them away.
-  `arena_plugin_loads_and_round_trips` function L215-231 — `()` — capability / info assertions where the Client abstracts them away.
-  `arena_plugin_grows_buffer_on_too_small_retry` function L234-256 — `()` — capability / info assertions where the Client abstracts them away.
-  `trait_and_method_metadata_readable_through_handle` function L259-285 — `()` — capability / info assertions where the Client abstracts them away.
-  `has_capability_returns_false_for_high_bits` function L288-302 — `()` — capability / info assertions where the Client abstracts them away.
-  `discover_surfaces_wasm_package_with_wasm_runtime` function L308-347 — `()` — Routing reserves the WASM seat (FIDIUS-I-0021 Phase 1): a `runtime = "wasm"`

#### crates/fidius-host/tests/macro_egress_e2e.rs

- pub `Fetcher` interface L40-42 — `{ fn fetch() }` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `macro_fetcher_component` function L45-59 — `() -> &'static [u8]` — Build the macro-fetcher component once.
-  `BYTES` variable L46 — `: OnceLock<Vec<u8>>` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `mock_http_once` function L62-80 — `(body: &'static str) -> (String, std::thread::JoinHandle<()>)` — One-shot loopback mock HTTP server serving a single request with `body`.
-  `AllowAll` struct L82 — `-` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `AllowAll` type L83-87 — `impl EgressPolicy for AllowAll` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `authorize` function L84-86 — `(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied>` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `DenyAll` struct L89 — `-` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `DenyAll` type L90-94 — `impl EgressPolicy for DenyAll` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `authorize` function L91-93 — `(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied>` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `stage_pkg` function L97-108 — `(root: &std::path::Path)` — Stage the macro-fetcher as a `runtime = "wasm"` package declaring `http`.
-  `macro_connector_egress_allowed` function L111-129 — `()` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `macro_connector_egress_denied` function L132-154 — `()` — wasi:http `generate!` compose, and that the result rides the two-key gate.
-  `macro_connector_no_policy_fails_closed` function L157-173 — `()` — wasi:http `generate!` compose, and that the result rides the two-key gate.

#### crates/fidius-host/tests/macro_wasm.rs

- pub `Greeter` interface L42-47 — `{ fn greet(), fn echo() }` — here via a separate `cargo build --target wasm32-wasip2` invocation.
-  `macro_greeter_component` function L50-64 — `() -> &'static [u8]` — Build the macro-greeter component once and return its bytes.
-  `BYTES` variable L51 — `: OnceLock<Vec<u8>>` — here via a separate `cargo build --target wasm32-wasip2` invocation.
-  `stage_pkg` function L67-89 — `(root: &std::path::Path)` — Stage a `runtime = "wasm"` package containing the built component.
-  `macro_built_component_loads_and_calls` function L92-118 — `()` — here via a separate `cargo build --target wasm32-wasip2` invocation.
-  `macro_descriptor_export_and_hash_are_self_consistent` function L121-131 — `()` — here via a separate `cargo build --target wasm32-wasip2` invocation.

#### crates/fidius-host/tests/macro_wasm_streaming.rs

- pub `Ticker` interface L37-39 — `{ fn tick() }` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `macro_ticker_component` function L41-55 — `() -> &'static [u8]` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `BYTES` variable L42 — `: OnceLock<Vec<u8>>` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `stage_pkg` function L57-79 — `(root: &std::path::Path)` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `macro_descriptor_marks_tick_streaming` function L82-91 — `()` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `macro_streaming_component_loads_and_streams` function L94-116 — `()` — Requires the wasm component toolchain (cargo + wasm32-wasip2).
-  `macro_streaming_bounded_and_cancellable` function L119-141 — `()` — Requires the wasm component toolchain (cargo + wasm32-wasip2).

#### crates/fidius-host/tests/package_e2e.rs

-  `test_package_dir` function L23-25 — `() -> PathBuf` — End-to-end package tests: validate, build, load, call.
-  `TestSchema` struct L28-31 — `{ category: String, description: String }` — End-to-end package tests: validate, build, load, call.
-  `StrictSchema` struct L34-38 — `{ category: String, description: String, required_field: String }` — End-to-end package tests: validate, build, load, call.
-  `load_manifest_with_schema` function L41-51 — `()` — End-to-end package tests: validate, build, load, call.
-  `schema_mismatch_fails` function L54-64 — `()` — End-to-end package tests: validate, build, load, call.
-  `build_and_load_package` function L67-105 — `()` — End-to-end package tests: validate, build, load, call.
-  `AddInput` struct L94-97 — `{ a: i64, b: i64 }` — End-to-end package tests: validate, build, load, call.
-  `AddOutput` struct L99-101 — `{ result: i64 }` — End-to-end package tests: validate, build, load, call.
-  `discover_packages_finds_fixture` function L108-122 — `()` — End-to-end package tests: validate, build, load, call.
-  `missing_manifest_returns_error` function L125-129 — `()` — End-to-end package tests: validate, build, load, call.

#### crates/fidius-host/tests/plugin_dep_graph.rs

-  `plugin_without_host_feature_does_not_pull_libloading` function L26-65 — `()` — and asserts `libloading` is not in its dep graph.

#### crates/fidius-host/tests/python_plugin_e2e.rs

-  `stage_plugin` function L51-66 — `(tmp: &tempfile::TempDir) -> PathBuf` — Directory structure mirrors what a deployer would have:
-  `repo_root` function L68-75 — `() -> PathBuf` — 5.
-  `copy_dir` function L77-89 — `(src: &std::path::Path, dst: &std::path::Path)` — 5.
-  `byte_pipe_descriptor` function L95-97 — `() -> &'static PythonInterfaceDescriptor` — Produce the BytePipe descriptor from the Rust trait via the macro-emitted
-  `discover_lists_python_plugin_with_python_runtime` function L100-114 — `()` — 5.
-  `typed_method_round_trips` function L117-130 — `()` — 5.
-  `raw_wire_method_round_trips_2mb` function L133-152 — `()` — 5.
-  `tampered_interface_hash_is_rejected_at_load` function L155-190 — `()` — 5.

#### crates/fidius-host/tests/python_routing.rs

-  `HASH` variable L26 — `: u64` — when the `python` feature is enabled.
-  `METHODS` variable L27-30 — `: [PythonMethodDesc; 1]` — when the `python` feature is enabled.
-  `fresh_descriptor` function L32-44 — `() -> (&'static PythonInterfaceDescriptor, String)` — when the `python` feature is enabled.
-  `COUNTER` variable L33 — `: AtomicUsize` — when the `python` feature is enabled.
-  `copy_dir` function L46-58 — `(src: &std::path::Path, dst: &std::path::Path)` — when the `python` feature is enabled.
-  `make_python_package` function L60-109 — `( plugins_root: &std::path::Path, pkg_name: &str, entry_module: &str, ) -> PathB...` — when the `python` feature is enabled.
-  `repo_root` function L111-118 — `() -> PathBuf` — when the `python` feature is enabled.
-  `discover_surfaces_python_package` function L121-138 — `()` — when the `python` feature is enabled.
-  `load_python_dispatches_through_host` function L141-160 — `()` — when the `python` feature is enabled.
-  `load_python_unknown_name_returns_not_found` function L163-180 — `()` — when the `python` feature is enabled.
-  `cdylib_load_path_unaffected` function L183-201 — `()` — when the `python` feature is enabled.

#### crates/fidius-host/tests/python_streaming_e2e.rs

-  `ticker_descriptor` function L41-43 — `() -> &'static PythonInterfaceDescriptor` — The macro-generated descriptor for the `Ticker` interface — its
-  `stage` function L47-66 — `(tmp: &tempfile::TempDir) -> PathBuf` — Stage the py-ticker package into a fresh temp dir, vendor the in-tree SDK,
-  `repo_root` function L68-75 — `() -> PathBuf` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `copy_dir` function L77-89 — `(src: &std::path::Path, dst: &std::path::Path)` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `tick_index` function L91-93 — `() -> usize` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `discover_lists_streaming_python_plugin` function L96-107 — `()` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `server_stream_yields_all_items` function L110-130 — `()` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `huge_stream_is_bounded_and_cancellable` function L133-159 — `()` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.
-  `composition_pump_into_sink` function L162-186 — `()` — - the `fidius-test` composition harness (`pump`) wires the stream to a sink.

#### crates/fidius-host/tests/records_wasm.rs

- pub `Point` struct L38-41 — `{ x: i32, y: i32 }` — exercising the kebab↔snake/Pascal name normalization end to end.
- pub `Shape` enum L44-49 — `Circle | Rect | Triangle | Dot` — exercising the kebab↔snake/Pascal name normalization end to end.
- pub `Geo` interface L52-55 — `{ fn midpoint(), fn describe() }` — exercising the kebab↔snake/Pascal name normalization end to end.
-  `records_greeter_component` function L57-71 — `() -> &'static [u8]` — exercising the kebab↔snake/Pascal name normalization end to end.
-  `BYTES` variable L58 — `: OnceLock<Vec<u8>>` — exercising the kebab↔snake/Pascal name normalization end to end.
-  `stage_pkg` function L73-99 — `(root: &std::path::Path)` — exercising the kebab↔snake/Pascal name normalization end to end.
-  `record_in_record_out_round_trips` function L102-118 — `()` — exercising the kebab↔snake/Pascal name normalization end to end.
-  `variant_in_round_trips_all_cases` function L121-149 — `()` — exercising the kebab↔snake/Pascal name normalization end to end.

#### crates/fidius-host/tests/wasm_egress_e2e.rs

-  `IFACE` variable L38 — `: &str` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `fetcher_component` function L40-44 — `() -> Option<Vec<u8>>` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `mock_http_once` function L48-66 — `(body: &'static str) -> (String, std::thread::JoinHandle<()>)` — One-shot mock HTTP server on an ephemeral loopback port; serves a single
-  `AllowAll` struct L69 — `-` — Reference embedder policy: allow everything (the test's loopback grant).
-  `AllowAll` type L70-74 — `impl EgressPolicy for AllowAll` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `authorize` function L71-73 — `(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied>` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `DenyAll` struct L77 — `-` — Reference embedder policy: deny everything.
-  `DenyAll` type L78-82 — `impl EgressPolicy for DenyAll` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `authorize` function L79-81 — `(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied>` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `load` function L84-112 — `( caps: Vec<String>, egress: Option<Arc<dyn EgressPolicy>>, ) -> Result<PluginHa...` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `egress_allowed_fetches_body` function L115-125 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `egress_denied_by_policy` function L128-142 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `no_policy_fails_closed` function L145-157 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `no_capability_fails_closed` function L160-171 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `FETCHER_METHODS` variable L177-181 — `: [WasmMethodDesc; 1]` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `FETCHER` variable L182-187 — `: WasmInterfaceDescriptor` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `stage_fetcher_pkg` function L190-201 — `(root: &std::path::Path)` — Stage the fetcher as a loadable wasm package declaring the `http` capability.
-  `egress_via_builder_default_policy` function L204-222 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `egress_via_per_plugin_policy` function L225-244 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `load_wasm_without_egress_fails_closed` function L247-265 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).
-  `egress_via_builder_arc_dyn_policy` function L268-287 — `()` — embedder writes — fidius ships none of this (mechanism, not policy).

#### crates/fidius-host/tests/wasm_executor.rs

-  `IFACE` variable L32 — `: &str` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `EXPECTED_HASH` variable L33 — `: u64` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `greeter_component` function L36-50 — `() -> &'static [u8]` — Build the greeter component once (process-wide cache) and return its bytes.
-  `BYTES` variable L37 — `: OnceLock<Vec<u8>>` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `executor_with` function L52-92 — `(capabilities: Vec<String>) -> WasmComponentExecutor` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `executor` function L94-96 — `() -> WasmComponentExecutor` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `interface_hash_matches` function L99-101 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `typed_call_greet` function L104-111 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `typed_call_add_ok_and_err` function L114-129 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `raw_call_echo_bytes_reverses` function L132-136 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `method_count_and_info` function L139-143 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `METHOD_DESCS` variable L147-168 — `: [WasmMethodDesc; 4]` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `GREETER_DESC` variable L169-174 — `: WasmInterfaceDescriptor` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `stage_wasm_package` function L178-212 — `(root: &std::path::Path, capabilities: &[&str])` — Stage a `runtime = "wasm"` package directory containing the built component,
-  `load_wasm_through_host_and_call` function L215-234 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `load_wasm_rejects_interface_hash_mismatch` function L237-259 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `BAD_DESC` variable L238-243 — `: WasmInterfaceDescriptor` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `discover_surfaces_wasm_package` function L262-275 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `PROBE_ENV` variable L279 — `: usize` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `env_capability_denied_by_default` function L282-295 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `env_capability_granted_via_allowlist` function L298-313 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `bare_env_capability_rejected` function L316-333 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `scoped_env_does_not_leak_other_vars` function L336-352 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `python_greeter_component` function L360-364 — `() -> Option<Vec<u8>>` — The Python-authored component, if it's been built (see
-  `polyglot_python_guest_behaves_identically` function L370-419 — `()` — A Python guest implementing the SAME `greeter` WIT is loaded and called
-  `unknown_capability_rejected_at_load` function L422-437 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `set_precompiled` function L442-452 — `(pkg_dir: &std::path::Path, cwasm: &str)` — Record `precompiled = "<name>"` under `[wasm]` in a staged package.toml.
-  `precompiled_cwasm_loads_via_aot_and_calls` function L455-474 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `stale_cwasm_falls_back_to_jit` function L477-495 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `pack_unpack_load_roundtrip` function L498-521 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `sign_pkg` function L527-534 — `(pkg_dir: &std::path::Path) -> ed25519_dalek::VerifyingKey` — Sign a staged package dir over its `package_digest` (the same scheme
-  `signed_wasm_package_loads_when_signature_required` function L537-553 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `unsigned_wasm_package_rejected_when_signature_required` function L556-575 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `tampered_wasm_package_fails_verification` function L578-601 — `()` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `js_greeter_component` function L605-609 — `() -> Option<Vec<u8>>` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `polyglot_js_guest_behaves_identically` function L615-662 — `()` — A JavaScript guest (jco/ComponentizeJS) implementing the SAME `greeter` WIT
-  `go_greeter_component` function L666-670 — `() -> Option<Vec<u8>>` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `polyglot_go_guest_behaves_identically` function L676-722 — `()` — A Go guest (TinyGo + wit-bindgen-go) implementing the SAME `greeter` WIT loads
-  `c_greeter_component` function L726-730 — `() -> Option<Vec<u8>>` — `--features wasm` and requires the component toolchain (FIDIUS-T-0094).
-  `polyglot_c_guest_behaves_identically` function L736-782 — `()` — A C guest (wit-bindgen + wasi-sdk clang) implementing the SAME `greeter` WIT

#### crates/fidius-host/tests/wasm_streaming_e2e.rs

-  `IFACE` variable L33 — `: &str` — under the sandbox.
-  `HASH` variable L35 — `: u64` — under the sandbox.
-  `ticker_component` function L37-51 — `() -> &'static [u8]` — under the sandbox.
-  `BYTES` variable L38 — `: OnceLock<Vec<u8>>` — under the sandbox.
-  `handle` function L53-77 — `() -> PluginHandle` — under the sandbox.
-  `wasm_stream_yields_all_items` function L80-91 — `()` — under the sandbox.
-  `wasm_huge_stream_is_bounded_and_cancellable` function L94-107 — `()` — under the sandbox.
-  `wasm_empty_stream` function L110-117 — `()` — under the sandbox.
-  `wasm_composition_pump_into_sink` function L124-140 — `()` — under the sandbox.
-  `ticker_js_component` function L148-152 — `() -> Option<Vec<u8>>` — under the sandbox.
-  `js_handle` function L154-178 — `(bytes: &[u8]) -> PluginHandle` — under the sandbox.
-  `polyglot_js_guest_streams` function L181-200 — `()` — under the sandbox.
-  `polyglot_js_guest_bounded_and_cancellable` function L203-219 — `()` — under the sandbox.
-  `ticker_py_component` function L226-230 — `() -> Option<Vec<u8>>` — under the sandbox.
-  `py_wasm_handle` function L232-256 — `(bytes: &[u8]) -> PluginHandle` — under the sandbox.
-  `polyglot_py_wasm_guest_streams` function L259-277 — `()` — under the sandbox.
-  `polyglot_py_wasm_guest_bounded_and_cancellable` function L280-296 — `()` — under the sandbox.
-  `ticker_c_component` function L303-307 — `() -> Option<Vec<u8>>` — under the sandbox.
-  `c_wasm_handle` function L309-333 — `(bytes: &[u8]) -> PluginHandle` — under the sandbox.
-  `polyglot_c_wasm_guest_streams` function L336-354 — `()` — under the sandbox.
-  `polyglot_c_wasm_guest_bounded_and_cancellable` function L357-373 — `()` — under the sandbox.

### crates/fidius-macro/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-macro/src/impl_macro.rs

- pub `PluginImplAttrs` struct L106-115 — `{ trait_name: Ident, crate_path: Path, buffer_strategy: BufferStrategyAttr }` — Arguments to `#[plugin_impl(TraitName)]`, `#[plugin_impl(TraitName, crate = "...")]`,
- pub `generate_plugin_impl` function L170-308 — `(attrs: &PluginImplAttrs, item: &ItemImpl) -> syn::Result<TokenStream>` — Generate all code for a `#[plugin_impl(TraitName)]` invocation.
-  `MethodInfo` struct L31-51 — `{ name: &'a Ident, is_async: bool, returns_result: bool, arg_types: Vec<&'a Type...` — Info about an impl method, extracted from the impl block.
-  `impl_method_is_raw` function L56-73 — `(attrs: &[syn::Attribute]) -> syn::Result<bool>` — Detect a `#[wire(raw)]` attribute on an impl-side method.
-  `kebab_to_pascal` function L78-88 — `(s: &str) -> String` — kebab-case → PascalCase, for deriving the wit-bindgen resource type name from
-  `is_result_type` function L91-102 — `(ty: &Type) -> bool` — Check if a return type looks like `Result<T, ...>`.
-  `PluginImplAttrs` type L117-167 — `impl Parse for PluginImplAttrs` — dylibs, the FIDIUS_PLUGIN_REGISTRY.
-  `parse` function L118-166 — `(input: ParseStream) -> syn::Result<Self>` — dylibs, the FIDIUS_PLUGIN_REGISTRY.
-  `generate_wasm_adapter` function L318-586 — `( trait_name: &Ident, instance_name: &Ident, methods: &[MethodInfo], ) -> TokenS...` — Generate the WASM component auto-export adapter for `#[plugin_impl]`.
-  `collect_user_idents` function L590-635 — `(ty: &Type, out: &mut std::collections::BTreeSet<String>)` — Collect candidate user-type idents (non-primitive path leaves) from a type,
-  `gen_type` function L640-666 — `(ty: &Type, known: &std::collections::BTreeSet<String>, pkg_seg: &Ident) -> Toke...` — The wit-bindgen-generated type for an author type: identity for types holding
-  `wasm_first_generic` function L668-677 — `(seg: &syn::PathSegment) -> Option<&Type>` — dylibs, the FIDIUS_PLUGIN_REGISTRY.
-  `wasm_unsupported` function L683-693 — `(method: &Ident, reason: &str) -> TokenStream` — Emit a `#[cfg(target_family = "wasm")]`-gated `compile_error!` for a method
-  `generate_shims` function L697-1010 — `( impl_ident: &Ident, methods: &[MethodInfo], crate_path: &Path, buffer_strategy...` — Generate extern "C" shim functions for each method.
-  `generate_vtable_static` function L1016-1038 — `( trait_name: &Ident, impl_ident: &Ident, methods: &[&Ident], ) -> TokenStream` — Generate the static vtable with function pointers.
-  `generate_descriptor` function L1041-1143 — `( trait_name: &Ident, impl_ident: &Ident, methods: &[&Ident], crate_path: &Path,...` — Generate the PluginDescriptor static.
-  `generate_inventory_registration` function L1146-1157 — `(impl_ident: &Ident, crate_path: &Path) -> TokenStream` — Register the descriptor via inventory for multi-plugin support.

#### crates/fidius-macro/src/interface.rs

- pub `generate_interface` function L48-81 — `(ir: &InterfaceIR) -> syn::Result<TokenStream>` — Generate all code for a `#[plugin_interface]` invocation.
-  `strip_optional_attrs` function L29-45 — `(item: &ItemTrait) -> ItemTrait` — Strip fidius-specific helper attributes (`#[optional]`, `#[method_meta]`,
-  `is_fidius_helper` function L30-35 — `(attr: &syn::Attribute) -> bool` — capability bit constants, version/strategy constants, and a descriptor builder function.
-  `generate_metadata` function L92-190 — `(ir: &InterfaceIR) -> TokenStream` — Emit the static metadata arrays for `#[method_meta]` and `#[trait_meta]`
-  `generate_vtable` function L193-272 — `(ir: &InterfaceIR) -> TokenStream` — Generate the `#[repr(C)]` vtable struct.
-  `generate_constants` function L275-408 — `(ir: &InterfaceIR) -> TokenStream` — Generate interface hash, capability bit constants, version, and buffer strategy constants.
-  `generate_descriptor_builder` function L411-485 — `(ir: &InterfaceIR) -> TokenStream` — Generate the descriptor builder function used by `#[plugin_impl]`.
-  `generate_method_indices` function L488-504 — `(ir: &InterfaceIR) -> TokenStream` — Generate method index constants.
-  `generate_client` function L518-663 — `(ir: &InterfaceIR) -> TokenStream` — Generate a typed `{Trait}Client` struct that wraps a `PluginHandle` and

#### crates/fidius-macro/src/ir.rs

- pub `InterfaceAttrs` struct L30-36 — `{ version: u32, buffer_strategy: BufferStrategyAttr, crate_path: Path }` — Parsed attributes from `#[plugin_interface(version = N, buffer = Strategy)]`.
- pub `BufferStrategyAttr` enum L43-46 — `PluginAllocated | Arena` — Discriminants match `fidius_core::descriptor::BufferStrategyKind` — values
- pub `MetaKvAttr` struct L125-128 — `{ key: String, value: String }` — A static metadata key/value pair parsed from a `#[method_meta(...)]`
- pub `InterfaceIR` struct L132-140 — `{ trait_name: Ident, attrs: InterfaceAttrs, methods: Vec<MethodIR>, trait_metas:...` — Full IR for a parsed interface trait.
- pub `MethodIR` struct L145-180 — `{ name: Ident, arg_types: Vec<Type>, arg_names: Vec<Ident>, return_type: Option<...` — IR for a single trait method.
- pub `is_required` function L184-186 — `(&self) -> bool` — Whether this is a required (non-optional) method.
- pub `parse_interface` function L479-572 — `(attrs: InterfaceAttrs, item: &ItemTrait) -> syn::Result<InterfaceIR>` — Parse an `ItemTrait` into an `InterfaceIR`.
-  `InterfaceAttrs` type L48-120 — `impl Parse for InterfaceAttrs` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `parse` function L49-119 — `(input: ParseStream) -> syn::Result<Self>` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `MethodIR` type L182-187 — `= MethodIR` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `parse_meta_attrs` function L193-237 — `(attrs: &[Attribute], ident: &str) -> syn::Result<Vec<MetaKvAttr>>` — Parse all `#[method_meta("k", "v")]` or `#[trait_meta("k", "v")]`
-  `parse_optional_attr` function L240-258 — `(attrs: &[Attribute]) -> syn::Result<Option<u32>>` — Parse an `#[optional(since = N)]` attribute, if present.
-  `parse_wire_attr` function L263-280 — `(attrs: &[Attribute]) -> syn::Result<bool>` — Parse a `#[wire(raw)]` attribute, if present.
-  `is_vec_u8` function L283-310 — `(ty: &Type) -> bool` — Return `true` if the given type is `Vec<u8>`.
-  `result_ok_type` function L313-329 — `(ty: &Type) -> Option<&Type>` — Extract the first type parameter of `Result<_, _>`, if `ty` is a Result.
-  `validate_raw_method_signature` function L334-371 — `( method: &TraitItemFn, arg_types: &[Type], return_type: Option<&Type>, ) -> syn...` — Validate that a method flagged `#[wire(raw)]` has a supported signature:
-  `stream_item_type` function L378-396 — `(ty: &Type) -> Option<Type>` — Return the per-item type `T` if `ty` is a `Stream<T>` (i.e.
-  `build_signature_string` function L408-435 — `( method: &TraitItemFn, wire_raw: bool, stream_item: Option<&Type>, ) -> String` — Build the canonical signature string for a method.
-  `extract_arg_names` function L438-455 — `(method: &TraitItemFn) -> Vec<Ident>` — Extract argument names from a method signature (excluding `self`).
-  `extract_arg_types` function L458-468 — `(method: &TraitItemFn) -> Vec<Type>` — Extract argument types from a method signature (excluding `self`).
-  `extract_return_type` function L471-476 — `(method: &TraitItemFn) -> Option<Type>` — Extract the return type (unwrapped from `-> Type`).
-  `tests` module L575-763 — `-` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `parse_test_trait` function L579-587 — `(tokens: proc_macro2::TokenStream) -> InterfaceIR` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `basic_trait_parsing` function L590-607 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `optional_method_parsing` function L610-623 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `async_method_detection` function L626-636 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `rejects_mut_self` function L639-655 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `signature_string_format` function L658-668 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `interface_attrs_parsing` function L671-677 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `interface_attrs_with_crate_path` function L680-693 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `detects_server_streaming_return` function L696-715 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `streaming_and_unary_hash_differently` function L718-732 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `bare_stream_marker_is_detected` function L735-742 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.
-  `rejects_stream_in_argument_position` function L745-762 — `()` — Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.

#### crates/fidius-macro/src/lib.rs

- pub `plugin_interface` function L43-54 — `(attr: TokenStream, item: TokenStream) -> TokenStream` — Define a plugin interface from a trait.
- pub `plugin_impl` function L74-82 — `(attr: TokenStream, item: TokenStream) -> TokenStream` — Implement a plugin interface for a concrete type.
- pub `derive_wit_type` function L98-102 — `(_item: TokenStream) -> TokenStream` — Mark a `struct`/`enum` as usable in a WASM plugin interface (FIDIUS-I-0023).
-  `impl_macro` module L15 — `-`
-  `interface` module L16 — `-`
-  `ir` module L17 — `-`
-  `wit` module L18 — `-`

#### crates/fidius-macro/src/wit.rs

-  `to_kebab_case` function L32-47 — `(s: &str) -> String` — Convert a Rust identifier (CamelCase or snake_case) to kebab-case, the WIT
-  `result_ok_type` function L50-59 — `(ty: &Type) -> Option<&Type>` — Extract the `T` from `Result<T, _>`, if `ty` is a `Result`.
-  `WitMethod` struct L62-69 — `{ name: String, params: Vec<(String, String)>, ret: Option<String> }` — One method projected to WIT (already-mapped strings).
-  `render_wit` function L75-102 — `(iface_kebab: &str, methods: &[WitMethod]) -> String` — Render a complete `.wit` document for an interface and its methods.
-  `rust_type_to_wit` function L106-169 — `(ty: &Type) -> Result<String, String>` — Map a Rust argument/return type to its WIT spelling.
-  `return_to_wit` function L174-193 — `(ret: Option<&Type>) -> Result<Option<String>, String>` — Map a method's return type to an optional WIT return.
-  `is_unit` function L195-197 — `(ty: &Type) -> bool` — a clear compile error rather than silently-wrong WIT.
-  `path_is` function L199-205 — `(p: &syn::TypePath, name: &str) -> bool` — a clear compile error rather than silently-wrong WIT.
-  `single_generic` function L207-209 — `(seg: &'a syn::PathSegment, what: &str) -> Result<&'a Type, String>` — a clear compile error rather than silently-wrong WIT.
-  `first_generic` function L211-220 — `(seg: &syn::PathSegment) -> Option<&Type>` — a clear compile error rather than silently-wrong WIT.
-  `tests` module L223-296 — `-` — a clear compile error rather than silently-wrong WIT.
-  `wit` function L226-228 — `(s: &str) -> String` — a clear compile error rather than silently-wrong WIT.
-  `ret` function L229-231 — `(s: &str) -> Option<String>` — a clear compile error rather than silently-wrong WIT.
-  `primitives_and_strings` function L234-242 — `()` — a clear compile error rather than silently-wrong WIT.
-  `containers` function L245-251 — `()` — a clear compile error rather than silently-wrong WIT.
-  `returns` function L254-266 — `()` — a clear compile error rather than silently-wrong WIT.
-  `unsupported_is_error` function L269-271 — `()` — a clear compile error rather than silently-wrong WIT.
-  `renders_greeter_like_wit` function L274-295 — `()` — a clear compile error rather than silently-wrong WIT.

### crates/fidius-macro/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-macro/tests/arena_basic.rs

- pub `EchoArena` interface L26-28 — `{ fn echo() }` — buffer as the arena.
- pub `MyEcho` struct L30 — `-` — buffer as the arena.
-  `MyEcho` type L33-37 — `impl EchoArena for MyEcho` — buffer as the arena.
-  `echo` function L34-36 — `(&self, input: String) -> String` — buffer as the arena.
-  `arena_shim_round_trip_with_sufficient_buffer` function L42-85 — `()` — buffer as the arena.
-  `arena_shim_returns_buffer_too_small` function L88-119 — `()` — buffer as the arena.

#### crates/fidius-macro/tests/async_plugin.rs

- pub `AsyncProcessor` interface L21-23 — `{ fn process() }` — Test that async methods work with the fidius macros.
- pub `MyProcessor` struct L25 — `-` — Test that async methods work with the fidius macros.
-  `MyProcessor` type L28-33 — `impl AsyncProcessor for MyProcessor` — Test that async methods work with the fidius macros.
-  `process` function L29-32 — `(&self, input: String) -> String` — Test that async methods work with the fidius macros.
-  `can_call_async_method_via_vtable` function L38-71 — `()` — Test that async methods work with the fidius macros.

#### crates/fidius-macro/tests/crate_path.rs

- pub `Calculator` interface L23-25 — `{ fn add() }` — to verify custom crate path resolution.
- pub `MyCalculator` struct L27 — `-` — to verify custom crate path resolution.
-  `MyCalculator` type L30-34 — `impl Calculator for MyCalculator` — to verify custom crate path resolution.
-  `add` function L31-33 — `(&self, input: String) -> String` — to verify custom crate path resolution.
-  `custom_crate_path_compiles_and_works` function L39-43 — `()` — to verify custom crate path resolution.
-  `custom_crate_path_shim_callable` function L46-78 — `()` — to verify custom crate path resolution.

#### crates/fidius-macro/tests/impl_basic.rs

- pub `Greeter` interface L21-23 — `{ fn greet() }` — Test that #[plugin_impl] compiles and generates expected items.
- pub `HelloGreeter` struct L25 — `-` — Test that #[plugin_impl] compiles and generates expected items.
-  `HelloGreeter` type L28-32 — `impl Greeter for HelloGreeter` — Test that #[plugin_impl] compiles and generates expected items.
-  `greet` function L29-31 — `(&self, name: String) -> String` — Test that #[plugin_impl] compiles and generates expected items.
-  `get_registry` function L37-39 — `() -> &'static fidius_core::descriptor::PluginRegistry` — Test that #[plugin_impl] compiles and generates expected items.
-  `registry_exists_and_is_valid` function L42-47 — `()` — Test that #[plugin_impl] compiles and generates expected items.
-  `descriptor_fields_are_correct` function L50-61 — `()` — Test that #[plugin_impl] compiles and generates expected items.
-  `can_call_shim_via_vtable` function L64-100 — `()` — Test that #[plugin_impl] compiles and generates expected items.

#### crates/fidius-macro/tests/interface_basic.rs

- pub `Greeter` interface L21-26 — `{ fn greet(), fn greet_fancy() }` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `vtable_struct_exists` function L29-34 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `interface_hash_is_nonzero` function L37-39 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `interface_version_matches` function L42-44 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `buffer_strategy_matches` function L47-49 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `capability_constant_exists` function L52-55 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.
-  `method_index_constants_exist` function L58-61 — `()` — Basic test that #[plugin_interface] compiles and generates expected items.

#### crates/fidius-macro/tests/metadata.rs

- pub `Tagged` interface L27-37 — `{ fn create(), fn list(), fn version() }` — into the PluginDescriptor at the plugin-link level (not dylib).
- pub `MyTagged` struct L39 — `-` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `MyTagged` type L42-52 — `impl Tagged for MyTagged` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `create` function L43-45 — `(&self, name: String) -> String` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `list` function L46-48 — `(&self) -> String` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `version` function L49-51 — `(&self) -> String` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `read_cstr` function L56-60 — `(ptr: *const std::ffi::c_char) -> &'static str` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `trait_metadata_is_populated` function L63-76 — `()` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `method_metadata_is_populated_per_method` function L79-106 — `()` — into the PluginDescriptor at the plugin-link level (not dylib).
-  `interface_hash_unaffected_by_metadata` function L109-116 — `()` — into the PluginDescriptor at the plugin-link level (not dylib).

#### crates/fidius-macro/tests/multi_arg.rs

- pub `MultiArg` interface L23-35 — `{ fn status(), fn echo(), fn concat(), fn add_three() }` — with uniform tuple encoding.
- pub `MyMultiArg` struct L37 — `-` — with uniform tuple encoding.
-  `MyMultiArg` type L40-56 — `impl MultiArg for MyMultiArg` — with uniform tuple encoding.
-  `status` function L41-43 — `(&self) -> String` — with uniform tuple encoding.
-  `echo` function L45-47 — `(&self, msg: String) -> String` — with uniform tuple encoding.
-  `concat` function L49-51 — `(&self, a: String, b: String) -> String` — with uniform tuple encoding.
-  `add_three` function L53-55 — `(&self, x: i64, y: i64, z: i64) -> i64` — with uniform tuple encoding.
-  `get_registry` function L60-62 — `() -> &'static fidius_core::descriptor::PluginRegistry` — with uniform tuple encoding.
-  `call_vtable` function L65-107 — `( vtable: &__fidius_MultiArg::MultiArg_VTable, desc: &fidius_core::descriptor::P...` — Helper: call a vtable method by index with given input bytes.
-  `zero_args_status` function L110-122 — `()` — with uniform tuple encoding.
-  `one_arg_echo` function L125-137 — `()` — with uniform tuple encoding.
-  `two_args_concat` function L140-153 — `()` — with uniform tuple encoding.
-  `three_args_add` function L156-168 — `()` — with uniform tuple encoding.
-  `method_indices_correct` function L171-176 — `()` — with uniform tuple encoding.

#### crates/fidius-macro/tests/multi_plugin.rs

- pub `Greeter` interface L21-23 — `{ fn greet() }` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
- pub `HelloGreeter` struct L26 — `-` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
- pub `GoodbyeGreeter` struct L36 — `-` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `HelloGreeter` type L29-33 — `impl Greeter for HelloGreeter` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `greet` function L30-32 — `(&self, name: String) -> String` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `GoodbyeGreeter` type L39-43 — `impl Greeter for GoodbyeGreeter` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `greet` function L40-42 — `(&self, name: String) -> String` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `registry_has_two_plugins` function L49-54 — `()` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `both_descriptors_are_valid` function L57-80 — `()` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
-  `can_call_both_plugins` function L83-123 — `()` — Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.

#### crates/fidius-macro/tests/raw_wire.rs

- pub `TypedPipe` interface L32-34 — `{ fn process() }` — without needing to load a dylib.
- pub `RawPipe` interface L37-40 — `{ fn process() }` — without needing to load a dylib.
- pub `Mixed` interface L56-65 — `{ fn bulk(), fn ping(), fn bulk_v2() }` — without needing to load a dylib.
- pub `FallibleBytePipe` interface L84-87 — `{ fn maybe() }` — without needing to load a dylib.
-  `raw_marker_changes_interface_hash` function L43-50 — `()` — without needing to load a dylib.
-  `mixed_interface_companion_module_compiles` function L68-77 — `()` — without needing to load a dylib.
-  `raw_method_with_result_return_compiles` function L90-93 — `()` — without needing to load a dylib.

#### crates/fidius-macro/tests/smoke_cdylib.rs

-  `load_cdylib_and_call_plugin` function L23-136 — `()` — loads it via dlopen/dlsym and verifies the registry and vtable work.
-  `AddInput` struct L96-99 — `{ a: i64, b: i64 }` — loads it via dlopen/dlsym and verifies the registry and vtable work.
-  `AddOutput` struct L101-103 — `{ result: i64 }` — loads it via dlopen/dlsym and verifies the registry and vtable work.

#### crates/fidius-macro/tests/trybuild.rs

-  `compile_fail_tests` function L16-19 — `()`

### crates/fidius-macro/tests/compile_fail

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-macro/tests/compile_fail/caller_allocated_removed.rs

- pub `BadPlugin` interface L11-13 — `{ fn do_thing() }`
-  `main` function L15 — `()`

#### crates/fidius-macro/tests/compile_fail/duplicate_method_meta_key.rs

- pub `BadPlugin` interface L7-11 — `{ fn do_thing() }`
-  `main` function L13 — `()`

#### crates/fidius-macro/tests/compile_fail/missing_version.rs

- pub `BadPlugin` interface L18-20 — `{ fn do_thing() }`
-  `main` function L22 — `()`

#### crates/fidius-macro/tests/compile_fail/mut_self.rs

- pub `BadPlugin` interface L18-20 — `{ fn mutate() }`
-  `main` function L22 — `()`

#### crates/fidius-macro/tests/compile_fail/reserved_fidius_namespace.rs

- pub `BadPlugin` interface L7-10 — `{ fn do_thing() }`
-  `main` function L12 — `()`

#### crates/fidius-macro/tests/compile_fail/stream_in_arg_position.rs

- pub `BadStream` interface L22-24 — `{ fn sink() }`
-  `main` function L26 — `()`

### crates/fidius-python

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-python/build.rs

-  `main` function L26-47 — `()` — Build script: configure PyO3 cfg flags and emit a runtime rpath so the

### crates/fidius-python/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-python/src/error.rs

- pub `pyerr_to_plugin_error` function L40-69 — `(err: PyErr) -> PluginError` — Convert a `PyErr` into a `PluginError`, preserving class name, message,
-  `format_traceback` function L74-79 — `(py: Python<'_>, tb: Bound<'_, PyTraceback>) -> PyResult<String>` — Format a Python traceback into a plain string by calling
-  `tests` module L82-105 — `-` — plugin code can raise typed errors without their fields being flattened.
-  `maps_value_error_to_plugin_error` function L86-104 — `()` — plugin code can raise typed errors without their fields being flattened.

#### crates/fidius-python/src/handle.rs

- pub `PythonCallError` enum L44-70 — `InvalidMethodIndex | WireModeMismatch | InputDecode | OutputEncode | Plugin` — Errors a typed call can produce on the Python side.
- pub `PythonPluginHandle` struct L74-82 — `{ descriptor: &'static PythonInterfaceDescriptor, _module: Py<PyAny>, method_cal...` — Loaded-and-validated handle to one Python plugin.
- pub `descriptor` function L97-99 — `(&self) -> &'static PythonInterfaceDescriptor` — `code = <ExceptionClassName>` otherwise.
- pub `method_count` function L101-103 — `(&self) -> usize` — `code = <ExceptionClassName>` otherwise.
- pub `call_typed` function L112-134 — `( &self, method_index: usize, input_bincode: &[u8], ) -> Result<Vec<u8>, PythonC...` — Typed dispatch.
- pub `call_typed_json` function L139-159 — `( &self, method_index: usize, input_json: &[u8], ) -> Result<Vec<u8>, PythonCall...` — Typed dispatch where the input is already JSON-serialised (the
- pub `call_streaming_start` function L165-190 — `( &self, method_index: usize, input_json: &[u8], ) -> Result<crate::stream::Pyth...` — Start a server-streaming call (FIDIUS-I-0026).
- pub `call_raw` function L193-212 — `(&self, method_index: usize, input: &[u8]) -> Result<Vec<u8>, PythonCallError>` — Raw dispatch — pass bytes in, get bytes out, no encoding.
-  `PythonPluginHandle` type L84-237 — `= PythonPluginHandle` — `code = <ExceptionClassName>` otherwise.
-  `new` function L85-95 — `( descriptor: &'static PythonInterfaceDescriptor, module: Py<PyAny>, method_call...` — `code = <ExceptionClassName>` otherwise.
-  `lookup_method` function L214-236 — `( &self, index: usize, attempting_raw: bool, ) -> Result<MethodLookup<'_>, Pytho...` — `code = <ExceptionClassName>` otherwise.
-  `MethodLookup` struct L239-241 — `{ callable: &'a Py<PyAny> }` — `code = <ExceptionClassName>` otherwise.
-  `build_call_args` function L250-269 — `( py: Python<'py>, input: &serde_json::Value, ) -> PyResult<Bound<'py, PyTuple>>` — Build positional args for `callable.call(...)` from a JSON value.

#### crates/fidius-python/src/interpreter.rs

- pub `ensure_initialized` function L38-46 — `()` — Idempotent: ensure the embedded Python interpreter is initialised.
-  `INIT` variable L30 — `: Once` — separate `Mutex<PyInterpreter>` to manage.

#### crates/fidius-python/src/lib.rs

- pub `error` module L27 — `-` — Python plugin runtime for Fidius.
- pub `handle` module L28 — `-` — under FIDIUS-I-0020.
- pub `interpreter` module L29 — `-` — under FIDIUS-I-0020.
- pub `loader` module L30 — `-` — under FIDIUS-I-0020.
- pub `stream` module L31 — `-` — under FIDIUS-I-0020.
- pub `value_bridge` module L32 — `-` — under FIDIUS-I-0020.

#### crates/fidius-python/src/loader.rs

- pub `PythonLoadError` enum L47-82 — `Manifest | NotPythonRuntime | MissingPythonSection | ImportFailed | InterfaceHas...` — Errors that can happen during Python plugin load.
- pub `load_python_plugin` function L89-139 — `( package_dir: &Path, descriptor: &'static PythonInterfaceDescriptor, ) -> Resul...` — Load a Python plugin package against a static interface descriptor.
-  `prepend_sys_path` function L144-173 — `(py: Python<'_>, dir: &Path) -> Result<(), PythonLoadError>` — Prepend `<dir>/vendor` and `<dir>` to `sys.path` if not already present.
-  `validate_interface_hash` function L175-197 — `( module: &Bound<'_, PyModule>, descriptor: &'static PythonInterfaceDescriptor, ...` — All Python work happens in the host's embedded interpreter (T-0085).
-  `resolve_methods` function L199-227 — `( module: &Bound<'_, PyModule>, descriptor: &'static PythonInterfaceDescriptor, ...` — All Python work happens in the host's embedded interpreter (T-0085).
-  `import_failure` function L229-235 — `(what: &str, err: PyErr) -> PythonLoadError` — All Python work happens in the host's embedded interpreter (T-0085).

#### crates/fidius-python/src/stream.rs

- pub `PyStreamStep` enum L39-46 — `Item | End | Error` — One step of advancing a Python plugin's server-streaming iterator.
- pub `PythonStream` struct L53-55 — `{ iter: Py<PyAny> }` — A handle to an in-flight Python server-stream — the iterator obtained by
- pub `next` function L64-78 — `(&self) -> PyStreamStep` — Advance one item.
- pub `cancel` function L84-91 — `(&self)` — Cancel the stream: run the generator's cleanup by calling `close()`,
-  `PythonStream` type L57-92 — `= PythonStream` — split (`fidius-python` has no async runtime).
-  `new` function L58-60 — `(iter: Py<PyAny>) -> Self` — split (`fidius-python` has no async runtime).
-  `tests` module L95-201 — `-` — split (`fidius-python` has no async runtime).
-  `stream_from` function L100-109 — `(code: &str) -> PythonStream` — Build a `PythonStream` from a snippet that evaluates to an iterator.
-  `item_i64` function L111-116 — `(step: PyStreamStep) -> i64` — split (`fidius-python` has no async runtime).
-  `step_name` function L118-124 — `(s: &PyStreamStep) -> &'static str` — split (`fidius-python` has no async runtime).
-  `yields_items_then_end` function L127-135 — `()` — split (`fidius-python` has no async runtime).
-  `generator_exception_becomes_error` function L138-150 — `()` — split (`fidius-python` has no async runtime).
-  `gen_from_def` function L153-162 — `(code: &str) -> PythonStream` — Run a snippet that binds `it` to an iterator/generator in fresh globals.
-  `cancel_runs_generator_finally` function L165-200 — `()` — split (`fidius-python` has no async runtime).

#### crates/fidius-python/src/value_bridge.rs

- pub `value_to_pyobject` function L33-65 — `(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>>` — Convert a `serde_json::Value` into a Python object owned by `py`.
- pub `pyobject_to_value` function L72-141 — `(obj: &Bound<'_, PyAny>) -> PyResult<Value>` — Convert a Python object back into a `serde_json::Value`.
-  `tests` module L144-167 — `-` — which bypasses this layer entirely.
-  `roundtrip_primitives` function L149-166 — `()` — which bypasses this layer entirely.

### crates/fidius-python/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-python/tests/loader_e2e.rs

-  `HASH` variable L29 — `: u64` — independently.
-  `GREETER_METHODS` variable L30-43 — `: [PythonMethodDesc; 3]` — independently.
-  `ERROR_METHODS` variable L45-48 — `: [PythonMethodDesc; 1]` — independently.
-  `fresh_descriptor` function L55-68 — `( methods: &'static [PythonMethodDesc], ) -> (&'static PythonInterfaceDescriptor...` — Make a `'static` interface descriptor with a unique name so each test
-  `COUNTER` variable L58 — `: AtomicUsize` — independently.
-  `make_plugin` function L75-127 — `( tmp: &tempfile::TempDir, entry_module: &str, declared_hash: u64, methods_sourc...` — Stand up a minimal Python plugin package on disk:
-  `GREETER_METHODS_SRC` variable L129-141 — `: &str` — independently.
-  `ERROR_METHODS_SRC` variable L143-147 — `: &str` — independently.
-  `repo_root` function L149-156 — `() -> PathBuf` — independently.
-  `copy_dir` function L158-170 — `(src: &std::path::Path, dst: &std::path::Path)` — independently.
-  `load_greeter` function L172-178 — `() -> (tempfile::TempDir, fidius_python::PythonPluginHandle)` — independently.
-  `typed_call_round_trip_string` function L181-187 — `()` — independently.
-  `typed_call_with_struct_args` function L190-218 — `()` — independently.
-  `DoubleIn` struct L194-197 — `{ name: String, count: i64 }` — independently.
-  `DoubleOut` struct L199-202 — `{ name: String, twice: i64 }` — independently.
-  `raw_call_round_trip_2mb` function L221-231 — `()` — independently.
-  `plugin_error_round_trips_with_code_and_details` function L234-260 — `()` — independently.
-  `interface_hash_mismatch_is_rejected` function L263-273 — `()` — independently.
-  `wire_mode_mismatch_typed_called_as_raw_errors` function L276-281 — `()` — independently.
-  `out_of_range_method_index_errors` function L284-288 — `()` — independently.

#### crates/fidius-python/tests/smoke.rs

-  `interpreter_evaluates_simple_expression` function L25-35 — `()` — Python exception.
-  `pyerr_to_plugin_error_preserves_class_message_and_traceback` function L38-50 — `()` — Python exception.

### crates/fidius-test/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-test/src/dylib.rs

- pub `dylib_fixture` function L52-58 — `(plugin_dir: impl Into<PathBuf>) -> DylibFixtureBuilder` — Start building a dylib fixture for the plugin crate at `plugin_dir`.
- pub `DylibFixtureBuilder` struct L61-65 — `{ plugin_dir: PathBuf, release: bool, signing_key: Option<SigningKey> }` — Builder for [`DylibFixture`].
- pub `with_release` function L70-73 — `(mut self, release: bool) -> Self` — Build in release mode.
- pub `signed_with` function L81-84 — `(mut self, key: &SigningKey) -> Self` — Sign the produced dylib with `key`, writing a `.sig` file alongside it.
- pub `build` function L90-116 — `(self) -> DylibFixture` — Execute the build (or return cached result) and produce the fixture.
- pub `DylibFixture` struct L121-127 — `{ plugin_output_dir: PathBuf, dylib_path: PathBuf }` — A built plugin ready to be loaded by a `PluginHost`.
- pub `dir` function L131-133 — `(&self) -> &Path` — Directory containing the built dylib — `search_path` for `PluginHost`.
- pub `dylib_path` function L137-139 — `(&self) -> &Path` — Full path to the dylib file itself.
-  `DylibFixtureBuilder` type L67-117 — `= DylibFixtureBuilder` — ```
-  `DylibFixture` type L129-140 — `= DylibFixture` — ```
-  `CacheKey` struct L145-148 — `{ plugin_dir: PathBuf, release: bool }` — ```
-  `cache` function L150-153 — `() -> &'static Mutex<HashMap<CacheKey, DylibFixture>>` — ```
-  `CACHE` variable L151 — `: OnceLock<Mutex<HashMap<CacheKey, DylibFixture>>>` — ```
-  `dylib_extension` function L155-163 — `() -> &'static str` — ```
-  `build_uncached` function L165-205 — `(plugin_dir: &Path, release: bool) -> DylibFixture` — ```

#### crates/fidius-test/src/lib.rs

- pub `dylib` module L45 — `-` — Testing helpers for Fidius plugin authors and hosts.
- pub `signing` module L46 — `-` — ```
- pub `stream` module L48 — `-` — ```

#### crates/fidius-test/src/signing.rs

- pub `fixture_keypair_with_seed` function L30-34 — `(seed: u8) -> (SigningKey, VerifyingKey)` — Deterministic Ed25519 keypair derived from `seed` repeated 32 times.
- pub `fixture_keypair` function L37-39 — `() -> (SigningKey, VerifyingKey)` — Convenience: [`fixture_keypair_with_seed(1)`](fixture_keypair_with_seed).
- pub `sign_dylib` function L45-54 — `(dylib: &Path, key: &SigningKey) -> std::io::Result<()>` — Sign a plugin dylib in place by writing a detached `.sig` file alongside it.

#### crates/fidius-test/src/stream.rs

- pub `StreamSink` interface L42-45 — `{ fn accept() }` — The destination side of a pipe: a consumer `pump` hands each item to, in
- pub `stream_of` function L52-56 — `(items: Vec<Value>) -> ChunkStream` — An in-memory source over a fixed item sequence.
- pub `collect` function L60-66 — `(mut s: ChunkStream) -> Result<Vec<Value>, CallError>` — Drain a stream to a `Vec`, stopping at — and returning — the first error.
- pub `pump` function L76-84 — `(mut out: ChunkStream, into: &S) -> Result<(), CallError>` — The reference pull-loop wiring a producer stream to a [`StreamSink`].
- pub `CollectSink` struct L89-91 — `{ items: Mutex<Vec<Value>> }` — A [`StreamSink`] that records everything it accepts — for asserting on the
- pub `new` function L95-97 — `() -> Self` — A fresh, empty sink.
- pub `take` function L100-102 — `(&self) -> Vec<Value>` — Snapshot of everything accepted so far.
-  `CollectSink` type L93-103 — `= CollectSink` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `CollectSink` type L106-111 — `impl StreamSink for CollectSink` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `accept` function L107-110 — `(&self, item: Value) -> Result<(), CallError>` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `tests` module L114-175 — `-` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `vals` function L118-120 — `(xs: &[i64]) -> Vec<Value>` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `ints` function L122-124 — `(vs: Vec<Value>) -> Vec<i64>` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `stream_of_then_collect_round_trips` function L127-130 — `()` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `collect_surfaces_first_error` function L133-141 — `()` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `pump_delivers_all_items_to_sink` function L144-148 — `()` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `pump_stops_on_producer_error` function L151-161 — `()` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").
-  `compose_single_plugin_idiom` function L164-174 — `()` — See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").

### crates/fidius-test/tests

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-test/tests/smoke.rs

-  `plugin_source_dir` function L25-27 — `() -> PathBuf` — fixture.
-  `fixture_keypair_is_deterministic` function L30-35 — `()` — fixture.
-  `fixture_keypair_with_seed_differs_by_seed` function L38-42 — `()` — fixture.
-  `sign_dylib_produces_verifiable_signature` function L45-67 — `()` — fixture.
-  `dylib_fixture_builds_plugin_and_host_can_discover` function L70-94 — `()` — fixture.
-  `dylib_fixture_is_cached_across_builds` function L97-106 — `()` — fixture.
-  `client_in_process_calls_plugin_without_dylib_load` function L109-120 — `()` — fixture.
-  `client_in_process_returns_not_found_for_missing_plugin` function L123-130 — `()` — fixture.

### crates/fidius-wit/src

> *Semantic summary to be generated by AI agent.*

#### crates/fidius-wit/src/generate.rs

- pub `Generated` struct L33-45 — `{ interface_name: String, iface_kebab: String, user_types: Vec<String>, wit: Str...` — The product of generating from a plugin crate's source.
- pub `generate` function L50-55 — `(src: &str) -> Result<Generated, String>` — Generate WIT + conversions from a crate's source string (`lib.rs`).
- pub `generate_from_path` function L60-68 — `(lib_rs: &std::path::Path) -> Result<Generated, String>` — Like [`generate`], but reads `lib_rs` and follows external `mod m;` files
- pub `conv_expr` function L352-379 — `(access: &str, ty: &Type, known: &BTreeSet<String>) -> String` — Conversion expression for a field/payload `access` of type `ty`.
- pub `contains_user_type` function L383-398 — `(ty: &Type, known: &BTreeSet<String>) -> bool` — Whether `ty` is, or contains (through `Vec`/`Option`/`Box`), a user type in
-  `Collected` struct L73-77 — `{ structs: Vec<(Vec<String>, syn::ItemStruct)>, enums: Vec<(Vec<String>, syn::It...` — `#[derive(WitType)]` types (tagged with their Rust module path) + the
-  `collect` function L81-127 — `( items: &[Item], mod_path: &[String], dir: Option<&std::path::Path>, acc: &mut ...` — Recursively gather items, descending into inline `mod m { ..
-  `assemble` function L130-207 — `(acc: Collected) -> Result<Generated, String>` — Build the `.wit` + conversions from the collected items.
-  `author_path` function L210-216 — `(mod_path: &[String], name: &str) -> String` — `crate::<mod::path>::<Name>` — the author-side path for a type at `mod_path`.
-  `render_conversions` function L222-344 — `( iface_kebab: &str, structs: &[(Vec<String>, syn::ItemStruct)], enums: &[(Vec<S...` — Render `From` impls (both directions) between each user type and its
-  `single_generic` function L400-409 — `(seg: &syn::PathSegment) -> Option<&Type>` — the `fidius wit` CLI, which read the source files.
-  `has_attr` function L412-420 — `(attrs: &[syn::Attribute], name: &str) -> bool` — Does `attrs` contain `#[<name>(...)]` / `#[<path>::<name>]` (last segment match)?
-  `has_derive` function L423-445 — `(attrs: &[syn::Attribute], name: &str) -> bool` — Does `attrs` contain a `#[derive(...
-  `tests` module L448-521 — `-` — the `fidius wit` CLI, which read the source files.
-  `SRC` variable L451-464 — `: &str` — the `fidius wit` CLI, which read the source files.
-  `generates_wit_with_records_variants_and_funcs` function L467-483 — `()` — the `fidius wit` CLI, which read the source files.
-  `generates_conversions_both_ways` function L486-499 — `()` — the `fidius wit` CLI, which read the source files.
-  `primitive_only_interface_has_no_conversions` function L502-511 — `()` — the `fidius wit` CLI, which read the source files.
-  `unsupported_type_errors` function L514-520 — `()` — the `fidius wit` CLI, which read the source files.

#### crates/fidius-wit/src/lib.rs

- pub `to_kebab_case` function L35-50 — `(s: &str) -> String` — Convert a Rust identifier (CamelCase or snake_case) to kebab-case, the WIT
- pub `result_ok_type` function L53-62 — `(ty: &Type) -> Option<&Type>` — Extract the `T` from `Result<T, _>`, if `ty` is a `Result`.
- pub `WitMethod` struct L65-78 — `{ name: String, params: Vec<(String, String)>, ret: Option<String>, stream_item:...` — One method projected to WIT (already-mapped strings).
- pub `stream_item_type` function L83-92 — `(ty: &Type) -> Option<&Type>` — If `ty` is `fidius::Stream<T>` (final path segment `Stream`, exactly one type
- pub `wit_type_with` function L97-155 — `(ty: &Type, known: &BTreeSet<String>) -> Result<String, String>` — Map a Rust argument/return type to its WIT spelling, where `known` holds the
- pub `rust_type_to_wit` function L159-161 — `(ty: &Type) -> Result<String, String>` — Primitive/std-only mapping (no user types) — the form `fidius-macro` uses for
- pub `return_to_wit_with` function L166-188 — `( ret: Option<&Type>, known: &BTreeSet<String>, ) -> Result<Option<String>, Stri...` — Map a method's return type to an optional WIT return, with user types in
- pub `return_to_wit` function L191-193 — `(ret: Option<&Type>) -> Result<Option<String>, String>` — Primitive/std-only return mapping (no user types).
- pub `struct_to_record` function L198-215 — `(item: &ItemStruct, known: &BTreeSet<String>) -> Result<String, String>` — Render a `record <name> { ...
- pub `enum_to_wit` function L225-268 — `( item: &ItemEnum, known: &BTreeSet<String>, ) -> Result<(Vec<String>, String), ...` — Render a Rust enum to WIT: a `variant <name> { ...
- pub `render_wit_full` function L274-323 — `(iface_kebab: &str, type_defs: &[String], methods: &[WitMethod]) -> String` — Render a complete `.wit` document: package + interface (the `plugin-error`
- pub `render_wit` function L327-329 — `(iface_kebab: &str, methods: &[WitMethod]) -> String` — Convenience: render a WIT document with no user type defs (the primitives-only
-  `generate` module L30 — `-` — helper, and the `fidius wit` CLI can all share one implementation.
-  `is_unit` function L333-335 — `(ty: &Type) -> bool` — helper, and the `fidius wit` CLI can all share one implementation.
-  `path_is` function L337-343 — `(p: &syn::TypePath, name: &str) -> bool` — helper, and the `fidius wit` CLI can all share one implementation.
-  `single_generic` function L345-347 — `(seg: &'a syn::PathSegment, what: &str) -> Result<&'a Type, String>` — helper, and the `fidius wit` CLI can all share one implementation.
-  `first_generic` function L349-358 — `(seg: &syn::PathSegment) -> Option<&Type>` — helper, and the `fidius wit` CLI can all share one implementation.
-  `tests` module L361-513 — `-` — helper, and the `fidius wit` CLI can all share one implementation.
-  `known` function L364-366 — `(names: &[&str]) -> BTreeSet<String>` — helper, and the `fidius wit` CLI can all share one implementation.
-  `wit` function L367-369 — `(s: &str) -> String` — helper, and the `fidius wit` CLI can all share one implementation.
-  `primitives_strings_containers` function L372-380 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `returns` function L383-394 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `user_types_need_the_known_set` function L397-410 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `struct_renders_to_record` function L413-419 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `struct_with_nested_user_type` function L422-427 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `enum_renders_to_variant` function L430-439 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `struct_variant_synthesizes_a_record` function L442-451 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `multifield_tuple_variant_is_rejected` function L454-457 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `full_document_places_type_defs_before_funcs` function L460-483 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `streaming_method_renders_a_resource` function L486-501 — `()` — helper, and the `fidius wit` CLI can all share one implementation.
-  `stream_item_type_detects_marker` function L504-512 — `()` — helper, and the `fidius wit` CLI can all share one implementation.

### pluggable-poc/crates/emit-console/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/emit-console/src/lib.rs

- pub `ConsoleEmitPlugin` struct L19-23 — `{ max_rows: Option<usize>, total_rows: usize, batch_count: usize }` — Pretty-prints Arrow RecordBatches to stdout.
- pub `new` function L26-32 — `() -> Self`
-  `ConsoleEmitPlugin` type L25-33 — `= ConsoleEmitPlugin`
-  `ConsoleEmitPlugin` type L35-39 — `impl Default for ConsoleEmitPlugin`
-  `default` function L36-38 — `() -> Self`
-  `ConsoleEmitPlugin` type L41-83 — `impl EmitPlugin for ConsoleEmitPlugin`
-  `init` function L42-45 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `write_batch` function L47-73 — `(&mut self, input: &DataBatch) -> Result<(), PluginError>`
-  `finalize` function L75-82 — `(&mut self) -> Result<(), PluginError>`

### pluggable-poc/crates/ingest-csv/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/ingest-csv/src/lib.rs

- pub `CsvIngestPlugin` struct L25-29 — `{ reader: Option<arrow::csv::Reader<File>>, file_path: String, has_header: bool ...` — Reads a CSV file and produces Arrow RecordBatches.
- pub `new` function L32-38 — `() -> Self`
-  `CsvIngestPlugin` type L31-39 — `= CsvIngestPlugin`
-  `CsvIngestPlugin` type L41-45 — `impl Default for CsvIngestPlugin`
-  `default` function L42-44 — `() -> Self`
-  `CsvIngestPlugin` type L47-137 — `impl IngestPlugin for CsvIngestPlugin`
-  `init` function L48-90 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `next_batch` function L92-131 — `(&mut self, max_rows: usize) -> Result<Option<DataBatch>, PluginError>`
-  `close` function L133-136 — `(&mut self) -> Result<(), PluginError>`

### pluggable-poc/crates/pipeline-host/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/pipeline-host/src/config.rs

- pub `PipelineFile` struct L23-29 — `{ pipeline: PipelineMeta, ingest: StageConfig, transform: Vec<StageConfig>, emit...` — Top-level pipeline configuration parsed from TOML.
- pub `PipelineMeta` struct L32-38 — `{ name: String, mode: PipelineMode, batch_size: usize }`
- pub `StageConfig` struct L46-72 — `{ kind: String, plugin: Option<String>, script: Option<String>, entrypoint: Opti...` — Configuration for a single pipeline stage.
- pub `plugin_config` function L79-83 — `(&self) -> PluginConfig`
- pub `isolation_tier` function L85-87 — `(&self) -> IsolationTier`
- pub `timeout` function L89-91 — `(&self) -> u64`
- pub `load_pipeline` function L95-99 — `(path: &Path) -> anyhow::Result<PipelineFile>` — Load and parse a pipeline TOML file.
-  `default_batch_size` function L40-42 — `() -> usize`
-  `default_kind` function L74-76 — `() -> String`
-  `StageConfig` type L78-92 — `= StageConfig`

#### pluggable-poc/crates/pipeline-host/src/main.rs

-  `arrow_bridge` module L15 — `-`
-  `config` module L16 — `-`
-  `orchestrator` module L17 — `-`
-  `Cli` struct L31-39 — `{ pipeline: PathBuf, bench: bool }`
-  `main` function L41-101 — `() -> Result<()>`
-  `build_ingest` function L104-122 — `( stage: &StageConfig, _project_root: &Path, ) -> Result<Box<dyn IngestPlugin>>` — Build an ingest plugin from config.
-  `build_transform` function L125-204 — `( stage: &StageConfig, project_root: &Path, ) -> Result<Box<dyn TransformPlugin>...` — Build a transform plugin from config.
-  `build_emit` function L207-225 — `( stage: &StageConfig, _project_root: &Path, ) -> Result<Box<dyn EmitPlugin>>` — Build an emit plugin from config.

#### pluggable-poc/crates/pipeline-host/src/orchestrator.rs

- pub `Pipeline` struct L21-27 — `{ name: String, batch_size: usize, ingest: Box<dyn IngestPlugin>, transforms: Ve...` — Assembled pipeline ready to execute.
- pub `run` function L30-111 — `(pipeline: &mut Pipeline) -> Result<PipelineStats, PluginError>` — Run the pipeline: pull batches from ingest, push through transforms, emit.
- pub `PipelineStats` struct L114-123 — `{ batches: usize, rows_ingested: usize, rows_emitted: usize, total_time: Duratio...`
- pub `print_summary` function L126-169 — `(&self)`
-  `PipelineStats` type L125-170 — `= PipelineStats`
-  `pct` function L172-178 — `(part: Duration, total: Duration) -> f64`

### pluggable-poc/crates/pipeline-types/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/pipeline-types/src/lib.rs

- pub `IsolationTier` enum L26-32 — `Native | Thread | ZeroCopy | Process` — Isolation tier for plugin execution.
- pub `PipelineMode` enum L37-41 — `Batch | Streaming` — Pipeline execution mode.
- pub `DataBatch` struct L45-48 — `{ batch: RecordBatch, metadata: HashMap<String, String> }` — Data flowing between pipeline stages.
- pub `new` function L51-56 — `(batch: RecordBatch) -> Self`
- pub `with_metadata` function L58-60 — `(batch: RecordBatch, metadata: HashMap<String, String>) -> Self`
- pub `num_rows` function L62-64 — `(&self) -> usize`
- pub `PluginConfig` struct L69-71 — `{ params: HashMap<String, String> }` — Configuration passed to a plugin at init time.
- pub `PluginError` enum L75-90 — `InvalidConfig | Processing | Fatal | Timeout | Arrow` — Plugin error types with severity.
- pub `IngestPlugin` interface L95-101 — `{ fn init(), fn next_batch(), fn close() }` — Ingest plugin trait — pulls data into the pipeline.
- pub `TransformPlugin` interface L104-113 — `{ fn init(), fn process_batch(), fn flush(), fn close() }` — Transform plugin trait — processes data batches in the pipeline.
- pub `EmitPlugin` interface L116-122 — `{ fn init(), fn write_batch(), fn finalize() }` — Emit plugin trait — writes data out of the pipeline.
-  `DataBatch` type L50-65 — `= DataBatch`
-  `close` function L98-100 — `(&mut self) -> Result<(), PluginError>`
-  `flush` function L107-109 — `(&mut self) -> Result<Option<DataBatch>, PluginError>`
-  `close` function L110-112 — `(&mut self) -> Result<(), PluginError>`
-  `finalize` function L119-121 — `(&mut self) -> Result<(), PluginError>`

### pluggable-poc/crates/plugin-runtime/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/plugin-runtime/src/ffi_plugin.rs

- pub `FfiTransformPlugin` struct L35-38 — `{ dylib_path: PathBuf, library: Option<libloading::Library> }` — A transform plugin loaded from a compiled shared library (.dylib/.so) via FFI.
- pub `new` function L41-46 — `(dylib_path: PathBuf) -> Self`
-  `PluginInitFn` type L23 — `= unsafe extern "C" fn(*const u8, usize) -> i32` — Type aliases for the FFI function signatures exported by the plugin dylib.
-  `PluginProcessBatchFn` type L24-29 — `= unsafe extern "C" fn( *mut FFI_ArrowArray, *mut FFI_ArrowSchema, *mut FFI_Arro...`
-  `PluginCloseFn` type L30 — `= unsafe extern "C" fn()`
-  `FfiTransformPlugin` type L40-47 — `= FfiTransformPlugin`
-  `FfiTransformPlugin` type L49-141 — `impl TransformPlugin for FfiTransformPlugin`
-  `init` function L50-80 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L82-130 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`
-  `close` function L132-140 — `(&mut self) -> Result<(), PluginError>`
-  `FfiTransformPlugin` type L143-147 — `impl Drop for FfiTransformPlugin`
-  `drop` function L144-146 — `(&mut self)`

#### pluggable-poc/crates/plugin-runtime/src/lib.rs

- pub `ffi_plugin` module L15 — `-`
- pub `native` module L16 — `-`
- pub `pyo3_process` module L17 — `-`
- pub `pyo3_thread` module L18 — `-`
- pub `pyo3_zerocopy` module L19 — `-`
- pub `serialize_ipc` function L27-33 — `(batch: &RecordBatch) -> anyhow::Result<Vec<u8>>` — Serialize a RecordBatch to Arrow IPC stream bytes.
- pub `deserialize_ipc` function L36-44 — `(bytes: &[u8]) -> anyhow::Result<RecordBatch>` — Deserialize Arrow IPC stream bytes to a RecordBatch.

#### pluggable-poc/crates/plugin-runtime/src/pyo3_process.rs

- pub `PyO3ProcessTransform` struct L44-51 — `{ script_path: PathBuf, entrypoint: String, harness_path: PathBuf, config: Plugi...` — PyO3 process-isolated transform executor (Tier 3).
- pub `new` function L54-68 — `( script: impl Into<PathBuf>, entrypoint: &str, harness: impl Into<PathBuf>, tim...`
-  `MSG_INIT` variable L24 — `: u32`
-  `MSG_PROCESS_BATCH` variable L25 — `: u32`
-  `MSG_FLUSH` variable L26 — `: u32`
-  `MSG_CLOSE` variable L27 — `: u32`
-  `RESP_OK` variable L30 — `: u32`
-  `RESP_BATCH` variable L31 — `: u32`
-  `RESP_NONE` variable L32 — `: u32`
-  `RESP_ERROR` variable L33 — `: u32`
-  `PyO3ProcessTransform` type L53-69 — `= PyO3ProcessTransform`
-  `PyO3ProcessTransform` type L71-168 — `impl TransformPlugin for PyO3ProcessTransform`
-  `init` function L72-107 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L109-131 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`
-  `flush` function L133-150 — `(&mut self) -> Result<Option<DataBatch>, PluginError>`
-  `close` function L152-167 — `(&mut self) -> Result<(), PluginError>`
-  `PyO3ProcessTransform` type L170-174 — `impl Drop for PyO3ProcessTransform`
-  `drop` function L171-173 — `(&mut self)`
-  `send_message` function L176-202 — `(child: &mut Option<Child>, msg_type: u32, payload: &[u8]) -> Result<(), PluginE...`
-  `recv_message` function L204-230 — `(child: &mut Option<Child>) -> Result<(u32, Vec<u8>), PluginError>`

#### pluggable-poc/crates/plugin-runtime/src/pyo3_thread.rs

- pub `PyO3ThreadTransform` struct L32-39 — `{ script_path: PathBuf, entrypoint: String, config: PluginConfig, timeout_ms: u6...` — PyO3 thread-isolated transform executor (Tier 2).
- pub `new` function L42-50 — `(script: impl Into<PathBuf>, entrypoint: &str, timeout_ms: u64) -> Self`
-  `PyO3ThreadTransform` type L41-51 — `= PyO3ThreadTransform`
-  `PyO3ThreadTransform` type L53-139 — `impl TransformPlugin for PyO3ThreadTransform`
-  `init` function L54-95 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L97-133 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`
-  `close` function L135-138 — `(&mut self) -> Result<(), PluginError>`
-  `pyerr` function L141-143 — `(e: impl std::fmt::Display) -> PluginError`

#### pluggable-poc/crates/plugin-runtime/src/pyo3_zerocopy.rs

- pub `PyO3ZeroCopyTransform` struct L34-40 — `{ script_path: PathBuf, entrypoint: String, config: PluginConfig, timeout_ms: u6...` — PyO3 zero-copy transform executor (Tier 2+).
- pub `new` function L43-51 — `(script: impl Into<PathBuf>, entrypoint: &str, timeout_ms: u64) -> Self`
-  `PyO3ZeroCopyTransform` type L42-52 — `= PyO3ZeroCopyTransform`
-  `PyO3ZeroCopyTransform` type L54-160 — `impl TransformPlugin for PyO3ZeroCopyTransform`
-  `init` function L55-93 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L95-154 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`
-  `close` function L156-159 — `(&mut self) -> Result<(), PluginError>`
-  `pyerr` function L162-164 — `(e: impl std::fmt::Display) -> PluginError`

### pluggable-poc/crates/transform-double/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/transform-double/src/lib.rs

- pub `DoubleTransformPlugin` struct L26-28 — `{ columns: Option<Vec<String>> }` — Native Rust column doubler — uses Arrow's vectorized compute
- pub `new` function L31-33 — `() -> Self`
-  `DoubleTransformPlugin` type L30-34 — `= DoubleTransformPlugin`
-  `DoubleTransformPlugin` type L36-40 — `impl Default for DoubleTransformPlugin`
-  `default` function L37-39 — `() -> Self`
-  `DoubleTransformPlugin` type L42-87 — `impl TransformPlugin for DoubleTransformPlugin`
-  `init` function L43-50 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L52-86 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`

### pluggable-poc/crates/transform-normalize/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/transform-normalize/src/lib.rs

- pub `NormalizeTransformPlugin` struct L30-33 — `{ columns: Vec<String>, method: Method }` — Min-max or z-score normalization on specified columns.
- pub `new` function L36-41 — `() -> Self`
-  `Method` enum L24-27 — `MinMax | ZScore` — Normalization method.
-  `NormalizeTransformPlugin` type L35-42 — `= NormalizeTransformPlugin`
-  `NormalizeTransformPlugin` type L44-48 — `impl Default for NormalizeTransformPlugin`
-  `default` function L45-47 — `() -> Self`
-  `NormalizeTransformPlugin` type L50-112 — `impl TransformPlugin for NormalizeTransformPlugin`
-  `init` function L51-65 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L67-111 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`
-  `min_max_normalize` function L114-139 — `(array: &Float64Array) -> Result<Float64Array, PluginError>`
-  `z_score_normalize` function L141-163 — `(array: &Float64Array) -> Result<Float64Array, PluginError>`

### pluggable-poc/crates/transform-onnx/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/crates/transform-onnx/src/lib.rs

- pub `OnnxTransformPlugin` struct L28-33 — `{ model: Option<TractModel>, input_columns: Vec<String>, output_column: String, ...` — ONNX model inference via tract — runs a model on input columns and
- pub `new` function L36-43 — `() -> Self`
-  `TractModel` type L24 — `= SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>`
-  `OnnxTransformPlugin` type L35-44 — `= OnnxTransformPlugin`
-  `OnnxTransformPlugin` type L46-50 — `impl Default for OnnxTransformPlugin`
-  `default` function L47-49 — `() -> Self`
-  `OnnxTransformPlugin` type L52-155 — `impl TransformPlugin for OnnxTransformPlugin`
-  `init` function L53-85 — `(&mut self, config: &PluginConfig) -> Result<(), PluginError>`
-  `process_batch` function L87-154 — `(&mut self, input: DataBatch) -> Result<DataBatch, PluginError>`

### pluggable-poc/data

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/data/generate_data.py

- pub `generate` function L10-25 — `def generate(output_path: str, num_rows: int = 1000, seed: int = 42)`

### pluggable-poc/models

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/models/train_model.py

- pub `train_and_export` function L16-94 — `def train_and_export(output_path: str = "models/classifier.onnx", n_samples: int...`

### pluggable-poc/plugins/ffi/transform-double-ffi/src

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/plugins/ffi/transform-double-ffi/src/lib.rs

- pub `plugin_init` function L29-58 — `(config_json: *const u8, config_len: usize) -> i32` — Initialize the plugin with a JSON config string.
- pub `plugin_process_batch` function L66-106 — `( in_array: *mut FFI_ArrowArray, in_schema: *mut FFI_ArrowSchema, out_array: *mu...` — Process a single batch.
- pub `plugin_close` function L110-112 — `()` — Close the plugin and free resources.
-  `COLUMNS` variable L22 — `: Mutex<Option<Vec<String>>>` — Columns to double (None = all numeric columns).
-  `Config` struct L42-45 — `{ columns: Option<String> }`
-  `process_batch_inner` function L115-146 — `(batch: &RecordBatch) -> Result<RecordBatch, arrow::error::ArrowError>` — Inner processing logic — uses Arrow's vectorized compute kernels.

### pluggable-poc/plugins

> *Semantic summary to be generated by AI agent.*

#### pluggable-poc/plugins/harness.py

- pub `read_message` function L42-49 — `def read_message()` — Read a framed message from stdin.
- pub `write_message` function L52-58 — `def write_message(msg_type, payload=b"")` — Write a framed message to stdout.
- pub `ipc_to_table` function L61-64 — `def ipc_to_table(ipc_bytes: bytes) -> pa.Table` — Deserialize Arrow IPC stream bytes to a PyArrow Table.
- pub `table_to_ipc` function L67-74 — `def table_to_ipc(table: pa.Table) -> bytes` — Serialize a PyArrow Table to Arrow IPC stream bytes.
- pub `load_plugin` function L77-86 — `def load_plugin(script_path: str, entrypoint: str)` — Dynamically load a Python plugin module and return the entry function.
- pub `main` function L89-137 — `def main()`

#### pluggable-poc/plugins/transform_column_doubler.py

- pub `transform` function L15-54 — `def transform(ipc_bytes_or_table, params: dict)` — Double all numeric columns in the input.

### python/fidius

> *Semantic summary to be generated by AI agent.*

#### python/fidius/_errors.py

- pub `PluginError` class L32-57 — `(Exception) { __init__ }` — Structured plugin error that round-trips to the host with its fields intact.
- pub `__init__` method L45-54 — `def __init__( self, code: str, message: str, details: Optional[dict] = None, ) -...`
- pub `__repr__` method L56-57 — `def __repr__(self) -> str`

#### python/fidius/_registry.py

- pub `method` function L33-49 — `def method(func: Callable) -> Callable` — Register *func* under its ``__name__`` as a fidius plugin method.
- pub `get_method` function L52-70 — `def get_method(name: str, module: str | None = None) -> Callable` — Look up a previously-registered method.
- pub `list_methods` function L73-81 — `def list_methods(module: str | None = None) -> list[str]` — Return the sorted list of registered method names.
- pub `reset_registry` function L84-86 — `def reset_registry() -> None` — Clear the registry.

### python/tests

> *Semantic summary to be generated by AI agent.*

#### python/tests/test_sdk.py

- pub `test_method_registers_under_function_name` function L40-46 — `def test_method_registers_under_function_name()`
- pub `test_decorator_returns_function_unchanged` function L49-55 — `def test_decorator_returns_function_unchanged()`
- pub `test_multiple_methods_in_one_module` function L58-71 — `def test_multiple_methods_in_one_module()`
- pub `test_duplicate_registration_raises` function L74-83 — `def test_duplicate_registration_raises()`
- pub `test_get_method_unknown_raises_keyerror` function L86-88 — `def test_get_method_unknown_raises_keyerror()`
- pub `test_plugin_error_carries_code_message_details` function L91-97 — `def test_plugin_error_carries_code_message_details()`
- pub `test_plugin_error_details_optional` function L100-102 — `def test_plugin_error_details_optional()`
- pub `test_module_importable_from_vendor_layout` function L105-134 — `def test_module_importable_from_vendor_layout(tmp_path)` — Simulate the vendored-load pattern: copy fidius/ into a temp dir,

### tests/test-plugin-smoke/src

> *Semantic summary to be generated by AI agent.*

#### tests/test-plugin-smoke/src/lib.rs

- pub `Calculator` interface L21-34 — `{ fn add(), fn add_direct(), fn version(), fn multiply() }`
- pub `AddInput` struct L37-40 — `{ a: i64, b: i64 }`
- pub `AddOutput` struct L43-45 — `{ result: i64 }`
- pub `MulInput` struct L48-51 — `{ a: i64, b: i64 }`
- pub `MulOutput` struct L54-56 — `{ result: i64 }`
- pub `BasicCalculator` struct L58 — `-`
- pub `ArenaEcho` interface L86-88 — `{ fn echo() }`
- pub `ArenaEchoer` struct L90 — `-`
- pub `BytePipe` interface L103-110 — `{ fn reverse(), fn name() }`
- pub `ReverseBytes` struct L112 — `-`
- pub `Ticker` interface L134-137 — `{ fn tick() }`
- pub `TickerImpl` struct L142 — `-`
-  `BasicCalculator` type L61-81 — `impl Calculator for BasicCalculator`
-  `add` function L62-66 — `(&self, input: AddInput) -> AddOutput`
-  `add_direct` function L68-70 — `(&self, a: i64, b: i64) -> i64`
-  `version` function L72-74 — `(&self) -> String`
-  `multiply` function L76-80 — `(&self, input: MulInput) -> MulOutput`
-  `ArenaEchoer` type L93-97 — `impl ArenaEcho for ArenaEchoer`
-  `echo` function L94-96 — `(&self, input: String) -> String`
-  `ReverseBytes` type L115-125 — `impl BytePipe for ReverseBytes`
-  `reverse` function L117-120 — `(&self, mut data: Vec<u8>) -> Vec<u8>`
-  `name` function L122-124 — `(&self) -> String`
-  `TickerImpl` type L145-149 — `impl Ticker for TickerImpl`
-  `tick` function L146-148 — `(&self, count: u32) -> fidius::Stream<u64>`

### tests/wasm-fixtures/fetcher/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/fetcher/src/lib.rs

-  `Component` struct L19 — `-`
-  `Component` type L21-37 — `impl Guest for Component`
-  `fetch` function L25-30 — `(url: String) -> String` — Plain-string return so the host test never has to round-trip a WIT
-  `fidius_interface_hash` function L34-36 — `() -> u64` — Interface-hash carrier; the host's `load_wasm` checks it against the
-  `do_fetch` function L39-90 — `(url: String) -> Result<String, String>`

### tests/wasm-fixtures/greeter/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/greeter/src/lib.rs

-  `bindings` module L9 — `-`
-  `INTERFACE_HASH` variable L15 — `: u64` — Must match what the host expects for this interface.
-  `Component` struct L17 — `-`
-  `Component` type L19-47 — `impl Guest for Component`
-  `greet` function L20-22 — `(name: String) -> String`
-  `add` function L24-30 — `(a: i64, b: i64) -> Result<i64, PluginError>`
-  `echo_bytes` function L32-37 — `(data: Vec<u8>) -> Vec<u8>`
-  `fidius_interface_hash` function L39-41 — `() -> u64`
-  `probe_env` function L43-46 — `() -> bool`

### tests/wasm-fixtures/greeter-go

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/greeter-go/main.go

-  `init` function L17-34 — `func init()`
-  `main` function L36 — `func main()`

### tests/wasm-fixtures/greeter-py

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/greeter-py/app.py

- pub `Greeter` class L14-33 — `{ greet, add, echo_bytes, probe_env, fidius_interface_hash }` — Implements the exported `greeter` interface.
- pub `greet` method L17-18 — `def greet(self, name: str) -> str`
- pub `add` method L20-22 — `def add(self, a: int, b: int) -> int`
- pub `echo_bytes` method L24-25 — `def echo_bytes(self, data: bytes) -> bytes`
- pub `probe_env` method L27-29 — `def probe_env(self) -> bool`
- pub `fidius_interface_hash` method L31-33 — `def fidius_interface_hash(self) -> int`

### tests/wasm-fixtures/macro-fetcher/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/macro-fetcher/src/lib.rs

- pub `Fetcher` interface L13-16 — `{ fn fetch() }`
- pub `MyFetcher` struct L18 — `-`
-  `MyFetcher` type L21-28 — `impl Fetcher for MyFetcher`
-  `fetch` function L22-27 — `(&self, url: String) -> String`

### tests/wasm-fixtures/macro-greeter/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/macro-greeter/src/lib.rs

- pub `Greeter` interface L11-16 — `{ fn greet(), fn echo() }`
- pub `MyGreeter` struct L18 — `-`
-  `MyGreeter` type L21-32 — `impl Greeter for MyGreeter`
-  `greet` function L22-24 — `(&self, name: String) -> String`
-  `echo` function L27-31 — `(&self, data: Vec<u8>) -> Vec<u8>`

### tests/wasm-fixtures/macro-ticker/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/macro-ticker/src/lib.rs

- pub `Ticker` interface L13-16 — `{ fn tick() }`
- pub `MyTicker` struct L18 — `-`
-  `MyTicker` type L21-25 — `impl Ticker for MyTicker`
-  `tick` function L22-24 — `(&self, count: u32) -> fidius_guest::Stream<u64>`

### tests/wasm-fixtures/records-greeter

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/records-greeter/build.rs

-  `main` function L7-9 — `()`

### tests/wasm-fixtures/records-greeter/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/records-greeter/src/geom.rs

- pub `Point` struct L7-10 — `{ x: i32, y: i32 }`

#### tests/wasm-fixtures/records-greeter/src/lib.rs

- pub `geom` module L11 — `-`
- pub `Shape` enum L15-20 — `Circle | Rect | Triangle | Dot`
- pub `Geo` interface L23-26 — `{ fn midpoint(), fn describe() }`
- pub `MyGeo` struct L28 — `-`
-  `MyGeo` type L31-47 — `impl Geo for MyGeo`
-  `midpoint` function L32-37 — `(&self, a: Point, b: Point) -> Point`
-  `describe` function L39-46 — `(&self, s: Shape) -> String`

### tests/wasm-fixtures/ticker/src

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/ticker/src/lib.rs

-  `bindings` module L8 — `-`
-  `INTERFACE_HASH` variable L17 — `: u64`
-  `Component` struct L19 — `-`
-  `Ticks` struct L23-26 — `{ current: Cell<u64>, count: u64 }` — Guest-side stream state.
-  `Ticks` type L28-38 — `impl GuestTickStream for Ticks`
-  `next` function L29-37 — `(&self) -> Result<Option<u64>, PluginError>`
-  `Component` type L40-53 — `impl Guest for Component`
-  `TickStream` type L41 — `= Ticks`
-  `tick` function L43-48 — `(count: u32) -> TickStream`
-  `fidius_interface_hash` function L50-52 — `() -> u64`

### tests/wasm-fixtures/ticker-js

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/ticker-js/ticker.js

- pub `constructor` method L14-17 — `constructor(count)`
- pub `next` method L18-25 — `next()`
-  `TickStream` class L13-26 — `-`

### tests/wasm-fixtures/ticker-py

> *Semantic summary to be generated by AI agent.*

#### tests/wasm-fixtures/ticker-py/app.py

- pub `TickStream` class L19-31 — `(ticker_exports.TickStream) { __init__, next }` — The server-stream resource handle.
- pub `__init__` method L22-24 — `def __init__(self, count: int)`
- pub `next` method L26-31 — `def next(self) -> Optional[int]`
- pub `Ticker` class L34-42 — `{ tick, fidius_interface_hash }` — Implements the exported `ticker` interface (the free functions).
- pub `tick` method L37-38 — `def tick(self, count: int) -> TickStream`
- pub `fidius_interface_hash` method L40-42 — `def fidius_interface_hash(self) -> int`

### wasm-spike/guest/src

> *Semantic summary to be generated by AI agent.*

#### wasm-spike/guest/src/lib.rs

- pub `fd_alloc` function L34-41 — `(len: usize) -> *mut u8` — Allocate `len` bytes in the guest's linear memory and return the pointer.
- pub `fd_dealloc` function L45-51 — `(ptr: *mut u8, len: usize)` — Free a buffer previously returned by `fd_alloc` (or by `fd_call_raw`).
- pub `fd_call_raw` function L61-73 — `(ptr: *mut u8, len: usize) -> u64` — The raw-wire round trip.

### wasm-spike/host/src

> *Semantic summary to be generated by AI agent.*

#### wasm-spike/host/src/main.rs

-  `WARMUP` variable L30 — `: u32` — Run: cargo run --release -- <path-to-guest.wasm>
-  `ITERS` variable L31 — `: u32` — Run: cargo run --release -- <path-to-guest.wasm>
-  `bench` function L33-42 — `(iters: u32, mut f: F) -> f64` — Run: cargo run --release -- <path-to-guest.wasm>
-  `round_trip` function L46-68 — `( store: &mut Store<()>, memory: &wasmtime::Memory, alloc: &TypedFunc<u32, u32>,...` — One raw-wire round trip on a warm instance: write `input` into guest memory
-  `main` function L70-164 — `()` — Run: cargo run --release -- <path-to-guest.wasm>

### wasm-spike/twogen/src

> *Semantic summary to be generated by AI agent.*

#### wasm-spike/twogen/src/lib.rs

- pub `Impl` struct L12 — `-`
- pub `touch` function L29-32 — `() -> u32` — Touch a wasi:http type so the import is retained (not DCE'd).
-  `exp` module L6-19 — `-`
-  `Impl` type L13-17 — `impl Guest for Impl`
-  `ping` function L14-16 — `() -> u32`
-  `client` module L22-33 — `-`

