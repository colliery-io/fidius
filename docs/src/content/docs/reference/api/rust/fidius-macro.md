---
title: "fidius-macro"
description: "API reference of the Rust module fidius-macro."
sidebar:
  order: 46
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


## Functions

### <span id="fn.plugin_interface"></span>`fn plugin_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn plugin_interface (attr : TokenStream , item : TokenStream) -> TokenStream
```

Define a plugin interface from a trait.

Generates a `#[repr(C)]` vtable struct, interface hash constant,
capability bit constants, and a descriptor builder function.

**Examples:**

```rust
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[optional(since = 2)]
    fn greet_fancy(&self, name: String) -> String;
}
```

<details>
<summary>Source</summary>

```rust
pub fn plugin_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as InterfaceAttrs);
    let item_trait = parse_macro_input!(item as ItemTrait);

    match ir::parse_interface(attrs, &item_trait) {
        Ok(ir) => match interface::generate_interface(&ir) {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.to_compile_error().into(),
    }
}
```

</details>



### <span id="fn.host_interface"></span>`fn host_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn host_interface (attr : TokenStream , item : TokenStream) -> TokenStream
```

Define a **host interface** from a trait — the plugin → host callback channel (the reverse direction of [`macro@plugin_interface`]).

The host implements the trait and offers it to plugins as a C-ABI
function table; plugin code calls back into the host mid-execution
through a generated typed client. Arguments and returns are
bincode-serialized with the same wire conventions as the host → plugin
direction.

**Examples:**

```rust
// Interface crate — shared by host and plugins:
#[fidius::host_interface(version = 1)]
pub trait CloacinaHost: Send + Sync {
    fn release_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
    fn reclaim_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
}

// Plugin code — call the host mid-execution:
let host = CloacinaHostClient::bound()?;   // Err(NotBound) if the host didn't bind
host.release_slot(&id)?;

// Host application — implement + bind after loading the plugin library:
let lib = fidius_host::loader::load_library(path)?;
CloacinaHostBinding::bind(&lib, std::sync::Arc::new(MyHost { .. }))?;
```

<details>
<summary>Source</summary>

```rust
pub fn host_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as HostInterfaceAttrs);
    let item_trait = parse_macro_input!(item as ItemTrait);

    match host_interface::generate_host_interface(&attrs, &item_trait) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
```

</details>



### <span id="fn.plugin_impl"></span>`fn plugin_impl`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn plugin_impl (attr : TokenStream , item : TokenStream) -> TokenStream
```

Implement a plugin interface for a concrete type.

Generates extern "C" FFI shims, a static vtable, a plugin descriptor,
and a plugin registry.

**Examples:**

```rust
pub struct MyGreeter;

#[plugin_impl(Greeter)]
impl Greeter for MyGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {name}!")
    }
}
```

<details>
<summary>Source</summary>

```rust
pub fn plugin_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as PluginImplAttrs);
    let item_impl = parse_macro_input!(item as ItemImpl);

    match impl_macro::generate_plugin_impl(&attrs, &item_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
```

</details>



### <span id="fn.derive_wit_type"></span>`fn derive_wit_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn derive_wit_type (_item : TokenStream) -> TokenStream
```

Mark a `struct`/`enum` as usable in a WASM plugin interface (FIDIUS-I-0023).

This is a **marker** derive: it emits no code. The `fidius wit` generator
(run from `build.rs`) keys on the `#[derive(WitType)]` attribute when it
parses the crate source, mapping the struct to a WIT `record` (named fields)
or the enum to a WIT `variant` (unit / single-field cases) and emitting the
generated↔author conversions the wasm adapter uses. The same type continues
to cross the cdylib/Python boundary via serde, unchanged.
```rust
#[derive(WitType, serde::Serialize, serde::Deserialize, Clone)]
pub struct Point { pub x: i32, pub y: i32 }
```

<details>
<summary>Source</summary>

```rust
pub fn derive_wit_type(_item: TokenStream) -> TokenStream {
    // Intentionally empty — the build-time WIT generator reads the annotation
    // from source; no per-type codegen is needed here.
    TokenStream::new()
}
```

</details>



