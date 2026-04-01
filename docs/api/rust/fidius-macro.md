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

**Attributes:**

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `version` | integer | yes | Interface version number. |
| `buffer` | ident | yes | Buffer strategy: `PluginAllocated`, `CallerAllocated`, or `Arena`. |
| `crate` | string | no | Path to the fidius crate (default: `"fidius"`). For white-label interfaces that re-export fidius, use `crate = "crate"` so generated code resolves through the interface crate. |

**Examples:**

```ignore
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[optional(since = 2)]
    fn greet_fancy(&self, name: String) -> String;
}

// White-label: resolve fidius through the current crate's re-export
#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "crate")]
pub trait MyPlugin: Send + Sync {
    fn execute(&self, input: String) -> String;
}
```

Methods can have zero, one, or multiple arguments. All arguments are
tuple-encoded at the FFI boundary automatically by the generated shims.

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

**Attributes:**

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| (positional) | ident | yes | Trait name to implement. |
| `crate` | string | no | Path to the fidius crate (default: `"fidius"`). Override for white-label scenarios where the plugin doesn't depend on fidius directly. |

**Examples:**

```ignore
pub struct MyGreeter;

#[plugin_impl(Greeter)]
impl Greeter for MyGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {name}!")
    }
}

// White-label: fidius accessed through the interface crate
#[plugin_impl(MyPlugin, crate = "my_interface::fidius")]
impl MyPlugin for MyImpl {
    fn execute(&self, input: String) -> String { ... }
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



