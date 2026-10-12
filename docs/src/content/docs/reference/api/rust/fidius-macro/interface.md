---
title: "fidius-macro::interface"
description: "Code generation for `#[plugin_interface]`."
editUrl: false
sidebar:
  order: 48
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Code generation for `#[plugin_interface]`.

Generates: the original trait, a `#[repr(C)]` vtable struct, interface hash constant,
capability bit constants, version/strategy constants, and a descriptor builder function.

## Functions

### <span id="fn.generate_interface"></span><code style="font-size: 0.85em">fn generate_​interface</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn generate_interface(ir: &InterfaceIR) -> syn::Result<TokenStream>
```

Generate all code for a `#[plugin_interface]` invocation.

<details>
<summary>Source</summary>

```rust
pub fn generate_interface(ir: &InterfaceIR) -> syn::Result<TokenStream> {
    // Both PluginAllocated and Arena are supported; vtable and shim codegen
    // branches on ir.attrs.buffer_strategy.
    match ir.attrs.buffer_strategy {
        BufferStrategyAttr::PluginAllocated | BufferStrategyAttr::Arena => {}
    }

    let cleaned_trait = strip_optional_attrs(&ir.original_trait);
    let vtable = generate_vtable(ir);
    let constants = generate_constants(ir);
    let descriptor_builder = generate_descriptor_builder(ir);
    let method_indices = generate_method_indices(ir);
    let metadata = generate_metadata(ir);
    let client = generate_client(ir);
    let companion_mod = format_ident!("__fidius_{}", ir.trait_name);
    Ok(quote! {
        #cleaned_trait
        /// Generated companion module for the plugin interface.
        ///
        /// Contains the VTable struct, interface hash, capability constants,
        /// vtable constructor, descriptor builder, and method index constants.
        /// Method indices follow trait declaration order (0-based).
        #[allow(non_snake_case, non_upper_case_globals, dead_code)]
        pub mod #companion_mod {
            use super::*;
            #vtable
            #constants
            #method_indices
            #metadata
            #descriptor_builder
        }
        #client
    })
}
```

</details>
