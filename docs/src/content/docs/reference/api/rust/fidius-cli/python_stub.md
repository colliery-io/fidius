---
title: "fidius-cli::python_stub"
description: "Python stub generator: turn a `#[plugin_interface]` trait into a `.py` file the plugin author imports. Gives them type-hinted signatures and the `__interface_hash__` constant the host expects at load time."
editUrl: false
sidebar:
  order: 3
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Python stub generator: turn a `#[plugin_interface]` trait into a `.py` file the plugin author imports. Gives them type-hinted signatures and the `__interface_hash__` constant the host expects at load time.

The stub computes its hash via `fidius_core::hash::interface_hash` over
signature strings produced by `fidius_core::hash::signature_string` —
the same call path the proc macro uses, so the two are guaranteed to
agree byte-for-byte.

## Functions

### <span id="fn.generate_stub"></span><code style="font-size: 0.85em">fn generate_​stub</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn generate_stub(
    interface_src: &Path,
    requested_trait: Option<&str>,
) -> Result<String>
```

Generate the contents of a Python stub file for the named trait found in `interface_src`. If `requested_trait` is `None`, exactly one `#[plugin_interface]` trait must be present.

<details>
<summary>Source</summary>

```rust
pub fn generate_stub(interface_src: &Path, requested_trait: Option<&str>) -> Result<String> {
    let source = std::fs::read_to_string(interface_src)?;
    let parsed: File = syn::parse_file(&source)
        .map_err(|e| format!("failed to parse {}: {e}", interface_src.display()))?;

    let traits: Vec<&ItemTrait> = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Trait(t) if has_plugin_interface_attr(t) => Some(t),
            _ => None,
        })
        .collect();

    let target = pick_trait(&traits, requested_trait, interface_src)?;
    let methods = extract_methods(target)?;

    Ok(render_python_stub(&target.ident.to_string(), &methods))
}
```

</details>

### <span id="fn.write_stub"></span><code style="font-size: 0.85em">fn write_​stub</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn write_stub(
    interface_src: &Path,
    out_path: &Path,
    requested_trait: Option<&str>,
) -> Result
```

Write the stub for the named trait to `out_path`. Special-cases `-` for stdout.

<details>
<summary>Source</summary>

```rust
pub fn write_stub(interface_src: &Path, out_path: &Path, requested_trait: Option<&str>) -> Result {
    let stub = generate_stub(interface_src, requested_trait)?;
    if out_path.as_os_str() == "-" {
        print!("{stub}");
    } else {
        let len = stub.len();
        std::fs::write(out_path, stub)?;
        println!(
            "Wrote Python stub for trait to {} ({} bytes)",
            out_path.display(),
            len
        );
    }
    Ok(())
}
```

</details>
