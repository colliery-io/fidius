# fidius-macro <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


## Functions

### `fidius-macro::plugin_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn plugin_interface (attr : TokenStream , item : TokenStream) -> TokenStream
```

Define a plugin interface from a trait.

Generates a `#[repr(C)]` vtable struct, interface hash constant,
capability bit constants, and a descriptor builder function.

**Examples:**

```ignore
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



### `fidius-macro::plugin_impl`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn plugin_impl (attr : TokenStream , item : TokenStream) -> TokenStream
```

Implement a plugin interface for a concrete type.

Generates extern "C" FFI shims, a static vtable, a plugin descriptor,
and a plugin registry.

**Examples:**

```ignore
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



### `fidius-macro::derive_wit_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


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
```ignore
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



